//! Drawing hair: strands, cards and the head, lit and shadowed.
//!
//! [`HairRenderer`] owns its pipelines and draws a groom prepared as
//! [`HairLods`] into a texture of its own, which a window can show or a
//! dataset can read back. A frame is up to six passes:
//!
//! 1. The head's depth from the key light -- its shadow on the hair and on
//!    itself.
//! 2. The hair's nearest depth from the key light, then
//! 3. its opacity in four layers behind that depth, added up (deep opacity
//!    maps, Yuksel & Keyser 2008). Both draw the policy's shadow strands --
//!    a thinned level, widened to the same coverage -- whatever is drawn in
//!    the view, cards included.
//! 4. The view, 4x multisampled into a half-float target: the backdrop, the
//!    head, and the hair as the [`LodPolicy`] picks it for the head's size
//!    on screen -- a prefix of a strand level or a card mesh. Partial
//!    coverage becomes a random share of the four samples, fresh every
//!    frame (stochastic transparency), so any number of sub-pixel strands
//!    add up in any order.
//! 5. With accumulation, the resolved frame is blended into a running mean;
//!    each frame's projection is jittered by a Halton offset, so a still
//!    view converges to a supersampled image (the cinematic policy takes 64
//!    frames, like a path tracer's samples).
//! 6. Tone mapping (ACES) into the output texture.
//!
//! With real-time physics ([`HairRenderer::set_dynamics`]), a compute pass
//! first moves every drawn strand of the levels in use by the simulated
//! strands it is bound to ([`crate::sim`]), and the head, cards and roots
//! follow the head's transform.

mod math;

use std::sync::Arc;

use crate::gpu::error::{HairGpuError, Result};
use fabelgeist_math::Vec3;
use wgpu::util::DeviceExt;

pub use math::{Mat4, halton, look_at, multiply, orthographic, perspective};

use crate::cards::{CardMesh, CardVertex};
use crate::lod::{Blend, HairLods, LodCounts, LodPolicy, Representation, StrandLevel};
use crate::material::HairMaterial;
use crate::sim::{Rigid, SimBinding};
use crate::surface::Surface;

const SAMPLES: u32 = 4;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Where the view is from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub fov_y: f32,
}

impl Camera {
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let distance = math::length(math::sub(self.target, self.eye));
        let near = (distance * 0.02).max(0.005);
        let far = distance * 10.0 + 5.0;
        let forward = math::normalize(math::sub(self.target, self.eye));
        let up = if forward[1].abs() > 0.99 {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        multiply(
            &perspective(self.fov_y, aspect, near, far),
            &look_at(self.eye, self.target, up),
        )
    }
}

/// A directional light: towards it, its colour and strength.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    /// From an azimuth (around the vertical, from the front) and an
    /// elevation, radians.
    pub fn from_angles(azimuth: f32, elevation: f32, color: [f32; 3], intensity: f32) -> Self {
        let (sa, ca) = azimuth.sin_cos();
        let (se, ce) = elevation.sin_cos();
        Self {
            direction: [ce * sa, se, ce * ca],
            color,
            intensity,
        }
    }
}

/// A three-point studio: key, fill and rim, a sky for ambient light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lighting {
    pub key: Light,
    pub fill: Light,
    pub rim: Light,
    pub sky_top: [f32; 3],
    pub sky_bottom: [f32; 3],
    pub exposure: f32,
    pub skin: [f32; 3],
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            key: Light::from_angles(0.7, 0.75, [1.0, 0.96, 0.9], 2.4),
            fill: Light::from_angles(-1.1, 0.2, [0.75, 0.85, 1.0], 0.5),
            rim: Light::from_angles(2.9, 0.5, [1.0, 0.95, 0.9], 1.8),
            sky_top: [0.55, 0.62, 0.72],
            sky_bottom: [0.18, 0.16, 0.15],
            exposure: 0.8,
            skin: [0.62, 0.45, 0.36],
        }
    }
}

/// What a frame drew.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameInfo {
    pub representation: Representation,
    /// The head's height on screen.
    pub pixels: f32,
    /// Frames in the running mean.
    pub frames: u32,
    /// Strands or card quads drawn.
    pub primitives: usize,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: Mat4,
    light_view_proj: Mat4,
    head: Mat4,
    camera: [f32; 4],
    jitter: [f32; 4],
    viewport: [f32; 4],
    key_dir: [f32; 4],
    key_color: [f32; 4],
    fill_dir: [f32; 4],
    fill_color: [f32; 4],
    rim_dir: [f32; 4],
    rim_color: [f32; 4],
    sky_top: [f32; 4],
    sky_bottom: [f32; 4],
    base_color: [f32; 4],
    material: [f32; 4],
    taper: [f32; 4],
    dom: [f32; 4],
    shadow: [f32; 4],
    mode: [f32; 4],
    skin: [f32; 4],
    roots: [f32; 4],
    waves: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawGpu {
    points: u32,
    count: u32,
    pad: [u32; 2],
    width_scale: f32,
    min_pixels: f32,
    pad2: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostGpu {
    params: [f32; 4],
}

/// The strand draws a frame makes: the view's, the shadows', and the
/// coarser level's fading in under the view's.
const MAIN: usize = 0;
const SHADOW: usize = 1;
const BLEND: usize = 2;

/// Which cards a card draw keeps: those whose own random number is in
/// `keep.x .. keep.y`, so two cuts of cards dissolve into each other.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CardDrawGpu {
    keep: [f32; 4],
}

struct GpuLevel {
    points: u32,
    count: u32,
    groups: [wgpu::BindGroup; 3],
    positions: Arc<wgpu::Buffer>,
    _infos: Arc<wgpu::Buffer>,
}

/// Real-time physics on the GPU: the binding, the simulated strands, and
/// for each level its rest pose and the pass that deforms it.
struct Dynamics {
    _ids: Arc<wgpu::Buffer>,
    _weights: Arc<wgpu::Buffer>,
    _guide_rest: Arc<wgpu::Buffer>,
    /// Written from the host, or the GPU simulation's own.
    guide_now: Arc<wgpu::Buffer>,
    /// The body the drawn strands are kept off ([`crate::gpu::head_shape`]),
    /// if any, and how far.
    _body: Arc<wgpu::Buffer>,
    body: Option<f32>,
    levels: Vec<DeformLevel>,
    /// The simulated strands moved since the levels were deformed.
    dirty: bool,
}

