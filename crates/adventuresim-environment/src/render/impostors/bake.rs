//! Baking impostors on the render thread: each tree drawn from every view
//! into its layer of the atlas, multisampled and resolved, then its mip
//! levels made from the resolved one. A bake runs a few trees a frame, so
//! a large one never holds a frame for long.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
    BindingType, Buffer, BufferBinding, BufferBindingType, BufferInitDescriptor, BufferSize,
    BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompareFunction, DepthStencilState, Extent3d, FragmentState,
    LoadOp, MultisampleState, Operations, PipelineCache, PrimitiveState,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, ShaderStages, StoreOp, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexState,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;

/// The atlas's formats: albedo and coverage; normal and occlusion.
pub(super) const ALBEDO_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
pub(super) const NORMAL_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
/// Samples a texel while baking: the coverage resolves to five steps.
const SAMPLES: u32 = 4;
/// Trees baked a frame.
const TREES_A_FRAME: usize = 2;
/// A uniform's dynamic offsets step by at least this many bytes.
const MODEL_STRIDE: u64 = 256;

const BAKE_SHADER: &str = "embedded://adventuresim_environment/render/shaders/impostor_bake.wgsl";
const MIP_SHADER: &str = "embedded://adventuresim_environment/render/shaders/impostor_mip.wgsl";

/// One tree to bake, as the bake shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct BakeModel {
    pub center_radius: [f32; 4],
    /// First branch record and count; first leaf record and count.
    pub records: [u32; 4],
    /// Sides around a tube, spans along it, blades a cluster, views a side.
    pub shape: [u32; 4],
    /// A texel's width, metres, the smallest blade drawn, texels, and how
    /// much larger every blade is drawn.
    pub texel: [f32; 4],
}

/// A bake handed to the render thread: the atlas to fill, the trees and
/// their records.
pub(super) struct BakeJob {
    pub albedo: AssetId<Image>,
    pub normal: AssetId<Image>,
    /// Views along each side of a layer's grid, and a view's side, texels.
    pub frames: u32,
    pub frame_texels: u32,
    pub mips: u32,
    pub models: Vec<BakeModel>,
    pub branches: Vec<u8>,
    pub leaves: Vec<u8>,
    pub species: Vec<u8>,
    pub plants: Vec<u8>,
    /// Trees baked so far, which the main world watches.
    pub progress: Arc<AtomicU32>,
}

/// Bakes waiting for the render thread, shared by both worlds.
#[derive(Resource, Clone, Default)]
pub(super) struct BakeQueue(pub Arc<Mutex<Vec<BakeJob>>>);

impl BakeQueue {
    pub(super) fn push(&self, jobs: Vec<BakeJob>) {
        let mut queue = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        // Only the latest bake matters.
        queue.clear();
        queue.extend(jobs.into_iter().rev());
    }

    fn take(&self) -> Option<BakeJob> {
        let mut queue = self.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.pop()
    }
}

/// The bake's pipelines and its multisampled targets.
#[derive(Resource)]
pub(super) struct Baker {
    layout: BindGroupLayoutDescriptor,
    mip_layout: BindGroupLayoutDescriptor,
    branches: CachedRenderPipelineId,
    leaves: CachedRenderPipelineId,
    mips: CachedRenderPipelineId,
    targets: Option<Targets>,
    active: Option<Active>,
}

/// Multisampled targets a layer is drawn into, of one size.
struct Targets {
    size: u32,
    albedo: TextureView,
    normal: TextureView,
    depth: TextureView,
}

/// A bake under way: its job, its buffers, and the next tree.
struct Active {
    job: BakeJob,
    bound: Option<(BindGroup, [Buffer; 5])>,
    next: usize,
}

