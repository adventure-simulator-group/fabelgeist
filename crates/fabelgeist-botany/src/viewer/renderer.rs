//! Draws a [`Snapshot`] into a texture the window shows.
//!
//! Three passes a frame. A compute pass takes the cut through the forest's
//! hierarchy (`fabelgeist-hlod`) for the camera and for the sun -- plants up
//! close, groves' proxies ([`crate::viewer::hlod`]) far away, nothing outside the
//! view -- and sorts what each draws into a list per level of detail, a
//! plant's level picked from its size on screen; each list's length is the
//! instance count of an indirect draw. Then the sun's depth into the shadow
//! map, from its lists; then the scene with 4x multisampling -- alpha to
//! coverage on the leaves -- resolved into the texture registered with the
//! UI.

use std::sync::Arc;

use anyhow::Result;
use wgpu::util::DeviceExt;

use crate::viewer::camera::{self, Mat4, OrbitCamera};
use fabelgeist_hlod::View as HlodView;
use fabelgeist_hlod::gpu::GpuHierarchy;
use fabelgeist_surface_field::gpu::GpuSurfaceIndex;
use fabelgeist_surface_field::{Shape, SurfaceParams};

use crate::viewer::profile::Profile;
use crate::viewer::sim::{
    BranchInstance, LeafInstance, PlantInstance, Snapshot, StructureVertex, TerrainVertex,
};

const SAMPLES: u32 = 4;
const SHADOW_SIZE: u32 = 2048;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const MAX_SPECIES: usize = 16;

/// How a draw builds its instances.
struct Level {
    /// Around a branch tube, and along each segment.
    sides: u32,
    spans: u32,
    /// Blades per leaf cluster.
    blades: u32,
    /// Branches grow fillets where they meet: a lateral's rings gather
    /// where it leaves its parent, and every vertex moves onto the smooth
    /// union of the surfaces around it.
    fillets: bool,
}

/// In full detail -- as the profile asks -- in middle detail, coarsest, and
/// for the shadow map.
fn levels(profile: &Profile) -> [Level; 4] {
    [
        Level {
            sides: profile.tube_sides.max(3),
            spans: profile.tube_spans.max(1),
            blades: profile.leaf_blades.max(1),
            fillets: true,
        },
        Level {
            sides: 6,
            spans: 1,
            blades: 6,
            fillets: false,
        },
        Level {
            sides: 4,
            spans: 1,
            blades: 2,
            fillets: false,
        },
        Level {
            sides: 4,
            spans: 1,
            blades: 6,
            fillets: false,
        },
    ]
}
const SHADOW_LEVEL: usize = 3;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: Mat4,
    light_view_proj: Mat4,
    camera: [f32; 4],
    sun: [f32; 4],
    sun_color: [f32; 4],
    sky_zenith: [f32; 4],
    sky_horizon: [f32; 4],
    ground: [f32; 4],
    cam_forward: [f32; 4],
    cam_right: [f32; 4],
    cam_up: [f32; 4],
    /// The litter grid: its corner (x, z), one over its cell, its columns.
    litter: [f32; 4],
    /// x: how far a fillet reaches, in radii of the branch it grows from.
    detail: [f32; 4],
    /// How the shaders find their way in the surface field.
    surface: SurfaceParams,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpeciesGpu {
    bark: [f32; 4],
    leaf: [f32; 4],
}

/// `scene.wgsl`'s `Lod`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LodGpu {
    sides: u32,
    spans: u32,
    blades: u32,
    full: u32,
    base: u32,
    pad: [u32; 3],
}

/// How the scene is lit and what is drawn.
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// Sun azimuth, radians.
    pub sun_azimuth: f32,
    /// Sun elevation, radians.
    pub sun_elevation: f32,
    pub shadows: bool,
    pub leaves: bool,
}

/// A storage buffer that grows by half again when it runs out.
struct Storage {
    buffer: wgpu::Buffer,
    count: u32,
    label: &'static str,
    usage: wgpu::BufferUsages,
}

