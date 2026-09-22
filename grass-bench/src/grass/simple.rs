//! The hand-rolled grass path, the thing under test.
//!
//! Each tier's tufts are split into 8 m x 8 m chunks; every chunk is an
//! ordinary `Mesh3d` entity with a fitted `Aabb` and a `VisibilityRange`, so
//! bevy's own CPU frustum + range culling decides what draws. Each visible
//! chunk is one `draw_indexed` of the tier's tuft mesh with an instance range
//! over a per-chunk vertex buffer that is uploaded once and cached. The
//! per-instance tier crossfade is the game's complementary dither, computed
//! from the tuft root distance in the vertex shader (not bevy's per-entity
//! `VISIBILITY_RANGE_DITHER`), so it is identical to the eidolon path.

use std::collections::HashMap;

use bevy::camera::{primitives::Aabb, visibility::{NoAutoAabb, VisibilityRange}};
use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey};
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::{SystemParamItem, lifetimeless::{Read, SRes}};
use bevy::light::NotShadowCaster;
use bevy::math::Vec3A;
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout};
use bevy::pbr::{
    MeshPipeline, MeshPipelineKey, MeshPipelineSystems, RenderMeshInstances, SetMeshViewBindGroup,
    SetMeshViewBindingArrayBindGroup, ViewKeyCache,
};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::allocator::{MeshAllocator, MeshSlabs};
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem,
    RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
    ViewBinnedRenderPhases,
};
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer,
    BufferInitDescriptor, BufferUsages, CachedRenderPipelineId, CompareFunction, PipelineCache,
    RenderPipelineDescriptor, ShaderStages, ShaderType, SpecializedMeshPipeline,
    SpecializedMeshPipelineError, SpecializedMeshPipelines, UniformBuffer, VertexAttribute,
    VertexFormat, VertexStepMode,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::sync_component::SyncComponent;
use bevy::render::sync_world::MainEntity;
use bevy::render::view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy_eidolon::components::InstanceData;

use super::{GRASS_SIMPLE_SHADER_HANDLE, GrassEntity, TIER_COUNT, TIERS, Tier, TierMesh, fitted_aabb, tier_params};

/// Chunk edge, metres.
pub const CHUNK_SIZE: f32 = 8.0;

// ---------------------------------------------------------------------------
// Main-world components and resources
// ---------------------------------------------------------------------------

/// One tier's tufts within one chunk; extracted once (on change) and turned
/// into a cached GPU vertex buffer in the render world.
#[derive(Component, Clone)]
pub struct GrassChunk {
    pub tier: u32,
    pub instances: Vec<InstanceData>,
}

impl SyncComponent for GrassChunk {
    type Target = Self;
}