pub(super) fn init(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    let storage = |binding| BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let layout = BindGroupLayoutDescriptor::new(
        "impostor_bake_layout",
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(size_of::<BakeModel>() as u64),
                },
                count: None,
            },
            storage(1),
            storage(2),
            storage(3),
            storage(4),
        ],
    );
    let texture = |binding| BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let mip_layout =
        BindGroupLayoutDescriptor::new("impostor_mip_layout", &[texture(0), texture(1)]);
    let bake_shader = asset_server.load(BAKE_SHADER);
    let mip_shader = asset_server.load(MIP_SHADER);
    let targets = vec![
        Some(ColorTargetState {
            format: ALBEDO_FORMAT,
            blend: None,
            write_mask: ColorWrites::ALL,
        }),
        Some(ColorTargetState {
            format: NORMAL_FORMAT,
            blend: None,
            write_mask: ColorWrites::ALL,
        }),
    ];
    let drawn = |label: &'static str, vertex: &'static str, fragment: &'static str| {
        RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![layout.clone()],
            vertex: VertexState {
                shader: bake_shader.clone(),
                entry_point: Some(vertex.into()),
                ..default()
            },
            fragment: Some(FragmentState {
                shader: bake_shader.clone(),
                entry_point: Some(fragment.into()),
                targets: targets.clone(),
                ..default()
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: SAMPLES,
                ..default()
            },
            ..default()
        }
    };
    let branches = pipeline_cache.queue_render_pipeline(drawn(
        "impostor_bake_branches",
        "branch_vertex_main",
        "branch_fragment",
    ));
    let leaves = pipeline_cache.queue_render_pipeline(drawn(
        "impostor_bake_leaves",
        "leaf_vertex_main",
        "leaf_fragment",
    ));
    let mips = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("impostor_mips".into()),
        layout: vec![mip_layout.clone()],
        vertex: VertexState {
            shader: mip_shader.clone(),
            entry_point: Some("mip_vertex".into()),
            ..default()
        },
        fragment: Some(FragmentState {
            shader: mip_shader,
            entry_point: Some("mip_fragment".into()),
            targets,
            ..default()
        }),
        ..default()
    });
    commands.insert_resource(Baker {
        layout,
        mip_layout,
        branches,
        leaves,
        mips,
        targets: None,
        active: None,
    });
}