impl Storage {
    fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages) -> Self {
        Self {
            buffer: Self::allocate(device, label, usage, 256),
            count: 0,
            label,
            usage,
        }
    }

    fn allocate(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        size: u64,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size.next_multiple_of(16).max(256),
            usage,
            mapped_at_creation: false,
        })
    }

    /// Room for `bytes`; whether the buffer was replaced, and so needs
    /// binding again.
    fn reserve(&mut self, device: &wgpu::Device, bytes: u64) -> bool {
        if self.buffer.size() >= bytes {
            return false;
        }
        let size = bytes.max(self.buffer.size() * 3 / 2);
        self.buffer = Self::allocate(device, self.label, self.usage, size);
        true
    }

    fn upload<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[T],
    ) -> bool {
        let bytes: &[u8] = bytemuck::cast_slice(data);
        let grown = self.reserve(device, bytes.len() as u64);
        self.count = data.len() as u32;
        if !bytes.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytes);
        }
        grown
    }
}

struct Targets {
    width: u32,
    height: u32,
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
    resolve: Arc<wgpu::Texture>,
    resolve_view: Arc<wgpu::TextureView>,
}

struct Pipelines {
    branch: wgpu::RenderPipeline,
    leaf: wgpu::RenderPipeline,
    terrain: wgpu::RenderPipeline,
    structure: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    branch_shadow: wgpu::RenderPipeline,
    leaf_shadow: wgpu::RenderPipeline,
    terrain_shadow: wgpu::RenderPipeline,
    structure_shadow: wgpu::RenderPipeline,
}

struct Layouts {
    scene: wgpu::BindGroupLayout,
    lod: wgpu::BindGroupLayout,
}

/// Everything bound to the instance buffers, made again when one grows.
struct Bindings {
    scene: wgpu::BindGroup,
    /// Branches at each level, then leaves.
    lods: Vec<wgpu::BindGroup>,
}

pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    format: wgpu::TextureFormat,
    pipelines: Pipelines,
    layouts: Layouts,
    globals: wgpu::Buffer,
    species: wgpu::Buffer,
    lods: Vec<wgpu::Buffer>,
    plants: Storage,
    branches: Storage,
    leaves: Storage,
    /// The draw lists: branches at each level and for shadows, then leaves
    /// the same, each with room for every instance.
    lists: Storage,
    /// The forest's hierarchy, and the pass that takes cuts through it.
    hierarchy: GpuHierarchy,
    /// The surface field: shapes, bucket ranges, entries.
    surface_shapes: Storage,
    /// The shapes' index, built on the GPU from `surface_shapes`.
    surface_index: GpuSurfaceIndex,
    surface: SurfaceParams,
    /// Foliage over each cell of a grid on the ground (`sim::Litter`), and
    /// the grid, as `Globals::litter`.
    litter: Storage,
    litter_grid: [f32; 4],
    /// Eight indirect draws: branches at each level and for shadows, then
    /// leaves the same.
    draws: wgpu::Buffer,
    bindings: Bindings,
    shadow_group: wgpu::BindGroup,
    empty_group: wgpu::BindGroup,
    shadow_view: wgpu::TextureView,
    targets: Option<Targets>,
    terrain: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
    /// The buildings, when the scene has any.
    structures: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
    bounds: [[f32; 3]; 2],
    /// Whether the snapshot's branches grow fillets, and how far they reach.
    fillets: bool,
    fillet_size: f32,
}

/// The scene's shaders, which query the surface field.
fn scene_source() -> String {
    format!(
        "{}{}",
        fabelgeist_surface_field::wgsl::module(),
        include_str!("scene.wgsl")
    )
}

fn storage_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    read_only: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn entries<'a>(buffers: &[&'a wgpu::Buffer]) -> Vec<wgpu::BindGroupEntry<'a>> {
    buffers
        .iter()
        .enumerate()
        .map(|(i, buffer)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: buffer.as_entire_binding(),
        })
        .collect()
}

