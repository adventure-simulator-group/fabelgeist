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

pub use frame_state::{RenderInputError, RenderInputPart};
pub use head::RenderSourceError;
pub use levels::{LevelBufferRole, RenderLevelError};
pub use readback::RenderReadbackError;

use atlas::UploadedAtlas;
use deformation::{DeformationPipeline, DeformationState, DeformationViews};
use draws::{DrawPlan, DrawRole};
use frame_state::{History, RenderedView};
use frame_uniforms::FrameProjection;
use head::HeadDraw;
use levels::GpuLevel;
use std::sync::Arc;

use crate::gpu::error::{HairGpuError, Result};
use fabelgeist_math::Vec3;
use wgpu::util::DeviceExt;

pub use math::{Mat4, halton, look_at, multiply, orthographic, perspective};

use crate::cards::{CardAtlas, CardMesh, CardVertex};
use crate::lod::{
    AccumulationFrames, HairLods, HairShading, LodCounts, LodPolicy, PixelSpan, Representation,
    Selection, ShadowMapSize, StrandLevel, StrandLevelIndex,
};
use crate::material::HairMaterial;
use crate::sim::{Rigid, SimBinding};

mod atlas;
mod deformation;
mod draws;
mod frame_state;
mod frame_uniforms;
mod head;
mod levels;
mod math;
mod readback;

const SAMPLES: u32 = 4;
const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The WGSL of the scene passes and of the full-screen passes after them.
pub const HAIR_WGSL: &str = include_str!("hair.wgsl");
pub const POST_WGSL: &str = include_str!("post.wgsl");
pub const DEFORM_WGSL: &str = include_str!("deform.wgsl");

/// Where the view is from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub fov_y: f32,
}

/// A directional light: towards it, its colour and strength.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
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

/// What a frame drew.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameInfo {
    pub representation: Representation,
    /// The head's height on screen.
    pub pixels: PixelSpan,
    /// Frames in the running mean.
    pub frames: AccumulationFrames,
    /// Population of the primary representation, excluding transition and
    /// shadow draws. Card populations currently count quads.
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

/// Which cards a card draw keeps: those whose own random number is in
/// `keep.x .. keep.y`, so two cuts of cards dissolve into each other.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CardDrawGpu {
    keep: [f32; 4],
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
}