/// Bake the next few trees of the bake under way, once its atlas and the
/// pipelines are ready.
pub(super) fn bake(
    mut baker: ResMut<Baker>,
    queue: Res<BakeQueue>,
    images: Res<RenderAssets<GpuImage>>,
    pipeline_cache: Res<PipelineCache>,
    device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let baker = &mut *baker;
    if let Some(job) = queue.take() {
        baker.active = Some(Active {
            job,
            bound: None,
            next: 0,
        });
    }
    let Some(active) = baker.active.as_mut() else {
        return;
    };
    let (Some(albedo), Some(normal)) = (
        images.get(active.job.albedo),
        images.get(active.job.normal),
    ) else {
        return;
    };
    let (Some(branch_pipeline), Some(leaf_pipeline), Some(mip_pipeline)) = (
        pipeline_cache.get_render_pipeline(baker.branches),
        pipeline_cache.get_render_pipeline(baker.leaves),
        pipeline_cache.get_render_pipeline(baker.mips),
    ) else {
        return;
    };
    let job = &active.job;
    let size = job.frames * job.frame_texels;
    if baker.targets.as_ref().is_none_or(|targets| targets.size != size) {
        baker.targets = Some(Targets::new(&device, size));
    }
    let Some(targets) = baker.targets.as_ref() else {
        return;
    };
    let (bind_group, _) = active.bound.get_or_insert_with(|| {
        let mut models = vec![0u8; job.models.len().max(1) * MODEL_STRIDE as usize];
        for (index, model) in job.models.iter().enumerate() {
            let at = index * MODEL_STRIDE as usize;
            models[at..at + size_of::<BakeModel>()].copy_from_slice(bytemuck::bytes_of(model));
        }
        let buffer = |label: &'static str, bytes: &[u8], usage: BufferUsages| {
            let mut contents = bytes.to_vec();
            contents.resize(contents.len().max(256), 0);
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some(label),
                contents: &contents,
                usage,
            })
        };
        let buffers = [
            buffer("impostor_models", &models, BufferUsages::UNIFORM),
            buffer("impostor_branches", &job.branches, BufferUsages::STORAGE),
            buffer("impostor_leaves", &job.leaves, BufferUsages::STORAGE),
            buffer("impostor_species", &job.species, BufferUsages::STORAGE),
            buffer("impostor_plants", &job.plants, BufferUsages::STORAGE),
        ];
        let layout = pipeline_cache.get_bind_group_layout(&baker.layout);
        let entries: Vec<BindGroupEntry> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| BindGroupEntry {
                binding: binding as u32,
                resource: if binding == 0 {
                    BindingResource::Buffer(BufferBinding {
                        buffer,
                        offset: 0,
                        size: BufferSize::new(size_of::<BakeModel>() as u64),
                    })
                } else {
                    buffer.as_entire_binding()
                },
            })
            .collect();
        let bind_group = device.create_bind_group("impostor_bake", &layout, &entries);
        (bind_group, buffers)
    });
    let mip_layout = pipeline_cache.get_bind_group_layout(&baker.mip_layout);
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("impostor_bake"),
    });
    let end = (active.next + TREES_A_FRAME).min(job.models.len());
    for layer in active.next..end {
        let model = &job.models[layer];
        let layer = layer as u32;
        let views = job.frames * job.frames;
        {
            let albedo_layer = layer_view(&albedo.texture, layer, 0);
            let normal_layer = layer_view(&normal.texture, layer, 0);
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("impostor_bake_tree"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &targets.albedo,
                        depth_slice: None,
                        resolve_target: Some(&albedo_layer),
                        ops: Operations {
                            load: LoadOp::Clear(LinearRgba::NONE.into()),
                            store: StoreOp::Discard,
                        },
                    }),
                    Some(RenderPassColorAttachment {
                        view: &targets.normal,
                        depth_slice: None,
                        resolve_target: Some(&normal_layer),
                        ops: Operations {
                            load: LoadOp::Clear(LinearRgba::NONE.into()),
                            store: StoreOp::Discard,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &targets.depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &**bind_group, &[layer * MODEL_STRIDE as u32]);
            let [_, branches, _, leaves] = model.records;
            let [sides, spans, blades, _] = model.shape;
            if branches > 0 {
                pass.set_pipeline(branch_pipeline);
                pass.draw(0..sides * spans * 6, 0..branches * views);
            }
            if leaves > 0 {
                pass.set_pipeline(leaf_pipeline);
                pass.draw(0..blades * 6, 0..leaves * views);
            }
        }
        for mip in 1..job.mips {
            let albedo_above = layer_view(&albedo.texture, layer, mip - 1);
            let normal_above = layer_view(&normal.texture, layer, mip - 1);
            let albedo_level = layer_view(&albedo.texture, layer, mip);
            let normal_level = layer_view(&normal.texture, layer, mip);
            let bind_group = device.create_bind_group(
                "impostor_mip",
                &mip_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&albedo_above),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&normal_above),
                    },
                ],
            );
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("impostor_mip"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &albedo_level,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(LinearRgba::NONE.into()),
                            store: StoreOp::Store,
                        },
                    }),
                    Some(RenderPassColorAttachment {
                        view: &normal_level,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(LinearRgba::NONE.into()),
                            store: StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(mip_pipeline);
            pass.set_bind_group(0, &*bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    render_queue.submit([encoder.finish()]);
    active.next = end;
    job.progress.store(end as u32, Ordering::Release);
    if end >= job.models.len() {
        baker.active = None;
    }
}

/// One layer and mip level of an atlas, as a target or a source.
fn layer_view(texture: &bevy::render::render_resource::Texture, layer: u32, mip: u32) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        label: Some("impostor_layer"),
        dimension: Some(TextureViewDimension::D2),
        base_mip_level: mip,
        mip_level_count: Some(1),
        base_array_layer: layer,
        array_layer_count: Some(1),
        ..default()
    })
}

impl Targets {
    fn new(device: &RenderDevice, size: u32) -> Self {
        let target = |label: &'static str, format: TextureFormat| {
            device
                .create_texture(&TextureDescriptor {
                    label: Some(label),
                    size: Extent3d {
                        width: size,
                        height: size,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: SAMPLES,
                    dimension: TextureDimension::D2,
                    format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&TextureViewDescriptor::default())
        };
        Self {
            size,
            albedo: target("impostor_bake_albedo", ALBEDO_FORMAT),
            normal: target("impostor_bake_normal", NORMAL_FORMAT),
            depth: target("impostor_bake_depth", DEPTH_FORMAT),
        }
    }
}