impl ExtractComponent for GrassChunk {
    type QueryData = &'static GrassChunk;
    /// Only re-extract when the main-world component changed: the render
    /// world keeps the previous copy (and its buffer) otherwise.
    type QueryFilter = Changed<GrassChunk>;
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

/// The five Vec4 grass uniforms of the game material, plus the tier's fade
/// band (eidolon carries that in its per-batch storage buffer instead).
#[derive(ShaderType, Clone, Copy, Default)]
pub struct GrassTierUniform {
    pub wind: Vec4,
    pub interaction: Vec4,
    pub interaction_motion: Vec4,
    pub params: Vec4,
    pub shading: Vec4,
    pub visibility_range: Vec4,
}

/// Per-tier material inputs; static for the bench, wind time comes from
/// `globals.time` in the shader.
#[derive(Resource, Clone, ExtractResource)]
pub struct GrassSimpleParams {
    pub tiers: [GrassTierUniform; TIER_COUNT],
}

impl Default for GrassSimpleParams {
    fn default() -> Self {
        Self::from_tiers(&TIERS)
    }
}

impl GrassSimpleParams {
    pub fn from_tiers(source: &[Tier; TIER_COUNT]) -> Self {
        let mut tiers = [GrassTierUniform::default(); TIER_COUNT];
        for (uniform, tier) in tiers.iter_mut().zip(source) {
            let params = tier_params(tier);
            *uniform = GrassTierUniform {
                wind: params.wind,
                interaction: params.interaction,
                interaction_motion: params.interaction_motion,
                params: params.params,
                shading: params.shading,
                visibility_range: tier.visibility_range(),
            };
        }
        Self { tiers }
    }
}

pub struct GrassSimplePlugin;

impl Plugin for GrassSimplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrassSimpleParams>().add_plugins((
            ExtractComponentPlugin::<GrassChunk>::default(),
            ExtractResourcePlugin::<GrassSimpleParams>::default(),
        ));
        app.sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, DrawGrassSimple>()
            .init_resource::<SpecializedMeshPipelines<GrassSimplePipeline>>()
            .init_resource::<GrassUniformBindGroups>()
            .init_resource::<QueuedGrassChunks>()
            .add_systems(
                RenderStartup,
                init_grass_simple_pipeline.after(MeshPipelineSystems),
            )
            .add_systems(
                Render,
                (
                    queue_grass_chunks
                        .in_set(RenderSystems::QueueMeshes)
                        .after(bevy::pbr::queue_material_meshes),
                    (prepare_chunk_buffers, prepare_grass_uniforms)
                        .in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

/// Buckets each tier's tufts into chunks and spawns one culled entity per
/// (tier, chunk).
pub fn spawn(
    commands: &mut Commands,
    tiers: &[Tier; TIER_COUNT],
    tier_meshes: &[TierMesh],
    batches: [Vec<InstanceData>; TIER_COUNT],
    _grass_shadows: bool,
) {
    // BENCH_GRASS_TIER_MASK (bits: near=1, near_edge=2, far=4, vista=8) for
    // isolating one tier while debugging.
    let tier_mask: u32 = std::env::var("BENCH_GRASS_TIER_MASK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(u32::MAX);
    for (tier_index, ((tier, mesh), instances)) in
        tiers.iter().zip(tier_meshes).zip(batches).enumerate()
    {
        if tier_mask & (1 << tier_index) == 0 {
            continue;
        }
        let mut chunks: HashMap<(i32, i32), Vec<InstanceData>> = HashMap::new();
        for instance in instances {
            let key = (
                (instance.position.x / CHUNK_SIZE).floor() as i32,
                (instance.position.z / CHUNK_SIZE).floor() as i32,
            );
            chunks.entry(key).or_default().push(instance);
        }
        let chunk_count = chunks.len();
        for ((cx, cz), mut instances) in chunks {
            for (index, instance) in instances.iter_mut().enumerate() {
                instance.index = index as u32;
            }
            // World-space fitted bounds, re-expressed around the chunk's own
            // translation so bevy's range test measures from the chunk centre.
            let world_aabb = fitted_aabb(&instances, tier.footprint());
            let center = Vec3::from(world_aabb.center);
            // The whole-chunk range only gates visibility; the per-instance
            // crossfade lives in the shader. Pad the band by the chunk's
            // maximum horizontal reach so an instance is never culled while
            // inside it. Constant per tier, so bevy sees four distinct ranges.
            let pad = CHUNK_SIZE * core::f32::consts::FRAC_1_SQRT_2 + tier.footprint();
            let start = (tier.fade_in[0] - pad).max(0.0);
            let end = tier.fade_out[1] + pad;
            commands.spawn((
                Name::new(format!(
                    "grass simple {} chunk {cx},{cz} ({} tufts)",
                    tier.name,
                    instances.len()
                )),
                GrassEntity,
                Mesh3d(mesh.handle.clone()),
                Transform::from_translation(center),
                Visibility::Inherited,
                Aabb {
                    center: Vec3A::ZERO,
                    half_extents: world_aabb.half_extents,
                },
                // bevy recomputes the Aabb from the mesh (one tuft at the
                // origin) when Mesh3d is added unless told not to.
                NoAutoAabb,
                VisibilityRange {
                    start_margin: start..start,
                    end_margin: end..end,
                    use_aabb: false,
                },
                // No shadow phase in this iteration; also spares the light's
                // visibility pass the chunk walk.
                NotShadowCaster,
                GrassChunk {
                    tier: tier_index as u32,
                    instances,
                },
            ));
        }
        info!("grass simple {}: {chunk_count} chunks of {CHUNK_SIZE} m", tier.name);
    }
}

// ---------------------------------------------------------------------------
// Render world
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct GrassSimplePipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
    uniform_layout: BindGroupLayoutDescriptor,
}

fn init_grass_simple_pipeline(mut commands: Commands, mesh_pipeline: Res<MeshPipeline>) {
    let uniform_layout = BindGroupLayoutDescriptor::new(
        "grass_simple_uniform_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX_FRAGMENT,
            uniform_buffer::<GrassTierUniform>(false),
        ),
    );
    commands.insert_resource(GrassSimplePipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        shader: GRASS_SIMPLE_SHADER_HANDLE,
        uniform_layout,
    });
}

impl SpecializedMeshPipeline for GrassSimplePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.label = Some("grass_simple_pipeline".into());
        descriptor.vertex.shader = self.shader.clone();

        // Groups 0 and 1 are the view; the mesh bind group at 2 is replaced by
        // the tier uniform (the instance buffer carries the transforms).
        if descriptor.layout.len() > 2 {
            descriptor.layout[2] = self.uniform_layout.clone();
        } else {
            descriptor.layout.push(self.uniform_layout.clone());
        }
        descriptor.layout.truncate(3);