struct DeformLevel {
    rest: wgpu::Buffer,
    params: wgpu::Buffer,
    group: wgpu::BindGroup,
    sim_points: u32,
    state: DeformationState,
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

#[derive(Clone)]
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
    resolution: ShadowMapSize,
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
    deformation: DeformationPipeline,
    cards: Vec<GpuMesh>,
    card_quads: Vec<usize>,
    /// The sizes of the groom's levels, to choose among them.
    counts: LodCounts,
    atlas: UploadedAtlas,
    head: Option<HeadDraw>,
    shadows: ShadowMaps,
    lighting_group: wgpu::BindGroup,
    opacity_group: wgpu::BindGroup,
    targets: Option<Targets>,
    /// The bounds everything drawn fits in, for the light's frustum.
    bounds: [[f32; 3]; 2],
    center: [f32; 3],
    /// The accumulation: frames in the mean, which history holds it, and
    /// what the view was.
    frames: AccumulationFrames,
    history: History,
    last: Option<RenderedView>,
    /// The simulated strands moved since the last frame: keep blending
    /// frames (a temporal filter) rather than starting the mean again.
    moving: bool,
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

impl HairRenderer {
    /// Build pipelines on `device` and its paired `queue`.
    ///
    /// Supplied native resources must belong to this same logical device.
    /// Host atlas allocation and shader admission failures are returned. Native
    /// pipeline, resource and queue failures follow the device's configured
    /// error handling; this constructor does not replace it.
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
        let deformation = DeformationPipeline::new(d)?;
        let atlas = UploadedAtlas::new(d, &queue, &CardAtlas::transparent()?)?;
        let shadows = Self::shadow_maps(d, ShadowMapSize::from_texels(16));
        let (lighting_group, opacity_group) =
            Self::lighting_groups(d, &layouts, &shadows, &atlas.view, &linear, &comparison);
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
            deformation,
            cards: Vec::new(),
            card_quads: Vec::new(),
            counts: LodCounts::default(),
            atlas,
            head: None,
            shadows,
            lighting_group,
            opacity_group,
            targets: None,
            bounds: [[-0.2, -0.3, -0.2], [0.2, 0.2, 0.2]],
            center: [0.0; 3],
            frames: AccumulationFrames::default(),
            history: History::Empty,
            last: None,
            moving: false,
        })
    }

    fn shadow_maps(device: &wgpu::Device, resolution: ShadowMapSize) -> ShadowMaps {
        let texture = |label: &str, format: wgpu::TextureFormat| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: resolution.texels(),
                        height: resolution.texels(),
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
            &self.atlas.view,
            &self.linear,
            &self.comparison,
        );
        self.lighting_group = lighting;
        self.opacity_group = opacity;
    }

    /// Upload a groom: every strand level, the cards and their atlas. A
    /// level larger than one storage binding may hold keeps the prefix of
    /// its strands that fits -- an even sample of the rest.
    pub fn set_groom(&mut self, lods: &HairLods) -> Result<()> {
        let atlas = UploadedAtlas::new(&self.device, &self.queue, &lods.atlas)?;
        let levels = lods
            .levels
            .iter()
            .map(|level| self.upload_level(level))
            .collect::<Result<Vec<_>>>()?;
        let cards = lods
            .cards
            .iter()
            .map(|cards| self.upload_cards(cards))
            .collect();
        let card_quads = lods
            .cards
            .iter()
            .map(|cards| cards.indices.len() / 6)
            .collect();
        let counts = lods.counts();
        let head = self
            .head
            .as_ref()
            .map(|head| head.with_cpu_cover(&self.device, lods));
        let (lighting_group, opacity_group) = Self::lighting_groups(
            &self.device,
            &self.layouts,
            &self.shadows,
            &atlas.view,
            &self.linear,
            &self.comparison,
        );
        let [lo, hi] = lods.bounds;
        // The head is in the light's frustum too, however short the hair.
        let bounds = [
            lo.min(Vec3::new(-0.12, -0.3, -0.13)).to_array(),
            hi.max(Vec3::new(0.12, 0.13, 0.13)).to_array(),
        ];
        self.dynamics = None;
        self.head_matrix = IDENTITY;
        self.levels = levels;
        self.cards = cards;
        self.card_quads = card_quads;
        self.counts = counts;
        self.atlas = atlas;
        self.lighting_group = lighting_group;
        self.opacity_group = opacity_group;
        self.head = head;
        self.bounds = bounds;
        self.center = lods.center.to_array();
        self.invalidate_view();
        Ok(())
    }

    /// Upload a groom made on the GPU ([`crate::gpu::GpuGroomer::lods`]):
    /// The levels, cards and head coverage reuse the groomer's geometry
    /// buffers. The host card-atlas mip chain is uploaded to a texture.
    ///
    /// The groom must belong to this renderer's native device. Coverage
    /// requires its exact shared head capture to be installed first, with
    /// matching device and byte extents. Absent coverage replaces a present
    /// head's old coverage with bare skin. Failed admission preserves the
    /// previous scene; native construction follows the device's error policy.
    pub fn set_gpu_groom(&mut self, lods: &crate::gpu::GpuLods) -> Result<()> {
        if lods.device() != self.device.as_ref() {
            return Err(RenderSourceError::ForeignGroomDevice.into());
        }
        let head = match (&self.head, &lods.cover) {
            (Some(head), cover) => Some(head.with_gpu_cover(&self.device, cover.as_ref())?),
            (None, Some(_)) => return Err(RenderSourceError::MissingHead.into()),
            (None, None) => None,
        };
        let levels = lods
            .levels
            .iter()
            .map(|level| self.upload_gpu_level(level))
            .collect::<Result<Vec<_>>>()?;
        let cards = lods
            .cards
            .iter()
            .map(|cards| GpuMesh {
                vertices: cards.vertices.buffer.clone(),
                indices: cards.indices.buffer.clone(),
                count: cards.index_count(),
            })
            .collect();
        let card_quads = lods
            .cards
            .iter()
            .map(|cards| cards.quads() as usize)
            .collect();
        let counts = lods.counts();
        let atlas = UploadedAtlas::new(&self.device, &self.queue, &lods.atlas)?;
        let (lighting_group, opacity_group) = Self::lighting_groups(
            &self.device,
            &self.layouts,
            &self.shadows,
            &atlas.view,
            &self.linear,
            &self.comparison,
        );
        let [lo, hi] = lods.bounds;
        let bounds = [
            lo.min(Vec3::new(-0.12, -0.3, -0.13)).to_array(),
            hi.max(Vec3::new(0.12, 0.13, 0.13)).to_array(),
        ];
        // Fallible source admission and native candidate construction precede
        // this publication; provider-handled errors retain that native policy.
        self.dynamics = None;
        self.head_matrix = IDENTITY;
        self.levels = levels;
        self.cards = cards;
        self.card_quads = card_quads;
        self.counts = counts;
        self.atlas = atlas;
        self.lighting_group = lighting_group;
        self.opacity_group = opacity_group;
        self.head = head;
        self.bounds = bounds;
        self.center = lods.center.to_array();
        self.invalidate_view();
        Ok(())
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
                    layout: &self.deformation.layout,
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
                    state: DeformationState::Pending,
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
        });
        self.invalidate_view();
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
        for level in &mut dynamics.levels {
            level.state = DeformationState::Pending;
        }
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
        self.invalidate_view();
    }

    /// Whether physics is on.
    pub fn has_dynamics(&self) -> bool {
        self.dynamics.is_some()
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
        self.invalidate_view();
        (output, output_view, true)
    }

    /// Forget the mean when its targets or scene inputs are replaced.
    fn invalidate_view(&mut self) {
        self.last = None;
        self.frames = AccumulationFrames::default();
        self.history = History::Empty;
    }

    /// Whether the last rendered view has its requested running mean, with
    /// no scene replacement or simulation movement waiting to be drawn.
    pub fn converged(&self, policy: &LodPolicy) -> bool {
        self.last.is_some_and(|view| {
            policy
                .select_blended(view.pixels, self.counts)
                .is_ok_and(|selection| {
                    view.representation == selection.representation
                        && view.blend == selection.blend
                        && view.min_pixels == policy.quality.min_pixels
                        && view.shading == policy.quality.shading
                        && view.shadow
                            == policy
                                .quality
                                .shadows
                                .enabled()
                                .then(|| policy.shadow(self.counts))
                        && view.shadow_resolution == policy.quality.shadows.size()
                })
        }) && !self.moving
            && self.frames >= policy.quality.accumulate.target()
    }

    /// Draw a frame into the output texture.
    pub fn render(
        &mut self,
        camera: &Camera,
        lighting: &Lighting,
        material: &HairMaterial,
        policy: &LodPolicy,
    ) -> Result<FrameInfo> {
        let (width, height) = self
            .targets
            .as_ref()
            .map(|targets| (targets.width, targets.height))
            .ok_or(HairGpuError::MissingRenderTargets)?;
        let quality = policy.quality;
        let projection = FrameProjection::new(
            self,
            camera,
            lighting,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let pixels = projection.pixels;
        let Selection {
            representation,
            blend,
        } = policy.select_blended(pixels, self.counts)?;
        let shadow_draw = policy.shadow(self.counts);
        let resolution = quality.shadows.size();
        let mut globals = Globals::for_frame(&projection, self, lighting, material, quality)?;

        // A new view starts the mean again; a still one takes the next
        // jittered frame, until it has all it wants.
        let wanted = quality.accumulate.target();
        // The head's pose moves with the simulation, which does not start
        // the mean again: the view is the same, the hair is moving.
        let mut view = globals;
        view.head = IDENTITY;
        let view = RenderedView {
            globals: view,
            representation,
            blend,
            shadow: quality.shadows.enabled().then_some(shadow_draw),
            min_pixels: quality.min_pixels,
            pixels,
            shading: quality.shading,
            shadow_resolution: resolution,
        }
        .validate()?;
        if let Some(resolution) = resolution
            && self.shadows.resolution != resolution
        {
            self.shadows = Self::shadow_maps(&self.device, resolution);
            self.rebind_lighting();
        }
        let mut frame = self.prepare_frame(view, wanted);
        let accumulating = frame.accumulating(wanted);
        let moving = frame.moving;
        if frame.is_complete(wanted) {
            return Ok(FrameInfo {
                representation,
                pixels,
                frames: frame.frames,
                primitives: self.primitives(representation),
            });
        }
        if frame.frames.count() > 0 {
            let n = frame.frames.count() % 64 + 1;
            globals.jitter[0] = (halton(n, 2) - 0.5) * 2.0 / width as f32;
            globals.jitter[1] = (halton(n, 3) - 0.5) * 2.0 / height as f32;
            globals.jitter[2] = n as f32;
        }
        self.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
        let main_draw = self.draw_gpu(representation, quality.min_pixels);
        let shadow_gpu = self.draw_gpu(shadow_draw, quality.min_pixels);
        if let Some(draw) = main_draw {
            self.queue.write_buffer(
                &self.draws[DrawRole::Main.index()],
                0,
                bytemuck::bytes_of(&draw),
            );
        }
        let plan = DrawPlan::new(Selection {
            representation,
            blend,
        });
        if let Some(blend) = blend
            && let Some(draw) = self.draw_gpu(blend.to, quality.min_pixels)
        {
            // The incoming strands keep the finer form's apparent width.
            let width_scale = match representation {
                Representation::Strands { width_scale, .. } => width_scale.ratio(),
                _ => draw.width_scale,
            };
            self.queue.write_buffer(
                &self.draws[DrawRole::Blend.index()],
                0,
                bytemuck::bytes_of(&DrawGpu {
                    width_scale,
                    ..draw
                }),
            );
        }
        for (slot, draw) in plan.cards.iter().enumerate() {
            self.queue.write_buffer(
                &self.card_draws[slot],
                0,
                bytemuck::bytes_of(&CardDrawGpu {
                    keep: draw.coverage.keep(),
                }),
            );
        }
        if let Some(d) = shadow_gpu {
            self.queue.write_buffer(
                &self.draws[DrawRole::Shadow.index()],
                0,
                bytemuck::bytes_of(&d),
            );
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("hair frame"),
            });
        let deformations = self.record_deformations(
            &mut encoder,
            DeformationViews {
                main: representation,
                shadow: quality.shadows.enabled().then_some(shadow_draw),
                blend: blend.map(|blend| blend.to),
            },
        )?;
        let targets = self
            .targets
            .as_ref()
            .ok_or(HairGpuError::MissingRenderTargets)?;
        if quality.shadows.enabled() {
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
                    pass.set_vertex_buffer(0, head.mesh.vertices.slice(..));
                    pass.set_index_buffer(head.mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..head.mesh.count, 0, 0..1);
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
                    pass.set_bind_group(
                        1,
                        &self.levels[level.index()].groups[DrawRole::Shadow.index()],
                        &[],
                    );
                    pass.draw(
                        0..self.levels[level.index()].layout.native_draw_vertices(),
                        0..d.count,
                    );
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
                    pass.set_bind_group(
                        1,
                        &self.levels[level.index()].groups[DrawRole::Shadow.index()],
                        &[],
                    );
                    pass.set_bind_group(2, &self.opacity_group, &[]);
                    pass.draw(
                        0..self.levels[level.index()].layout.native_draw_vertices(),
                        0..d.count,
                    );
                }
            }
        }

        // The view.
        let targets = self
            .targets
            .as_ref()
            .ok_or(HairGpuError::MissingRenderTargets)?;
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
            if let Some(head) = &self.head {
                pass.set_pipeline(&self.pipelines.head);
                pass.set_vertex_buffer(0, head.mesh.vertices.slice(..));
                pass.set_vertex_buffer(1, head.cover.slice(..));
                pass.set_index_buffer(head.mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..head.mesh.count, 0, 0..1);
            }
            for draw in &plan.strands {
                let Some(l) = self.levels.get(draw.level.index()) else {
                    continue;
                };
                let instances = draw.instances.start.min(l.layout.native_strands())
                    ..draw.instances.end.min(l.layout.native_strands());
                if instances.is_empty() {
                    continue;
                }
                pass.set_pipeline(&self.pipelines.strands);
                pass.set_bind_group(1, &l.groups[draw.role.index()], &[]);
                pass.draw(0..l.layout.native_draw_vertices(), instances);
            }
            for (i, draw) in plan.cards.iter().enumerate() {
                if let Some(cards) = self.cards.get(draw.level.index())
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
        let source = if accumulating {
            let from = frame.history.previous();
            let to = from.next();
            // Moving hair keeps a share of each new frame: an exponential
            // mean, which smooths the stochastic coverage without trailing
            // far behind the motion.
            let mut weight = frame.frames.next_weight();
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
                        view: &targets.accum[to.index()],
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
                pass.set_bind_group(0, &targets.accumulate[from.index()], &[]);
                pass.draw(0..3, 0..1);
            }
            frame.history = History::Current(to);
            &targets.tonemap_accum[to.index()]
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
        // These states describe submitted commands, not completed GPU work.
        self.publish_deformations(deformations);
        self.publish_frame(frame);
        Ok(FrameInfo {
            representation,
            pixels,
            frames: self.frames,
            primitives: self.primitives(representation),
        })
    }

    fn primitives(&self, representation: Representation) -> usize {
        match representation {
            Representation::Strands { level, count, .. } => count.count().min(
                self.levels
                    .get(level.index())
                    .map_or(0, |level| level.layout.strands().count()),
            ),
            Representation::Cards { level } => {
                self.card_quads.get(level.index()).copied().unwrap_or(0)
            }
            Representation::Empty => 0,
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
        while info.frames < policy.quality.accumulate.target() {
            info = self.render(camera, lighting, material, policy)?;
        }
        Ok(info)
    }
}

/// A device and queue of their own, for drawing without a window: the
/// dataset's renders, snapshots and tests. Requests the adapter's buffer
/// and storage-binding limits; individual inputs must still fit them.
/// Native errors use the provider's default error handling.
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

/// The deform pass's source: [`DEFORM_WGSL`] and the head's distance field
/// it keeps the drawn strands off.
fn deform_source() -> String {
    format!(
        "{DEFORM_WGSL}{}{}",
        crate::gpu::wgsl::common::MATH,
        crate::gpu::wgsl::common::HEAD
    )
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