/// The six planes of the view frustum of `m` (column-major, depth zero to
/// one), each facing in and normalised.
fn frustum(m: &Mat4) -> [[f32; 4]; 6] {
    let row = |r: usize| [m[0][r], m[1][r], m[2][r], m[3][r]];
    let (r0, r1, r2, r3) = (row(0), row(1), row(2), row(3));
    let combine = |a: [f32; 4], b: [f32; 4], s: f32| -> [f32; 4] {
        let p = [
            a[0] + s * b[0],
            a[1] + s * b[1],
            a[2] + s * b[2],
            a[3] + s * b[3],
        ];
        let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-12);
        [p[0] / n, p[1] / n, p[2] / n, p[3] / n]
    };
    [
        combine(r3, r0, 1.0),
        combine(r3, r0, -1.0),
        combine(r3, r1, 1.0),
        combine(r3, r1, -1.0),
        combine(r2, r2, 0.0),
        combine(r3, r2, -1.0),
    ]
}

impl Renderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let source = scene_source();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("botany scene"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        use wgpu::ShaderStages as S;
        let layouts = Layouts {
            scene: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("botany scene"),
                entries: &[
                    uniform_entry(0, S::VERTEX_FRAGMENT),
                    storage_entry(1, S::VERTEX_FRAGMENT, true),
                    storage_entry(2, S::VERTEX, true),
                    storage_entry(3, S::VERTEX_FRAGMENT, true),
                    storage_entry(4, S::VERTEX, true),
                    storage_entry(5, S::VERTEX_FRAGMENT, true),
                    storage_entry(6, S::VERTEX_FRAGMENT, true),
                    storage_entry(7, S::VERTEX_FRAGMENT, true),
                    storage_entry(8, S::FRAGMENT, true),
                ],
            }),
            lod: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("botany level of detail"),
                entries: &[
                    uniform_entry(0, S::VERTEX),
                    storage_entry(1, S::VERTEX, true),
                ],
            }),
        };
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("botany shadow map"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: S::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: S::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("botany empty"),
            entries: &[],
        });

        let uniform = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let globals = uniform("botany globals", std::mem::size_of::<Globals>());
        let lods: Vec<wgpu::Buffer> = (0..8)
            .map(|_| uniform("botany level of detail", std::mem::size_of::<LodGpu>()))
            .collect();
        let species = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("botany species"),
            size: (MAX_SPECIES * std::mem::size_of::<SpeciesGpu>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let input = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let plants = Storage::new(&device, "botany plants", input);
        let branches = Storage::new(&device, "botany branches", input);
        let leaves = Storage::new(&device, "botany leaves", input);
        let lists = Storage::new(&device, "botany draw lists", wgpu::BufferUsages::STORAGE);
        let hierarchy = GpuHierarchy::new(&device);
        let surface_shapes = Storage::new(&device, "botany surface shapes", input);
        let surface_index = GpuSurfaceIndex::new(&device);
        let litter = Storage::new(&device, "botany litter", input);
        let draws = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("botany draws"),
            size: 8 * 16,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shadow = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("botany shadow map"),
            size: wgpu::Extent3d {
                width: SHADOW_SIZE,
                height: SHADOW_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow.create_view(&Default::default());
        let comparison = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("botany shadow compare"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("botany shadow map"),
            layout: &shadow_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&comparison),
                },
            ],
        });
        let empty_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("botany empty"),
            layout: &empty_layout,
            entries: &[],
        });

        let main_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("botany main"),
            bind_group_layouts: &[
                Some(&layouts.scene),
                Some(&shadow_layout),
                Some(&layouts.lod),
            ],
            immediate_size: 0,
        });
        let depth_only_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("botany depth"),
            bind_group_layouts: &[
                Some(&layouts.scene),
                Some(&empty_layout),
                Some(&layouts.lod),
            ],
            immediate_size: 0,
        });

        let terrain_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3],
        };

        let structure_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<StructureVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3, 3 => Float32],
        };

        let main = |label: &str,
                    vs: &str,
                    fs: &str,
                    buffers: &[wgpu::VertexBufferLayout],
                    cull: Option<wgpu::Face>,
                    coverage: bool,
                    depth_write: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&main_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: cull,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    mask: !0,
                    alpha_to_coverage_enabled: coverage,
                },
                multiview_mask: None,
                cache: None,
            })
        };
        let shadow_pipeline =
            |label: &str, vs: &str, fs: Option<&str>, buffers: &[wgpu::VertexBufferLayout]| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&depth_only_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some(vs),
                        compilation_options: Default::default(),
                        buffers,
                    },
                    fragment: fs.map(|fs| wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(fs),
                        compilation_options: Default::default(),
                        targets: &[],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::LessEqual),
                        stencil: Default::default(),
                        bias: wgpu::DepthBiasState {
                            constant: 2,
                            slope_scale: 2.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: Default::default(),
                    multiview_mask: None,
                    cache: None,
                })
            };

        let pipelines = Pipelines {
            branch: main(
                "botany branches",
                "branch_vs",
                "branch_fs",
                &[],
                Some(wgpu::Face::Back),
                false,
                true,
            ),
            leaf: main("botany leaves", "leaf_vs", "leaf_fs", &[], None, true, true),
            terrain: main(
                "botany terrain",
                "terrain_vs",
                "terrain_fs",
                std::slice::from_ref(&terrain_layout),
                None,
                false,
                true,
            ),
            structure: main(
                "botany buildings",
                "structure_vs",
                "structure_fs",
                std::slice::from_ref(&structure_layout),
                None,
                false,
                true,
            ),
            sky: main("botany sky", "sky_vs", "sky_fs", &[], None, false, false),
            branch_shadow: shadow_pipeline("botany branch shadow", "branch_shadow_vs", None, &[]),
            leaf_shadow: shadow_pipeline(
                "botany leaf shadow",
                "leaf_shadow_vs",
                Some("leaf_shadow_fs"),
                &[],
            ),
            terrain_shadow: shadow_pipeline(
                "botany terrain shadow",
                "terrain_shadow_vs",
                None,
                std::slice::from_ref(&terrain_layout),
            ),
            structure_shadow: shadow_pipeline(
                "botany buildings shadow",
                "structure_shadow_vs",
                None,
                std::slice::from_ref(&structure_layout),
            ),
        };

        let bindings = Self::bind(
            &device,
            &layouts,
            [&globals, &species],
            &lods,
            [&plants, &branches, &leaves, &lists, &litter],
            [
                &surface_shapes.buffer,
                surface_index.starts(),
                surface_index.entries(),
            ],
        );
        Ok(Self {
            device,
            queue,
            format,
            pipelines,
            layouts,
            globals,
            species,
            lods,
            plants,
            branches,
            leaves,
            lists,
            hierarchy,
            surface_shapes,
            surface_index,
            surface: SurfaceParams::default(),
            litter,
            litter_grid: [0.0; 4],
            draws,
            bindings,
            shadow_group,
            empty_group,
            shadow_view,
            targets: None,
            terrain: None,
            structures: None,
            bounds: [[-10.0, 0.0, -10.0], [10.0, 5.0, 10.0]],
            fillets: false,
            fillet_size: 0.0,
        })
    }

    /// Bind groups over the current instance buffers.
    fn bind(
        device: &wgpu::Device,
        layouts: &Layouts,
        [globals, species]: [&wgpu::Buffer; 2],
        lods: &[wgpu::Buffer],
        [plants, branches, leaves, lists, litter]: [&Storage; 5],
        [surface_shapes, surface_starts, surface_entries]: [&wgpu::Buffer; 3],
    ) -> Bindings {
        let scene = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("botany scene"),
            layout: &layouts.scene,
            entries: &entries(&[
                globals,
                species,
                &plants.buffer,
                &branches.buffer,
                &leaves.buffer,
                surface_shapes,
                surface_starts,
                surface_entries,
                &litter.buffer,
            ]),
        });
        let lods = lods
            .iter()
            .map(|lod| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("botany level of detail"),
                    layout: &layouts.lod,
                    entries: &entries(&[lod, &lists.buffer]),
                })
            })
            .collect();
        Bindings { scene, lods }
    }

    /// The texture the scene resolves into, and whether it was just made
    /// (and so needs registering with the UI again).
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
    ) -> (Arc<wgpu::Texture>, Arc<wgpu::TextureView>, bool) {
        let (width, height) = (width.max(1), height.max(1));
        let fresh = self
            .targets
            .as_ref()
            .is_none_or(|t| t.width != width || t.height != height);
        if fresh {
            let make = |label: &str,
                        format: wgpu::TextureFormat,
                        samples: u32,
                        usage: wgpu::TextureUsages| {
                self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
            };
            let color = make(
                "botany color",
                self.format,
                SAMPLES,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let depth = make(
                "botany depth",
                DEPTH,
                SAMPLES,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let resolve = Arc::new(make(
                "botany view",
                self.format,
                1,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            ));
            let resolve_view = Arc::new(resolve.create_view(&Default::default()));
            self.targets = Some(Targets {
                width,
                height,
                color: color.create_view(&Default::default()),
                depth: depth.create_view(&Default::default()),
                resolve,
                resolve_view,
            });
        }
        let t = self.targets.as_ref().expect("made above");
        (t.resolve.clone(), t.resolve_view.clone(), fresh)
    }

    pub fn bounds(&self) -> [[f32; 3]; 2] {
        self.bounds
    }

    /// Take a new snapshot's instances; its terrain, when it carries one.
    pub fn upload(&mut self, snapshot: &Snapshot) {
        let (device, queue) = (&self.device, &self.queue);
        let mut grown = self
            .plants
            .upload::<PlantInstance>(device, queue, &snapshot.plant_bounds);
        grown |= self
            .branches
            .upload::<BranchInstance>(device, queue, &snapshot.branches);
        grown |= self
            .leaves
            .upload::<LeafInstance>(device, queue, &snapshot.leaves);
        // The litter on the ground, and where its grid lies.
        let litter = &snapshot.litter;
        grown |= self.litter.upload::<[f32; 4]>(device, queue, &litter.cells);
        self.litter_grid = [
            litter.origin[0],
            litter.origin[1],
            1.0 / litter.cell.max(1e-3),
            litter.columns as f32,
        ];
        // The surface field: the shapes up, and their index built there.
        let field = &snapshot.blend;
        grown |= self
            .surface_shapes
            .upload::<Shape>(device, queue, &field.shapes);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("botany surface index"),
        });
        grown |= self.surface_index.record(
            device,
            queue,
            &mut encoder,
            &self.surface_shapes.buffer,
            field.shapes.len() as u32,
            field.cell,
            field.entries,
        );
        queue.submit([encoder.finish()]);
        self.surface = self.surface_index.params();
        // The hierarchy, and a list per level and one for shadows for each
        // stream, of every instance at most.
        self.hierarchy.upload(device, queue, &snapshot.hierarchy);
        let (branches, leaves) = (self.branches.count, self.leaves.count);
        grown |= self
            .lists
            .reserve(device, 16 * (branches as u64 + leaves as u64));
        if grown {
            self.bindings = Self::bind(
                device,
                &self.layouts,
                [&self.globals, &self.species],
                &self.lods,
                [
                    &self.plants,
                    &self.branches,
                    &self.leaves,
                    &self.lists,
                    &self.litter,
                ],
                [
                    &self.surface_shapes.buffer,
                    self.surface_index.starts(),
                    self.surface_index.entries(),
                ],
            );
        }
        self.fillets = snapshot.fillets;
        self.fillet_size = snapshot.fillet_size;

        let species: Vec<SpeciesGpu> = snapshot
            .species
            .iter()
            .take(MAX_SPECIES)
            .map(|s| SpeciesGpu {
                bark: [s.bark[0], s.bark[1], s.bark[2], 1.0],
                leaf: [s.leaf[0], s.leaf[1], s.leaf[2], s.foliage as u32 as f32],
            })
            .collect();
        queue.write_buffer(&self.species, 0, bytemuck::cast_slice(&species));
        if let Some(terrain) = &snapshot.terrain {
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("botany terrain vertices"),
                contents: bytemuck::cast_slice(&terrain.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("botany terrain indices"),
                contents: bytemuck::cast_slice(&terrain.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            self.terrain = Some((vertices, indices, terrain.indices.len() as u32));
        }
        if let Some(structures) = &snapshot.structures {
            self.structures = (!structures.indices.is_empty()).then(|| {
                let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("botany building vertices"),
                    contents: bytemuck::cast_slice(&structures.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("botany building indices"),
                    contents: bytemuck::cast_slice(&structures.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                (vertices, indices, structures.indices.len() as u32)
            });
        }
        self.bounds = snapshot.bounds;
    }

    /// The scene's lighting, and the sun's view over all of it.
    fn globals(&self, camera: &OrbitCamera, view: &View, aspect: f32, height: f32) -> Globals {
        let [lo, hi] = self.bounds;
        let center = camera::scale(camera::add(lo, hi), 0.5);
        let radius = self.radius();
        let (se, ce) = view.sun_elevation.sin_cos();
        let (sa, ca) = view.sun_azimuth.sin_cos();
        let sun = camera::normalize([ce * sa, se, ce * ca]);
        let light_eye = camera::add(center, camera::scale(sun, radius * 2.0));
        let up = if sun[1].abs() > 0.99 {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let light_view_proj = camera::multiply(
            &camera::orthographic(-radius, radius, -radius, radius, 0.1, radius * 4.0),
            &camera::look_at(light_eye, center, up),
        );

        // Low sun: warmer and dimmer.
        let warmth = (1.0 - se).clamp(0.0, 1.0);
        let sun_color = [3.2, 3.0 - 0.6 * warmth, 2.7 - 1.3 * warmth];
        let (forward, right, cam_up) = camera.basis();
        let eye = camera.eye();
        let tan_half = (camera.fov_y * 0.5).tan();
        let srgb_by_hand = !self.format.is_srgb();
        Globals {
            view_proj: camera.view_projection(aspect),
            light_view_proj,
            camera: [eye[0], eye[1], eye[2], 2.0 * tan_half / height.max(1.0)],
            sun: [sun[0], sun[1], sun[2], if view.shadows { 1.0 } else { 0.0 }],
            sun_color: [sun_color[0], sun_color[1], sun_color[2], 0.0],
            sky_zenith: [0.11, 0.3, 0.8, 0.0],
            sky_horizon: [0.5, 0.66, 0.88, 0.2 / (radius * 6.0)],
            ground: [0.3, 0.27, 0.2, 2.0 * radius / SHADOW_SIZE as f32],
            cam_forward: [forward[0], forward[1], forward[2], tan_half],
            cam_right: [right[0], right[1], right[2], aspect],
            cam_up: [
                cam_up[0],
                cam_up[1],
                cam_up[2],
                if srgb_by_hand { 1.0 } else { 0.0 },
            ],
            litter: self.litter_grid,
            detail: [self.fillet_size, 0.0, 0.0, 0.0],
            surface: self.surface,
        }
    }

    /// Half the diagonal of the scene's bounds: what the shadow map covers.
    fn radius(&self) -> f32 {
        let [lo, hi] = self.bounds;
        (camera::dot(camera::sub(hi, lo), camera::sub(hi, lo)).sqrt() * 0.5).max(5.0)
    }

    /// Draw the scene as `camera` sees it. Records and submits its own
    /// command buffer.
    pub fn render(&mut self, camera: &OrbitCamera, view: &View, profile: &Profile) {
        let Some(targets) = &self.targets else {
            return;
        };
        let aspect = targets.width as f32 / targets.height as f32;
        let globals = self.globals(camera, view, aspect, targets.height as f32);
        self.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // The draws the cuts fill in: each level's vertex count, its instance
        // count zeroed to be counted -- and how each level builds them.
        let (branches, leaves) = (self.branches.count, self.leaves.count);
        let levels = levels(profile);
        for (i, lod) in self.lods.iter().enumerate() {
            let (level, stride) = if i < 4 {
                (&levels[i], branches)
            } else {
                (&levels[i - 4], leaves)
            };
            let gpu = LodGpu {
                sides: level.sides,
                spans: level.spans,
                blades: level.blades,
                full: u32::from(level.fillets && self.fillets),
                base: if i < 4 { 0 } else { 4 * branches } + (i as u32 % 4) * stride,
                pad: [0; 3],
            };
            self.queue.write_buffer(lod, 0, bytemuck::bytes_of(&gpu));
        }
        let shadow_texel = 2.0 * self.radius() / SHADOW_SIZE as f32;
        let lists = HlodView {
            total_levels: 4,
            stream_base: [0, 4 * branches, 0, 0],
            stream_stride: [branches, leaves, 0, 0],
            ..HlodView::default()
        };
        // The camera's cut draws levels 0-2; the sun's, level 3.
        let views = [
            HlodView {
                planes: frustum(&globals.view_proj),
                eye: globals.camera,
                tolerance: profile.proxy_tolerance,
                levels_px: [profile.full_pixels, profile.middle_pixels],
                levels: 3,
                level_offset: 0,
                ..lists
            },
            HlodView {
                planes: frustum(&globals.light_view_proj),
                texel: shadow_texel,
                tolerance: profile.proxy_tolerance,
                levels: 1,
                level_offset: SHADOW_LEVEL as u32,
                ..lists
            },
        ];
        let mut draws = [[0u32; 4]; 8];
        for (i, level) in levels.iter().enumerate() {
            draws[i][0] = level.sides * level.spans * 6;
            draws[4 + i][0] = level.blades * 6;
        }
        self.queue
            .write_buffer(&self.draws, 0, bytemuck::cast_slice(&draws));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("botany frame"),
            });

        let views = if view.shadows {
            &views[..]
        } else {
            &views[..1]
        };
        self.hierarchy.record(
            &self.device,
            &self.queue,
            &mut encoder,
            views,
            &self.lists.buffer,
            &self.draws,
        );

        if view.shadows {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("botany shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bindings.scene, &[]);
            pass.set_bind_group(1, &self.empty_group, &[]);
            self.draw(&mut pass, true, view.leaves);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("botany scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color,
                    depth_slice: None,
                    resolve_target: Some(&targets.resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.bindings.scene, &[]);
            pass.set_bind_group(1, &self.shadow_group, &[]);
            self.draw(&mut pass, false, view.leaves);
            pass.set_pipeline(&self.pipelines.sky);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
    }

    /// The terrain, then each level's branches and leaves -- or, for the
    /// shadow map, the shadow lists.
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>, shadow: bool, leaves: bool) {
        let p = &self.pipelines;
        let lods = &self.bindings.lods;
        // Every group the layout has is bound before anything is drawn.
        pass.set_bind_group(2, &lods[0], &[]);
        if let Some((vertices, indices, count)) = &self.terrain {
            pass.set_pipeline(if shadow {
                &p.terrain_shadow
            } else {
                &p.terrain
            });
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..*count, 0, 0..1);
        }
        if let Some((vertices, indices, count)) = &self.structures {
            pass.set_pipeline(if shadow {
                &p.structure_shadow
            } else {
                &p.structure
            });
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..*count, 0, 0..1);
        }
        let levels: &[usize] = if shadow { &[SHADOW_LEVEL] } else { &[0, 1, 2] };
        if self.branches.count > 0 {
            pass.set_pipeline(if shadow { &p.branch_shadow } else { &p.branch });
            for &level in levels {
                pass.set_bind_group(2, &lods[level], &[]);
                pass.draw_indirect(&self.draws, level as u64 * 16);
            }
        }
        if leaves && self.leaves.count > 0 {
            pass.set_pipeline(if shadow { &p.leaf_shadow } else { &p.leaf });
            for &level in levels {
                pass.set_bind_group(2, &lods[4 + level], &[]);
                pass.draw_indirect(&self.draws, (4 + level) as u64 * 16);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(label: &str, source: &str) {
        let module = match naga::front::wgsl::parse_str(source) {
            Ok(module) => module,
            Err(error) => panic!("{label}: {}", error.emit_to_string(source)),
        };
        if let Err(error) = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        {
            panic!("{label}: {}", error.emit_to_string(source));
        }
    }

    #[test]
    fn the_shaders_validate() {
        validate("scene", &scene_source());
        validate("hlod", fabelgeist_hlod::gpu::source());
    }
}