        // The chunks never appear in the prepass, so always test and write
        // depth regardless of the camera's prepass flags.
        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.depth_write_enabled = Some(true);
            depth_stencil.depth_compare = Some(CompareFunction::GreaterEqual);
        }
        descriptor.primitive.cull_mode = None;
        if descriptor.multisample.count > 1 {
            descriptor.multisample.alpha_to_coverage_enabled = true;
        }

        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader = self.shader.clone();
            if let Some(Some(target)) = fragment.targets.get_mut(0) {
                target.blend = None;
            }
        }

        // Per-instance attributes at locations 8-12, eidolon's layout.
        let rotation_offset = VertexFormat::Float32x4.size();
        let index_offset = rotation_offset + VertexFormat::Float32.size();
        let batch_id_offset = index_offset + VertexFormat::Uint32.size();
        let seed_offset = batch_id_offset + VertexFormat::Uint32.size();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 8,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    offset: rotation_offset,
                    shader_location: 9,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: index_offset,
                    shader_location: 10,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: batch_id_offset,
                    shader_location: 11,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: seed_offset,
                    shader_location: 12,
                },
            ],
        });
        Ok(descriptor)
    }
}

/// The instance buffer one draw reads: a cached per-chunk upload here, the
/// per-tier buffer refilled every frame in `culled.rs`. Whatever carries it
/// is queued and drawn by this module.
#[derive(Component)]
pub struct GrassChunkBuffer {
    pub buffer: Buffer,
    pub length: u32,
    pub tier: u32,
}

/// Uploads a chunk's instances once: on first sight and whenever the
/// extracted component changes (which, for the bench, is only at spawn).
fn prepare_chunk_buffers(
    mut commands: Commands,
    chunks: Query<(Entity, &GrassChunk), Or<(Changed<GrassChunk>, Without<GrassChunkBuffer>)>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, chunk) in &chunks {
        if chunk.instances.is_empty() {
            continue;
        }
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("grass_chunk_instances"),
            contents: bytemuck::cast_slice(&chunk.instances),
            usage: BufferUsages::VERTEX,
        });
        commands.entity(entity).insert(GrassChunkBuffer {
            buffer,
            length: chunk.instances.len() as u32,
            tier: chunk.tier,
        });
    }
}

/// One uniform buffer and bind group per tier (group 2).
#[derive(Resource, Default)]
struct GrassUniformBindGroups {
    buffers: Vec<UniformBuffer<GrassTierUniform>>,
    bind_groups: Vec<BindGroup>,
}

fn prepare_grass_uniforms(
    params: Option<Res<GrassSimpleParams>>,
    mut uniforms: ResMut<GrassUniformBindGroups>,
    pipeline: Res<GrassSimplePipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(params) = params else {
        return;
    };
    if !params.is_changed() && uniforms.bind_groups.len() == TIER_COUNT {
        return;
    }
    let layout = pipeline_cache.get_bind_group_layout(&pipeline.uniform_layout);
    uniforms
        .buffers
        .resize_with(TIER_COUNT, UniformBuffer::default);
    uniforms.bind_groups.clear();
    let GrassUniformBindGroups {
        buffers,
        bind_groups,
    } = &mut *uniforms;
    for (buffer, tier) in buffers.iter_mut().zip(&params.tiers) {
        buffer.set(*tier);
        buffer.write_buffer(&render_device, &render_queue);
        bind_groups.push(render_device.create_bind_group(
            "grass_simple_uniform_bind_group",
            &layout,
            &BindGroupEntries::single(buffer.binding().unwrap()),
        ));
    }
}

/// Which chunks each view has binned, with the pipeline they were binned
/// under. The `Opaque3d` phase is retained across frames, so entries are
/// removed when a chunk stops being visible and re-binned when its pipeline
/// changes (MSAA, prepass flags).
#[derive(Resource, Default)]
struct QueuedGrassChunks(HashMap<RetainedViewEntity, HashMap<MainEntity, CachedRenderPipelineId>>);