struct DeformLevel {
    rest: wgpu::Buffer,
    params: wgpu::Buffer,
    group: wgpu::BindGroup,
    sim_points: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DeformGpu {
    head: Mat4,
    points: u32,
    sim_points: u32,
    strands: u32,
    body: u32,
    offset: f32,
    pad: [u32; 3],
}

const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The head's surface as drawn: where to measure the hair's cover.
struct HeadSurface {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
}

struct GpuMesh {
    vertices: Arc<wgpu::Buffer>,
    indices: Arc<wgpu::Buffer>,
    count: u32,
}

struct Targets {
    width: u32,
    height: u32,
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
    resolved: wgpu::TextureView,
    accum: [wgpu::TextureView; 2],
    output: Arc<wgpu::Texture>,
    output_view: Arc<wgpu::TextureView>,
    /// Accumulate from history `i` into `1 - i`.
    accumulate: [wgpu::BindGroup; 2],
    /// Tone map the resolved frame, or history `i`.
    tonemap_resolved: wgpu::BindGroup,
    tonemap_accum: [wgpu::BindGroup; 2],
}

struct ShadowMaps {
    resolution: u32,
    depth: wgpu::TextureView,
    layers: wgpu::TextureView,
    head: wgpu::TextureView,
}

struct Layouts {
    globals: wgpu::BindGroupLayout,
    strands: wgpu::BindGroupLayout,
    cards: wgpu::BindGroupLayout,
    lighting: wgpu::BindGroupLayout,
    opacity: wgpu::BindGroupLayout,
    post: wgpu::BindGroupLayout,
}

struct Pipelines {
    strands: wgpu::RenderPipeline,
    cards: wgpu::RenderPipeline,
    head: wgpu::RenderPipeline,
    backdrop: wgpu::RenderPipeline,
    strand_depth: wgpu::RenderPipeline,
    strand_opacity: wgpu::RenderPipeline,
    head_depth: wgpu::RenderPipeline,
    accumulate: wgpu::RenderPipeline,
    tonemap: wgpu::RenderPipeline,
}

pub struct HairRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    output_format: wgpu::TextureFormat,
    layouts: Layouts,
    pipelines: Pipelines,
    globals: wgpu::Buffer,
    globals_group: wgpu::BindGroup,
    draws: [wgpu::Buffer; 3],
    card_draws: [wgpu::Buffer; 2],
    card_groups: [wgpu::BindGroup; 2],
    post: [wgpu::Buffer; 2],
    empty_group: wgpu::BindGroup,
    linear: wgpu::Sampler,
    comparison: wgpu::Sampler,
    levels: Vec<GpuLevel>,
    dynamics: Option<Dynamics>,
    head_matrix: Mat4,
    deform_layout: wgpu::BindGroupLayout,
    deform_pipeline: wgpu::ComputePipeline,
    cards: Vec<GpuMesh>,
    card_quads: Vec<usize>,
    /// The sizes of the groom's levels, to choose among them.
    counts: LodCounts,
    atlas: wgpu::TextureView,
    atlas_variants: u32,
    head: Option<GpuMesh>,
    /// The head's surface, to measure how much hair covers it.
    head_surface: Option<HeadSurface>,
    /// How much hair covers each of the head's vertices.
    head_cover: Option<Arc<wgpu::Buffer>>,
    shadows: ShadowMaps,
    lighting_group: wgpu::BindGroup,
    opacity_group: wgpu::BindGroup,
    targets: Option<Targets>,
    /// The bounds everything drawn fits in, for the light's frustum.
    bounds: [[f32; 3]; 2],
    center: [f32; 3],
    /// The accumulation: frames in the mean, which history holds it, and
    /// what the view was.
    frames: u32,
    history: usize,
    last: Option<(Globals, Representation, Option<Blend>)>,
    generation: u64,
    last_generation: u64,
    /// The simulated strands moved since the last frame: keep blending
    /// frames (a temporal filter) rather than starting the mean again.
    moving: bool,
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

fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32, kind: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(kind),
        count: None,
    }
}

/// The WGSL of the scene passes and of the full-screen passes after them.
pub const HAIR_WGSL: &str = include_str!("hair.wgsl");
pub const POST_WGSL: &str = include_str!("post.wgsl");
pub const DEFORM_WGSL: &str = include_str!("deform.wgsl");

/// The deform pass's source: [`DEFORM_WGSL`] and the head's distance field
/// it keeps the drawn strands off.
fn deform_source() -> String {
    format!(
        "{DEFORM_WGSL}{}{}",
        crate::gpu::wgsl::common::MATH,
        crate::gpu::wgsl::common::HEAD
    )
}