#[expect(clippy::too_many_arguments, reason = "bevy system")]
fn queue_grass_chunks(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    grass_pipeline: Res<GrassSimplePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<GrassSimplePipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    mesh_allocator: Res<MeshAllocator>,
    chunks: Query<(Entity, &MainEntity, &GrassChunkBuffer)>,
    mut phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    view_key_cache: Res<ViewKeyCache>,
    mut queued: ResMut<QueuedGrassChunks>,
    mut diag_frame: Local<u32>,
) {
    *diag_frame += 1;
    let diag = *diag_frame % 180 == 0;
    let draw_function = draw_functions.read().id::<DrawGrassSimple>();
    for (view, visible_entities) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let Some(&view_key) = view_key_cache.get(&view.retained_view_entity) else {
            continue;
        };
        let Some(visible_meshes) = visible_entities.get::<Mesh3d>() else {
            continue;
        };
        let queued = queued.0.entry(view.retained_view_entity).or_default();
        let mut visible_per_tier = [0u32; TIER_COUNT];
        let mut total_per_tier = [0u32; TIER_COUNT];

        // The visible list's render entities are placeholders for meshes in
        // bevy 0.19, so walk the entities carrying an instance buffer (real
        // render entities; chunks here, tiers in `culled.rs`) and test each
        // one's main entity against the sorted visible list.
        for (render_entity, main_entity, chunk) in &chunks {
            let main_entity = *main_entity;
            let visible = visible_meshes
                .entities_cpu_culling
                .binary_search_by_key(&main_entity, |(_, m)| *m)
                .is_ok();
            total_per_tier[chunk.tier as usize] += 1;
            if visible {
                visible_per_tier[chunk.tier as usize] += 1;
            }
            if !visible {
                if queued.remove(&main_entity).is_some() {
                    phase.remove(main_entity);
                }
                continue;
            }
            if chunk.length == 0 {
                continue;
            }
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(main_entity)
            else {
                continue;
            };
            let mesh_id = mesh_instance.mesh_asset_id();
            let Some(mesh) = meshes.get(mesh_id) else {
                continue;
            };
            let key = view_key
                | MeshPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    mesh.index_format(),
                );
            let pipeline = match pipelines.specialize(&pipeline_cache, &grass_pipeline, key, &mesh.layout) {
                Ok(pipeline) => pipeline,
                Err(err) => {
                    error!("grass simple pipeline: {err}");
                    continue;
                }
            };
            // Re-added every frame: bevy's own material queue runs in the same
            // set and dequeues every visible mesh it never specialized (these
            // chunks have no material), so retention cannot be relied on.
            let Some(slabs) = mesh_allocator.mesh_slabs(&mesh_id) else {
                continue;
            };
            if slabs.index_slab_id.is_none() {
                continue;
            }
            phase.remove(main_entity);
            phase.add(
                Opaque3dBatchSetKey {
                    pipeline,
                    draw_function,
                    material_bind_group_index: Some(chunk.tier),
                    slabs: MeshSlabs {
                        vertex_slab_id: slabs.vertex_slab_id,
                        index_slab_id: slabs.index_slab_id,
                        ..Default::default()
                    },
                    lightmap_slab: None,
                },
                Opaque3dBinKey {
                    asset_id: mesh_id.into(),
                },
                (render_entity, main_entity),
                InputUniformIndex(0),
                BinnedRenderPhaseType::UnbatchableMesh,
            );
            queued.insert(main_entity, pipeline);
        }
        if diag {
            info!(
                "grass simple view {:?}: visible chunks per tier {:?} of {:?}, queued {}",
                view.retained_view_entity, visible_per_tier, total_per_tier, queued.len()
            );
        }
    }
}

type DrawGrassSimple = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewBindingArrayBindGroup<1>,
    SetGrassUniformBindGroup<2>,
    DrawGrassChunk,
);

struct SetGrassUniformBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetGrassUniformBindGroup<I> {
    type Param = SRes<GrassUniformBindGroups>;
    type ViewQuery = ();
    type ItemQuery = Read<GrassChunkBuffer>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        chunk: Option<&'w GrassChunkBuffer>,
        uniforms: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(chunk) = chunk else {
            return RenderCommandResult::Skip;
        };
        let Some(bind_group) = uniforms.into_inner().bind_groups.get(chunk.tier as usize) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[]);
        RenderCommandResult::Success
    }
}

/// One `draw_indexed` of the tier tuft mesh over the chunk's instance range.
struct DrawGrassChunk;

impl<P: PhaseItem> RenderCommand<P> for DrawGrassChunk {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<GrassChunkBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        chunk: Option<&'w GrassChunkBuffer>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();
        let Some(chunk) = chunk else {
            return RenderCommandResult::Skip;
        };
        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let mesh_id = mesh_instance.mesh_asset_id();
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) = mesh_allocator.mesh_vertex_slice(&mesh_id) else {
            return RenderCommandResult::Skip;
        };
        let RenderMeshBufferInfo::Indexed {
            index_format,
            count,
        } = &gpu_mesh.buffer_info
        else {
            return RenderCommandResult::Skip;
        };
        let Some(index_buffer_slice) = mesh_allocator.mesh_index_slice(&mesh_id) else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, chunk.buffer.slice(..));
        pass.set_index_buffer(index_buffer_slice.buffer.slice(..), *index_format);
        pass.draw_indexed(
            index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
            vertex_buffer_slice.range.start as i32,
            0..chunk.length,
        );
        RenderCommandResult::Success
    }
}