impl HairRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        output_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        use wgpu::ShaderStages as S;
        let d = &*device;
        let float = wgpu::TextureSampleType::Float { filterable: true };
        let unfilterable = wgpu::TextureSampleType::Float { filterable: false };
        let layouts = Layouts {
            globals: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair globals"),
                entries: &[uniform_entry(0, S::VERTEX | S::FRAGMENT)],
            }),
            strands: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair strands"),
                entries: &[
                    storage_entry(0),
                    storage_entry(1),
                    uniform_entry(2, S::VERTEX | S::FRAGMENT),
                ],
            }),
            cards: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair cards"),
                entries: &[uniform_entry(3, S::FRAGMENT)],
            }),
            lighting: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair lighting"),
                entries: &[
                    texture_entry(0, wgpu::TextureSampleType::Depth),
                    texture_entry(1, float),
                    texture_entry(2, wgpu::TextureSampleType::Depth),
                    sampler_entry(3, wgpu::SamplerBindingType::Comparison),
                    sampler_entry(4, wgpu::SamplerBindingType::Filtering),
                    texture_entry(5, float),
                ],
            }),
            opacity: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair opacity"),
                entries: &[texture_entry(0, wgpu::TextureSampleType::Depth)],
            }),
            post: d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hair post"),
                entries: &[
                    texture_entry(0, unfilterable),
                    texture_entry(1, unfilterable),
                    uniform_entry(2, S::FRAGMENT),
                ],
            }),
        };
        let empty_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hair empty"),
            entries: &[],
        });
        let empty_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hair empty"),
            layout: &empty_layout,
            entries: &[],
        });

        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hair"),
            source: wgpu::ShaderSource::Wgsl(HAIR_WGSL.into()),
        });
        let post_shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hair post"),
            source: wgpu::ShaderSource::Wgsl(POST_WGSL.into()),
        });
        let layout = |label: &str, groups: &[&wgpu::BindGroupLayout]| {
            let groups: Vec<Option<&wgpu::BindGroupLayout>> =
                groups.iter().map(|g| Some(*g)).collect();
            d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &groups,
                immediate_size: 0,
            })
        };
        let scene_layout = layout(
            "hair scene",
            &[&layouts.globals, &layouts.strands, &layouts.lighting],
        );
        let mesh_layout = layout(
            "hair mesh",
            &[&layouts.globals, &empty_layout, &layouts.lighting],
        );
        let card_layout = layout(
            "hair cards",
            &[&layouts.globals, &layouts.cards, &layouts.lighting],
        );
        let globals_layout = layout("hair globals only", &[&layouts.globals]);
        let light_layout = layout("hair light", &[&layouts.globals, &layouts.strands]);
        let opacity_layout = layout(
            "hair opacity",
            &[&layouts.globals, &layouts.strands, &layouts.opacity],
        );
        let post_layout = layout("hair post", &[&layouts.post]);

        let depth_state = |write: bool, compare: wgpu::CompareFunction| wgpu::DepthStencilState {
            format: DEPTH,
            depth_write_enabled: Some(write),
            depth_compare: Some(compare),
            stencil: Default::default(),
            bias: Default::default(),
        };
        let card_attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32, 2 => Float32x3, 3 => Float32,
            4 => Float32x3, 5 => Float32, 6 => Float32, 7 => Float32
        ];
        let head_attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        let cover_attributes = wgpu::vertex_attr_array![2 => Float32x2];
        let card_buffer = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CardVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &card_attributes,
        };
        let head_buffer = wgpu::VertexBufferLayout {
            array_stride: 24,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &head_attributes,
        };
        // How much hair covers each vertex, a buffer of its own: written on
        // the host for a groom made there, the groomer's own for one made on
        // the GPU.
        let cover_buffer = wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &cover_attributes,
        };
        #[allow(clippy::too_many_arguments)]
        let scene = |label: &str,
                     layout: &wgpu::PipelineLayout,
                     vs: &str,
                     fs: &str,
                     buffers: &[wgpu::VertexBufferLayout],
                     topology: wgpu::PrimitiveTopology,
                     cull: Option<wgpu::Face>,
                     coverage: bool,
                     depth: wgpu::DepthStencilState| {
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
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
                        format: HDR,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
                    cull_mode: cull,
                    ..Default::default()
                },
                depth_stencil: Some(depth),
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    mask: !0,
                    alpha_to_coverage_enabled: coverage,
                },
                multiview_mask: None,
                cache: None,
            })
        };
        let less = wgpu::CompareFunction::LessEqual;
        let strip = wgpu::PrimitiveTopology::TriangleStrip;
        let list = wgpu::PrimitiveTopology::TriangleList;
        let shadow_bias = wgpu::DepthBiasState {
            constant: 2,
            slope_scale: 1.5,
            clamp: 0.0,
        };
        let full_screen = |label: &str, fs: &str, format: wgpu::TextureFormat| {
            d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&post_layout),
                vertex: wgpu::VertexState {
                    module: &post_shader,
                    entry_point: Some("screen_vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &post_shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipelines = Pipelines {
            strands: scene(
                "hair strands",
                &scene_layout,
                "strand_vs",
                "strand_fs",
                &[],
                strip,
                None,
                false,
                depth_state(true, less),
            ),
            cards: scene(
                "hair cards",
                &card_layout,
                "card_vs",
                "card_fs",
                &[card_buffer],
                list,
                None,
                false,
                depth_state(true, less),
            ),
            head: scene(
                "hair head",
                &mesh_layout,
                "head_vs",
                "head_fs",
                &[head_buffer.clone(), cover_buffer],
                list,
                Some(wgpu::Face::Back),
                false,
                depth_state(true, less),
            ),
            backdrop: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("hair backdrop"),
                layout: Some(&globals_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("screen_vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("backdrop_fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Always)),
                multisample: wgpu::MultisampleState {
                    count: SAMPLES,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            }),
            strand_depth: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("hair strand depth"),
                layout: Some(&light_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("strand_light_vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: strip,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    bias: shadow_bias,
                    ..depth_state(true, less)
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            }),
            strand_opacity: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("hair strand opacity"),
                layout: Some(&opacity_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("strand_light_vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("strand_opacity_fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: strip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            }),
            head_depth: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("hair head depth"),
                layout: Some(&globals_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("head_light_vs"),
                    compilation_options: Default::default(),
                    buffers: &[head_buffer],
                },
                fragment: None,
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    bias: shadow_bias,
                    ..depth_state(true, less)
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            }),
            accumulate: full_screen("hair accumulate", "accumulate_fs", HDR),
            tonemap: full_screen("hair tonemap", "tonemap_fs", output_format),
        };

        let globals = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hair globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hair globals"),
            layout: &layouts.globals,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        let small_uniform = |label: &str, size: usize| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let draws = [
            small_uniform("hair draw", std::mem::size_of::<DrawGpu>()),
            small_uniform("hair shadow draw", std::mem::size_of::<DrawGpu>()),
            small_uniform("hair blend draw", std::mem::size_of::<DrawGpu>()),
        ];
        let card_draws = [
            small_uniform("hair card draw", std::mem::size_of::<CardDrawGpu>()),
            small_uniform("hair card blend draw", std::mem::size_of::<CardDrawGpu>()),
        ];
        let card_groups = [0, 1].map(|i| {
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("hair card draw"),
                layout: &layouts.cards,
                entries: &[wgpu::BindGroupEntry {
                    binding: 3,
                    resource: card_draws[i].as_entire_binding(),
                }],
            })
        });
        let post = [
            small_uniform("hair accumulate", 16),
            small_uniform("hair tonemap", 16),
        ];
        let linear = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hair linear"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let comparison = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hair shadow compare"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let compute_entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: S::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let deform_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hair deform"),
            entries: &[
                compute_entry(0, true),
                compute_entry(1, false),
                compute_entry(2, true),
                compute_entry(3, true),
                compute_entry(4, true),
                compute_entry(5, true),
                uniform_entry(6, S::COMPUTE),
                compute_entry(7, true),
            ],
        });
        let deform_shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hair deform"),
            source: wgpu::ShaderSource::Wgsl(deform_source().into()),
        });
        let deform_pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("hair deform"),
            layout: Some(&d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("hair deform"),
                bind_group_layouts: &[Some(&deform_layout)],
                immediate_size: 0,
            })),
            module: &deform_shader,
            entry_point: Some("deform"),
            compilation_options: Default::default(),
            cache: None,
        });
        let atlas = Self::atlas_texture(d, &queue, 1, 1, &[vec![0; 4]]);
        let shadows = Self::shadow_maps(d, 16);
        let (lighting_group, opacity_group) =
            Self::lighting_groups(d, &layouts, &shadows, &atlas, &linear, &comparison);
        Ok(Self {
            device,
            queue,
            output_format,
            layouts,
            pipelines,
            globals,
            globals_group,
            draws,
            card_draws,
            card_groups,
            post,
            empty_group,
            linear,
            comparison,
            levels: Vec::new(),
            dynamics: None,
            head_matrix: IDENTITY,
            deform_layout,
            deform_pipeline,
            cards: Vec::new(),
            card_quads: Vec::new(),
            counts: LodCounts::default(),
            atlas,
            atlas_variants: 1,
            head: None,
            head_surface: None,
            head_cover: None,
            shadows,
            lighting_group,
            opacity_group,
            targets: None,
            bounds: [[-0.2, -0.3, -0.2], [0.2, 0.2, 0.2]],
            center: [0.0; 3],
            frames: 0,
            history: 0,
            last: None,
            generation: 0,
            last_generation: u64::MAX,
            moving: false,
        })
    }

    fn atlas_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        levels: &[Vec<u8>],
    ) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hair card atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, data) in levels.iter().enumerate() {
            let w = (width >> level).max(1);
            let h = (height >> level).max(1);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w * 4),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }
        texture.create_view(&Default::default())
    }

    fn shadow_maps(device: &wgpu::Device, resolution: u32) -> ShadowMaps {
        let texture = |label: &str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: resolution,
                        height: resolution,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        ShadowMaps {
            resolution,
            depth: texture("hair shadow depth", DEPTH),
            layers: texture("hair shadow layers", HDR),
            head: texture("hair head shadow", DEPTH),
        }
    }

    fn lighting_groups(
        device: &wgpu::Device,
        layouts: &Layouts,
        shadows: &ShadowMaps,
        atlas: &wgpu::TextureView,
        linear: &wgpu::Sampler,
        comparison: &wgpu::Sampler,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let lighting = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hair lighting"),
            layout: &layouts.lighting,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadows.depth),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadows.layers),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&shadows.head),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(comparison),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(linear),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(atlas),
                },
            ],
        });
        let opacity = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hair opacity"),
            layout: &layouts.opacity,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&shadows.depth),
            }],
        });
        (lighting, opacity)
    }

    fn rebind_lighting(&mut self) {
        let (lighting, opacity) = Self::lighting_groups(
            &self.device,
            &self.layouts,
            &self.shadows,
            &self.atlas,
            &self.linear,
            &self.comparison,
        );
        self.lighting_group = lighting;
        self.opacity_group = opacity;
    }

    /// Upload a groom: every strand level, the cards and their atlas. A
    /// level larger than one storage binding may hold keeps the prefix of
    /// its strands that fits -- an even sample of the rest.
    pub fn set_groom(&mut self, lods: &HairLods) {
        let limit = self.device.limits().max_storage_buffer_binding_size as usize;
        // New strands: any physics bound to the old ones is gone.
        self.dynamics = None;
        self.head_matrix = IDENTITY;
        self.levels = lods
            .levels
            .iter()
            .map(|level| self.upload_level(level, limit))
            .collect();
        self.cards = lods.cards.iter().map(|c| self.upload_cards(c)).collect();
        self.card_quads = lods.cards.iter().map(|c| c.indices.len() / 6).collect();
        self.counts = lods.counts();
        let atlas = &lods.atlas;
        self.atlas = Self::atlas_texture(
            &self.device,
            &self.queue,
            atlas.width,
            atlas.height,
            &atlas.levels,
        );
        self.atlas_variants = atlas.variants;
        self.rebind_lighting();
        let [lo, hi] = lods.bounds;
        // The head is in the light's frustum too, however short the hair.
        self.bounds = [
            lo.min(Vec3::new(-0.12, -0.3, -0.13)).to_array(),
            hi.max(Vec3::new(0.12, 0.13, 0.13)).to_array(),
        ];
        self.center = lods.center.to_array();
        self.cover_head(lods);
        self.generation += 1;
    }

    /// Upload a groom made on the GPU ([`crate::gpu::GpuGroomer::lods`]):
    /// nothing is copied -- the levels, the cards and the head's cover are
    /// drawn from the buffers the groomer's kernels wrote.
    pub fn set_gpu_groom(&mut self, lods: &crate::gpu::GpuLods) {
        self.dynamics = None;
        self.head_matrix = IDENTITY;
        self.levels = lods
            .levels
            .iter()
            .map(|l| {
                self.level_groups(
                    l.points as usize,
                    l.count as usize,
                    l.positions.buffer.clone(),
                    l.infos.buffer.clone(),
                )
            })
            .collect();
        self.cards = lods
            .cards
            .iter()
            .map(|c| GpuMesh {
                vertices: c.vertices.buffer.clone(),
                indices: c.indices.buffer.clone(),
                count: c.index_count(),
            })
            .collect();
        self.card_quads = lods.cards.iter().map(|c| c.quads() as usize).collect();
        self.counts = lods.counts();
        let atlas = &lods.atlas;
        self.atlas = Self::atlas_texture(
            &self.device,
            &self.queue,
            atlas.width,
            atlas.height,
            &atlas.levels,
        );
        self.atlas_variants = atlas.variants;
        self.rebind_lighting();
        if let Some(cover) = &lods.cover {
            self.head_cover = Some(cover.buffer.clone());
        }
        let [lo, hi] = lods.bounds;
        self.bounds = [
            lo.min(Vec3::new(-0.12, -0.3, -0.13)).to_array(),
            hi.max(Vec3::new(0.12, 0.13, 0.13)).to_array(),
        ];
        self.center = lods.center.to_array();
        self.generation += 1;
    }

    /// A level's bind groups, around its buffers.
    fn level_groups(
        &self,
        points: usize,
        count: usize,
        positions: Arc<wgpu::Buffer>,
        infos: Arc<wgpu::Buffer>,
    ) -> GpuLevel {
        let group = |draw: &wgpu::Buffer| {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("hair strands"),
                layout: &self.layouts.strands,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: positions.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: infos.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: draw.as_entire_binding(),
                    },
                ],
            })
        };
        GpuLevel {
            points: points as u32,
            count: count as u32,
            groups: [
                group(&self.draws[MAIN]),
                group(&self.draws[SHADOW]),
                group(&self.draws[BLEND]),
            ],
            positions,
            _infos: infos,
        }
    }

    fn upload_level(&self, level: &StrandLevel, limit: usize) -> GpuLevel {
        let per_strand = level.points * 16;
        let count = level
            .len()
            .min(limit / per_strand.max(1))
            .max(1.min(level.len()));
        let positions = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("hair positions"),
                contents: bytemuck::cast_slice(&level.positions[..count * level.points]),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            });
        let infos = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("hair strand info"),
                contents: bytemuck::cast_slice(
                    &level.strands[..count.max(1).min(level.len().max(1))],
                ),
                usage: wgpu::BufferUsages::STORAGE,
            });
        self.level_groups(level.points, count, Arc::new(positions), Arc::new(infos))
    }

    fn upload_cards(&self, mesh: &CardMesh) -> GpuMesh {
        let vertices = if mesh.vertices.is_empty() {
            vec![CardVertex::default(); 3]
        } else {
            mesh.vertices.clone()
        };
        GpuMesh {
            vertices: Arc::new(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("hair cards"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            ),
            indices: Arc::new(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("hair card indices"),
                        contents: bytemuck::cast_slice(if mesh.indices.is_empty() {
                            &[0u32, 1, 2][..]
                        } else {
                            &mesh.indices
                        }),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            ),
            count: mesh.indices.len() as u32,
        }
    }

    /// Turn real-time physics on for the uploaded groom: every drawn strand
    /// will follow the simulated strands `binding` ties it to, whose groomed
    /// points (in the head's frame) are `guide_rest`. Each level keeps a
    /// copy of its rest pose to deform from.
    pub fn set_dynamics(&mut self, binding: &SimBinding, guide_rest: &[[f32; 4]]) {
        use wgpu::BufferUsages as U;
        let d = &*self.device;
        let init = |label: &str, bytes: &[u8], usage: U| {
            Arc::new(d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage,
            }))
        };
        let ids = init(
            "hair bind ids",
            bytemuck::cast_slice(&binding.guides),
            U::STORAGE,
        );
        let weights = init(
            "hair bind weights",
            bytemuck::cast_slice(&binding.weights),
            U::STORAGE,
        );
        let guide_rest_buffer = init(
            "hair guide rest",
            bytemuck::cast_slice(guide_rest),
            U::STORAGE,
        );
        let guide_now = init(
            "hair guide now",
            bytemuck::cast_slice(guide_rest),
            U::STORAGE | U::COPY_DST,
        );
        self.bind_dynamics(
            ids,
            weights,
            guide_rest_buffer,
            guide_now,
            binding.points as u32,
            None,
        );
    }

    /// Turn real-time physics on for a groom made on the GPU, simulated
    /// there ([`crate::gpu::GpuSim`]): the levels deform straight from the
    /// simulation's buffers, which never leave the GPU. Each frame's step is
    /// [`HairRenderer::set_moved`].
    pub fn set_gpu_dynamics(&mut self, sim: &crate::gpu::GpuSim) {
        let (ids, weights) = sim.binding();
        self.bind_dynamics(
            ids.buffer.clone(),
            weights.buffer.clone(),
            sim.rest().buffer.clone(),
            sim.positions().buffer.clone(),
            sim.points() as u32,
            sim.body()
                .map(|body| (body.buffer.clone(), sim.settings.collision_offset)),
        );
    }

    /// Each level's rest pose, and the pass deforming it from the simulated
    /// strands in `guide_now`.
    fn bind_dynamics(
        &mut self,
        ids: Arc<wgpu::Buffer>,
        weights: Arc<wgpu::Buffer>,
        guide_rest_buffer: Arc<wgpu::Buffer>,
        guide_now: Arc<wgpu::Buffer>,
        sim_points: u32,
        body: Option<(Arc<wgpu::Buffer>, f32)>,
    ) {
        use wgpu::BufferUsages as U;
        let (body_buffer, body) = match body {
            Some((buffer, offset)) => (buffer, Some(offset)),
            None => (
                Arc::new(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("hair no body"),
                    size: 17 * 16,
                    usage: U::STORAGE,
                    mapped_at_creation: false,
                })),
                None,
            ),
        };
        // Physics already on (a simulation moved to another level): the
        // levels are deformed, and their rest poses are the copies made
        // when it started.
        let mut kept: Vec<Option<wgpu::Buffer>> = match self.dynamics.take() {
            Some(old) => old.levels.into_iter().map(|l| Some(l.rest)).collect(),
            None => Vec::new(),
        };
        let d = &*self.device;
        let mut encoder = d.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hair rest poses"),
        });
        let levels = self
            .levels
            .iter()
            .enumerate()
            .map(|(at, level)| {
                let size = level.positions.size();
                let rest = match kept.get_mut(at).and_then(Option::take) {
                    Some(rest) => rest,
                    None => {
                        let rest = d.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("hair rest pose"),
                            size,
                            usage: U::STORAGE | U::COPY_DST | U::COPY_SRC,
                            mapped_at_creation: false,
                        });
                        encoder.copy_buffer_to_buffer(&level.positions, 0, &rest, 0, size);
                        rest
                    }
                };
                let params = d.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("hair deform params"),
                    size: std::mem::size_of::<DeformGpu>() as u64,
                    usage: U::UNIFORM | U::COPY_DST,
                    mapped_at_creation: false,
                });
                let entries: [&wgpu::Buffer; 8] = [
                    &rest,
                    &level.positions,
                    &guide_rest_buffer,
                    &guide_now,
                    &ids,
                    &weights,
                    &params,
                    &body_buffer,
                ];
                let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("hair deform"),
                    layout: &self.deform_layout,
                    entries: &entries
                        .iter()
                        .enumerate()
                        .map(|(k, buffer)| wgpu::BindGroupEntry {
                            binding: k as u32,
                            resource: buffer.as_entire_binding(),
                        })
                        .collect::<Vec<_>>(),
                });
                DeformLevel {
                    rest,
                    params,
                    group,
                    sim_points,
                }
            })
            .collect();
        self.queue.submit([encoder.finish()]);
        self.dynamics = Some(Dynamics {
            _ids: ids,
            _weights: weights,
            _guide_rest: guide_rest_buffer,
            guide_now,
            _body: body_buffer,
            body,
            levels,
            dirty: true,
        });
        self.generation += 1;
    }

    /// The simulated strands' points this frame (world space), and where
    /// the head is.
    pub fn set_simulated(&mut self, guides: &[[f32; 4]], head: &Rigid) {
        let Some(dynamics) = &self.dynamics else {
            return;
        };
        self.queue
            .write_buffer(&dynamics.guide_now, 0, bytemuck::cast_slice(guides));
        self.set_moved(head);
    }

    /// The simulated strands moved -- on the GPU, where the renderer reads
    /// them -- and the head is at `head`.
    pub fn set_moved(&mut self, head: &Rigid) {
        let Some(dynamics) = &mut self.dynamics else {
            return;
        };
        dynamics.dirty = true;
        self.head_matrix = head.matrix();
        self.moving = true;
    }

    /// Physics off: every level back to its rest pose, the head home.
    pub fn clear_dynamics(&mut self) {
        let Some(dynamics) = self.dynamics.take() else {
            return;
        };
        let mut encoder = self.device.create_command_encoder(&Default::default());
        for (level, deform) in self.levels.iter().zip(&dynamics.levels) {
            encoder.copy_buffer_to_buffer(
                &deform.rest,
                0,
                &level.positions,
                0,
                level.positions.size(),
            );
        }
        self.queue.submit([encoder.finish()]);
        self.head_matrix = IDENTITY;
        self.generation += 1;
    }

    /// Whether physics is on.
    pub fn has_dynamics(&self) -> bool {
        self.dynamics.is_some()
    }

    /// Deform `level`'s strands from the simulated ones.
    fn deform(&self, encoder: &mut wgpu::CommandEncoder, level: usize) {
        let (Some(dynamics), Some(gpu)) = (&self.dynamics, self.levels.get(level)) else {
            return;
        };
        let deform = &dynamics.levels[level];
        let params = DeformGpu {
            head: self.head_matrix,
            points: gpu.points,
            sim_points: deform.sim_points,
            strands: gpu.count,
            body: u32::from(dynamics.body.is_some()),
            offset: dynamics.body.unwrap_or(0.0),
            pad: [0; 3],
        };
        self.queue
            .write_buffer(&deform.params, 0, bytemuck::bytes_of(&params));
        let groups = (gpu.points * gpu.count).div_ceil(64);
        let x = groups.clamp(1, 65535);
        let y = groups.div_ceil(x);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("hair deform"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.deform_pipeline);
        pass.set_bind_group(0, &deform.group, &[]);
        pass.dispatch_workgroups(x, y, 1);
    }

    /// Upload the body the hair is on.
    pub fn set_head(&mut self, surface: &Surface) {
        let vertices: Vec<[f32; 6]> = surface
            .positions
            .iter()
            .zip(&surface.normals)
            .map(|(p, n)| [p.x, p.y, p.z, n.x, n.y, n.z])
            .collect();
        let indices: Vec<u32> = surface.triangles.iter().flatten().copied().collect();
        // Bare until a groom covers it.
        self.head_cover = Some(Arc::new(self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("hair head cover"),
                contents: bytemuck::cast_slice(&vec![[0.0f32; 2]; vertices.len().max(1)]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        )));
        self.head_surface = Some(HeadSurface {
            positions: surface.positions.clone(),
            normals: surface.normals.clone(),
            triangles: surface.triangles.clone(),
        });
        self.head =
            Some(GpuMesh {
                vertices: Arc::new(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("hair head"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                )),
                indices: Arc::new(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("hair head indices"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                )),
                count: indices.len() as u32,
            });
        self.generation += 1;
    }

    /// Shade the head under the groom's hair: how much of each vertex's
    /// skin the hair covers, smoothed over the mesh -- measured a vertex at
    /// a time it steps along the hairline.
    fn cover_head(&mut self, lods: &HairLods) {
        let Some(surface) = &self.head_surface else {
            return;
        };
        let HeadSurface {
            positions,
            normals,
            triangles,
        } = surface;
        let mut cover = lods.cover.at(positions, normals);
        let n = cover.len();
        let mut sum = vec![[0.0f32; 2]; n];
        let mut count = vec![0.0f32; n];
        for _ in 0..3 {
            sum.iter_mut().for_each(|s| *s = [0.0; 2]);
            count.iter_mut().for_each(|c| *c = 0.0);
            for t in triangles {
                for &a in t {
                    for &b in t {
                        let (a, b) = (a as usize, b as usize);
                        sum[a][0] += cover[b][0];
                        sum[a][1] += cover[b][1];
                        count[a] += 1.0;
                    }
                }
            }
            for i in 0..n {
                if count[i] > 0.0 {
                    cover[i] = [sum[i][0] / count[i], sum[i][1] / count[i]];
                }
            }
        }
        // A buffer of the renderer's own: a groom from the GPU may have left
        // its groomer's bound here.
        let buffer = Arc::new(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("hair head cover"),
                    contents: bytemuck::cast_slice(&cover),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }),
        );
        self.head_cover = Some(buffer);
    }

    /// The output texture at `width` by `height`; whether it was made anew
    /// (and so needs registering again wherever it is shown).
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
    ) -> (Arc<wgpu::Texture>, Arc<wgpu::TextureView>, bool) {
        let (width, height) = (width.max(1), height.max(1));
        if let Some(t) = &self.targets
            && t.width == width
            && t.height == height
        {
            return (t.output.clone(), t.output_view.clone(), false);
        }
        let texture =
            |label: &str, format: wgpu::TextureFormat, samples: u32, usage: wgpu::TextureUsages| {
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
        use wgpu::TextureUsages as U;
        let color = texture("hair color", HDR, SAMPLES, U::RENDER_ATTACHMENT)
            .create_view(&Default::default());
        let depth = texture("hair depth", DEPTH, SAMPLES, U::RENDER_ATTACHMENT)
            .create_view(&Default::default());
        let resolved = texture(
            "hair resolved",
            HDR,
            1,
            U::RENDER_ATTACHMENT | U::TEXTURE_BINDING,
        )
        .create_view(&Default::default());
        let accum = [0, 1].map(|_| {
            texture(
                "hair history",
                HDR,
                1,
                U::RENDER_ATTACHMENT | U::TEXTURE_BINDING,
            )
            .create_view(&Default::default())
        });
        let output = Arc::new(texture(
            "hair output",
            self.output_format,
            1,
            U::RENDER_ATTACHMENT | U::TEXTURE_BINDING | U::COPY_SRC,
        ));
        let output_view = Arc::new(output.create_view(&Default::default()));
        let post_group =
            |input: &wgpu::TextureView, history: &wgpu::TextureView, post: &wgpu::Buffer| {
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("hair post"),
                    layout: &self.layouts.post,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(history),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: post.as_entire_binding(),
                        },
                    ],
                })
            };
        let accumulate = [
            post_group(&resolved, &accum[0], &self.post[0]),
            post_group(&resolved, &accum[1], &self.post[0]),
        ];
        let tonemap_resolved = post_group(&resolved, &accum[0], &self.post[1]);
        let tonemap_accum = [
            post_group(&accum[0], &accum[1], &self.post[1]),
            post_group(&accum[1], &accum[0], &self.post[1]),
        ];
        self.targets = Some(Targets {
            width,
            height,
            color,
            depth,
            resolved,
            accum,
            output: output.clone(),
            output_view: output_view.clone(),
            accumulate,
            tonemap_resolved,
            tonemap_accum,
        });
        self.last = None;
        (output, output_view, true)
    }

    /// Whether another frame would change the image: the view moved, or the
    /// running mean has not yet taken all its frames.
    pub fn converged(&self, policy: &LodPolicy) -> bool {
        self.frames >= policy.quality.accumulate.max(1) && self.last_generation == self.generation
    }

    /// Draw a frame into the output texture.
    pub fn render(
        &mut self,
        camera: &Camera,
        lighting: &Lighting,
        material: &HairMaterial,
        policy: &LodPolicy,
    ) -> Result<FrameInfo> {
        let Some(targets) = &self.targets else {
            return Err(HairGpuError::MissingRenderTargets);
        };
        let (width, height) = (targets.width, targets.height);
        let aspect = width as f32 / height as f32;
        let quality = policy.quality;

        // How tall the head is on screen, and what to draw for it.
        let distance = math::length(math::sub(self.center, camera.eye)).max(1e-3);
        let pixel_angle = 2.0 * (camera.fov_y * 0.5).tan() / height as f32;
        let pixels = 0.25 / (distance * pixel_angle);
        let (representation, blend) = if self.levels.is_empty() {
            (Representation::Cards { level: 0 }, None)
        } else {
            policy.select_blended(pixels, self.counts)
        };
        let shadow_draw = policy.shadow(self.counts);

        // The key light's frustum: a box around everything, from the light.
        let [mut lo, mut hi] = self.bounds;
        // Moving hair may leave its rest bounds: room for it in the light's
        // frustum.
        if self.dynamics.is_some() {
            lo = math::sub(lo, [0.12; 3]);
            hi = math::add(hi, [0.12; 3]);
        }
        let center = math::scale(math::add(lo, hi), 0.5);
        let radius = math::length(math::sub(hi, lo)) * 0.5;
        let key = math::normalize(lighting.key.direction);
        let light_up = if key[1].abs() > 0.99 {
            [0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let light_eye = math::add(center, math::scale(key, radius * 2.0));
        let light_view_proj = multiply(
            &orthographic(-radius, radius, -radius, radius, radius, radius * 3.0),
            &look_at(light_eye, center, light_up),
        );
        let resolution = quality.shadow_resolution.max(16);
        if quality.shadows && self.shadows.resolution != resolution {
            self.shadows = Self::shadow_maps(&self.device, resolution);
            self.rebind_lighting();
        }
        let texel = 2.0 * radius / self.shadows.resolution as f32;
        let base = material.base_color();
        let darkness = 1.0 - (base[0] * 0.3 + base[1] * 0.59 + base[2] * 0.11).clamp(0.0, 1.0);
        let light = |l: &Light| {
            let d = math::normalize(l.direction);
            (
                [d[0], d[1], d[2], l.intensity],
                [l.color[0], l.color[1], l.color[2], 0.0],
            )
        };
        let (key_dir, key_color) = light(&lighting.key);
        let (fill_dir, fill_color) = light(&lighting.fill);
        let (rim_dir, rim_color) = light(&lighting.rim);
        let mut globals = Globals {
            view_proj: camera.view_projection(aspect),
            light_view_proj,
            head: self.head_matrix,
            camera: [camera.eye[0], camera.eye[1], camera.eye[2], pixel_angle],
            jitter: [0.0, 0.0, 0.0, lighting.exposure],
            viewport: [
                width as f32,
                height as f32,
                1.0 / width as f32,
                1.0 / height as f32,
            ],
            key_dir,
            key_color,
            fill_dir,
            fill_color,
            rim_dir,
            rim_color,
            sky_top: [
                lighting.sky_top[0],
                lighting.sky_top[1],
                lighting.sky_top[2],
                0.0,
            ],
            sky_bottom: [
                lighting.sky_bottom[0],
                lighting.sky_bottom[1],
                lighting.sky_bottom[2],
                0.0,
            ],
            base_color: [base[0], base[1], base[2], material.random_color],
            material: [
                material.roughness,
                material.radial_roughness,
                material.cuticle_tilt,
                material.random_roughness,
            ],
            taper: [material.taper.tip, material.taper.shape, 0.35, 0.45],
            dom: [0.003, 0.01, 0.03, 2.0 * radius],
            shadow: [
                texel,
                0.9,
                0.35 + 0.5 * darkness,
                if quality.shadows { 1.0 } else { 0.0 },
            ],
            mode: [
                if quality.marschner { 1.0 } else { 0.0 },
                self.atlas_variants as f32,
                material.tip_lightness,
                material.grey,
            ],
            // The key's softness: a 12 mm penumbra, in shadow-map UV.
            skin: [
                lighting.skin[0],
                lighting.skin[1],
                lighting.skin[2],
                0.012 / (2.0 * radius),
            ],
            roots: [
                material.root_darkness_start,
                material.root_darkness_end,
                material.root_darkness_strength,
                material.wave_strength,
            ],
            waves: [
                material.wave_scale,
                material.wave_phase.to_radians(),
                0.0,
                0.0,
            ],
        };

        // A new view starts the mean again; a still one takes the next
        // jittered frame, until it has all it wants.
        let wanted = quality.accumulate.max(1);
        // The head's pose moves with the simulation, which does not start
        // the mean again: the view is the same, the hair is moving.
        let mut view = globals;
        view.head = IDENTITY;
        let same = self.last_generation == self.generation
            && self
                .last
                .is_some_and(|(g, r, b)| g == view && r == representation && b == blend);
        if !same {
            self.frames = 0;
        }
        self.last = Some((view, representation, blend));
        self.last_generation = self.generation;
        let moving = std::mem::take(&mut self.moving);
        if self.frames >= wanted && !moving {
            return Ok(FrameInfo {
                representation,
                pixels,
                frames: self.frames,
                primitives: self.primitives(representation),
            });
        }
        if self.frames > 0 {
            let n = self.frames % 64 + 1;
            globals.jitter[0] = (halton(n, 2) - 0.5) * 2.0 / width as f32;
            globals.jitter[1] = (halton(n, 3) - 0.5) * 2.0 / height as f32;
            globals.jitter[2] = n as f32;
        }
        self.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
        let draw = |r: Representation, levels: &[GpuLevel]| -> Option<DrawGpu> {
            let Representation::Strands {
                level,
                count,
                width_scale,
            } = r
            else {
                return None;
            };
            let l = levels.get(level)?;
            Some(DrawGpu {
                points: l.points,
                count: (count as u32).min(l.count),
                pad: [0; 2],
                width_scale,
                min_pixels: quality.min_pixels,
                pad2: [0.0; 2],
            })
        };
        let main_draw = draw(representation, &self.levels);
        let shadow_gpu = draw(shadow_draw, &self.levels);
        if let Some(d) = main_draw {
            self.queue
                .write_buffer(&self.draws[MAIN], 0, bytemuck::bytes_of(&d));
        }
        // What the view draws: the band's form, and the coarser one fading
        // in -- strands handing over along the shared order (the first
        // strands at the finer level, the rest at the coarser, all as wide
        // as the finer count wants), cards dissolving in card by card.
        let strands_of = |r: Representation| match r {
            Representation::Strands { count, .. } => count as u32,
            Representation::Cards { .. } => 0,
        };
        let amount = blend.map_or(0.0, |b| b.amount);
        let handed = |count: u32| ((count as f32) * (1.0 - amount)).round() as u32;
        let mut strand_draws: Vec<(usize, usize, std::ops::Range<u32>)> = Vec::new();
        let mut card_draws: Vec<(usize, [f32; 2])> = Vec::new();
        match representation {
            Representation::Strands { level, .. } => {
                let count = strands_of(representation);
                strand_draws.push((level, MAIN, 0..handed(count)));
            }
            Representation::Cards { level } => card_draws.push((level, [amount, 2.0])),
        }
        if let Some(Blend { to, .. }) = blend {
            match (representation, to) {
                (Representation::Strands { .. }, Representation::Strands { level, .. }) => {
                    let count = strands_of(representation);
                    strand_draws.push((level, BLEND, handed(count)..count));
                }
                (Representation::Cards { .. }, Representation::Strands { level, .. }) => {
                    let count = strands_of(to);
                    strand_draws.push((level, BLEND, 0..count - handed(count)));
                }
                (_, Representation::Cards { level }) => card_draws.push((level, [-1.0, amount])),
            }
            if let (Representation::Strands { .. }, Some(d)) = (to, draw(to, &self.levels)) {
                // As wide as the finer form draws its strands.
                let width_scale = match representation {
                    Representation::Strands { width_scale, .. } => width_scale,
                    _ => d.width_scale,
                };
                let d = DrawGpu { width_scale, ..d };
                self.queue
                    .write_buffer(&self.draws[BLEND], 0, bytemuck::bytes_of(&d));
            }
        }
        for (i, (_, keep)) in card_draws.iter().enumerate() {
            self.queue.write_buffer(
                &self.card_draws[i],
                0,
                bytemuck::bytes_of(&CardDrawGpu {
                    keep: [keep[0], keep[1], 0.0, 0.0],
                }),
            );
        }
        if let Some(d) = shadow_gpu {
            self.queue
                .write_buffer(&self.draws[SHADOW], 0, bytemuck::bytes_of(&d));
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("hair frame"),
            });
        // Physics: move the levels this frame draws.
        if self.dynamics.as_ref().is_some_and(|d| d.dirty) {
            let mut levels = Vec::new();
            if let Representation::Strands { level, .. } = representation {
                levels.push(level);
            }
            if let (true, Representation::Strands { level, .. }) = (quality.shadows, shadow_draw)
                && !levels.contains(&level)
            {
                levels.push(level);
            }
            for level in levels {
                self.deform(&mut encoder, level);
            }
            if let Some(d) = &mut self.dynamics {
                d.dirty = false;
            }
        }
        let targets = self.targets.as_ref().expect("checked above");
        if quality.shadows {
            // The head's shadow.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("hair head shadow"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.shadows.head,
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
                if let Some(head) = &self.head {
                    pass.set_pipeline(&self.pipelines.head_depth);
                    pass.set_bind_group(0, &self.globals_group, &[]);
                    pass.set_vertex_buffer(0, head.vertices.slice(..));
                    pass.set_index_buffer(head.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..head.count, 0, 0..1);
                }
            }
            // The hair's nearest depth.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("hair shadow depth"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.shadows.depth,
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
                if let (Some(d), Representation::Strands { level, .. }) = (shadow_gpu, shadow_draw)
                {
                    pass.set_pipeline(&self.pipelines.strand_depth);
                    pass.set_bind_group(0, &self.globals_group, &[]);
                    pass.set_bind_group(1, &self.levels[level].groups[SHADOW], &[]);
                    pass.draw(0..d.points * 2, 0..d.count);
                }
            }
            // Its opacity, layer by layer behind that.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("hair shadow opacity"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.shadows.layers,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                if let (Some(d), Representation::Strands { level, .. }) = (shadow_gpu, shadow_draw)
                {
                    pass.set_pipeline(&self.pipelines.strand_opacity);
                    pass.set_bind_group(0, &self.globals_group, &[]);
                    pass.set_bind_group(1, &self.levels[level].groups[SHADOW], &[]);
                    pass.set_bind_group(2, &self.opacity_group, &[]);
                    pass.draw(0..d.points * 2, 0..d.count);
                }
            }
        }

        // The view.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hair view"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.color,
                    depth_slice: None,
                    resolve_target: Some(&targets.resolved),
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
            pass.set_bind_group(0, &self.globals_group, &[]);
            pass.set_pipeline(&self.pipelines.backdrop);
            pass.draw(0..3, 0..1);
            pass.set_bind_group(1, &self.empty_group, &[]);
            pass.set_bind_group(2, &self.lighting_group, &[]);
            if let (Some(head), Some(cover)) = (&self.head, &self.head_cover) {
                pass.set_pipeline(&self.pipelines.head);
                pass.set_vertex_buffer(0, head.vertices.slice(..));
                pass.set_vertex_buffer(1, cover.slice(..));
                pass.set_index_buffer(head.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..head.count, 0, 0..1);
            }
            for (level, slot, instances) in &strand_draws {
                let Some(l) = self.levels.get(*level) else {
                    continue;
                };
                let instances = instances.start.min(l.count)..instances.end.min(l.count);
                if instances.is_empty() {
                    continue;
                }
                pass.set_pipeline(&self.pipelines.strands);
                pass.set_bind_group(1, &l.groups[*slot], &[]);
                pass.draw(0..l.points * 2, instances);
            }
            for (i, (level, _)) in card_draws.iter().enumerate() {
                if let Some(cards) = self.cards.get(*level)
                    && cards.count > 0
                {
                    pass.set_pipeline(&self.pipelines.cards);
                    pass.set_bind_group(1, &self.card_groups[i], &[]);
                    pass.set_vertex_buffer(0, cards.vertices.slice(..));
                    pass.set_index_buffer(cards.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..cards.count, 0, 0..1);
                }
            }
        }

        // Into the running mean, then onto the screen.
        let accumulating = wanted > 1 || moving;
        let source = if accumulating {
            let from = self.history;
            let to = 1 - from;
            // Moving hair keeps a share of each new frame: an exponential
            // mean, which smooths the stochastic coverage without trailing
            // far behind the motion.
            let mut weight = 1.0 / (self.frames + 1) as f32;
            if moving {
                weight = weight.max(0.35);
            }
            self.queue.write_buffer(
                &self.post[0],
                0,
                bytemuck::bytes_of(&PostGpu {
                    params: [weight, 0.0, 0.0, 0.0],
                }),
            );
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("hair accumulate"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &targets.accum[to],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&self.pipelines.accumulate);
                pass.set_bind_group(0, &targets.accumulate[from], &[]);
                pass.draw(0..3, 0..1);
            }
            self.history = to;
            &targets.tonemap_accum[to]
        } else {
            &targets.tonemap_resolved
        };
        self.queue.write_buffer(
            &self.post[1],
            0,
            bytemuck::bytes_of(&PostGpu {
                params: [
                    0.0,
                    lighting.exposure,
                    if self.output_format.is_srgb() {
                        0.0
                    } else {
                        1.0
                    },
                    0.0,
                ],
            }),
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hair tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines.tonemap);
            pass.set_bind_group(0, source, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.frames += 1;
        Ok(FrameInfo {
            representation,
            pixels,
            frames: self.frames,
            primitives: self.primitives(representation),
        })
    }

    fn primitives(&self, representation: Representation) -> usize {
        match representation {
            Representation::Strands { level, count, .. } => {
                count.min(self.levels.get(level).map_or(0, |l| l.count as usize))
            }
            Representation::Cards { level } => self.card_quads.get(level).copied().unwrap_or(0),
        }
    }

    /// Render until the running mean is complete: for stills.
    pub fn render_converged(
        &mut self,
        camera: &Camera,
        lighting: &Lighting,
        material: &HairMaterial,
        policy: &LodPolicy,
    ) -> Result<FrameInfo> {
        let mut info = self.render(camera, lighting, material, policy)?;
        while info.frames < policy.quality.accumulate.max(1) {
            info = self.render(camera, lighting, material, policy)?;
        }
        Ok(info)
    }

    /// The output texture's pixels, tightly packed rows of four bytes each.
    pub fn read_rgba(&self) -> Result<(u32, u32, Vec<u8>)> {
        let Some(targets) = &self.targets else {
            return Err(HairGpuError::NothingRendered);
        };
        let (width, height) = (targets.width, targets.height);
        let row = width * 4;
        let padded = row.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hair readback"),
            size: (padded * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            targets.output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::wait_indefinitely())?;
        rx.recv()??;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut pixels = Vec::with_capacity((row * height) as usize);
        for y in 0..height {
            let start = (y * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + row as usize]);
        }
        // Blue-first formats read back swapped.
        if matches!(
            self.output_format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        ) {
            for p in pixels.chunks_mut(4) {
                p.swap(0, 2);
            }
        }
        Ok((width, height, pixels))
    }
}

/// A device and queue of their own, for drawing without a window: the
/// dataset's renders, snapshots and tests. Asks for the adapter's own
/// storage limits, so a full groom fits in one binding.
pub fn headless_device() -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(HairGpuError::Adapter)?;
        let adapter_limits = adapter.limits();
        let limits = wgpu::Limits {
            max_buffer_size: adapter_limits.max_buffer_size,
            max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
            ..Default::default()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("hair"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::Performance,
                experimental_features: wgpu::ExperimentalFeatures::default(),
                trace: Default::default(),
            })
            .await?;
        Ok((Arc::new(device), Arc::new(queue)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(source: &str) {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{e:?}"));
    }

    #[test]
    fn the_shaders_validate() {
        validate(HAIR_WGSL);
        validate(POST_WGSL);
        validate(&deform_source());
    }

    #[test]
    fn uniforms_match_their_wgsl_sizes() {
        assert_eq!(std::mem::size_of::<Globals>(), 3 * 64 + 20 * 16);
        assert_eq!(std::mem::size_of::<CardDrawGpu>(), 16);
        assert_eq!(std::mem::size_of::<DeformGpu>(), 96);
        assert_eq!(std::mem::size_of::<DrawGpu>(), 32);
        assert_eq!(std::mem::size_of::<CardVertex>(), 64);
    }
}
