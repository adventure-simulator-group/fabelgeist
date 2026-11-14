//! Queues every chunk Bevy's CPU culling left visible into each camera's main
//! pass.
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey},
    ecs::system::SystemParam,
    mesh::Mesh3d,
    pbr::{MeshPipelineKey, RenderMeshInstances, ViewKeyCache},
    prelude::*,
    render::{
        mesh::{
            RenderMesh,
            allocator::{MeshAllocator, MeshSlabs},
        },
        render_asset::RenderAssets,
        render_phase::{
            BinnedRenderPhase, BinnedRenderPhaseType, InputUniformIndex, ViewBinnedRenderPhases,
        },
        render_resource::{CachedRenderPipelineId, PipelineCache, SpecializedMeshPipelines},
        sync_world::MainEntity,
        view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity},
    },
};

use super::{
    ChunkInstancedMaterial, ChunkMaterial,
    draw::ChunkGpu,
    material::PreparedChunkMaterial,
    pipeline::{ChunkMaterialPipeline, ChunkPipelineKey},
};

/// The chunks each view binned last frame, so chunks that left the view or
/// were despawned leave its retained phase.
#[derive(Resource)]
pub(super) struct QueuedChunks<M> {
    views: HashMap<RetainedViewEntity, HashSet<MainEntity>>,
    material: PhantomData<M>,
}

/// The pipeline one material specializes per chunk mesh.
#[derive(SystemParam)]
pub(super) struct ChunkPipelines<'w, M: ChunkInstancedMaterial> {
    pipeline: Res<'w, ChunkMaterialPipeline<M>>,
    specialized: ResMut<'w, SpecializedMeshPipelines<ChunkMaterialPipeline<M>>>,
    cache: Res<'w, PipelineCache>,
}

/// The render-world mesh and material state a chunk draw needs.
#[derive(SystemParam)]
pub(super) struct ChunkAssets<'w, M: ChunkInstancedMaterial> {
    meshes: Res<'w, RenderAssets<RenderMesh>>,
    mesh_instances: Res<'w, RenderMeshInstances>,
    mesh_allocator: Res<'w, MeshAllocator>,
    materials: Res<'w, RenderAssets<PreparedChunkMaterial<M>>>,
}

/// One visible chunk to bin.
struct ChunkDraw<'a, M: ChunkInstancedMaterial> {
    render_entity: Entity,
    main_entity: MainEntity,
    material: &'a Handle<M>,
}

impl<M> Default for QueuedChunks<M> {
    fn default() -> Self {
        Self {
            views: HashMap::new(),
            material: PhantomData,
        }
    }
}

impl<M: ChunkInstancedMaterial> ChunkDraw<'_, M> {
    /// Bins the chunk into `phase` once its pipeline has compiled.
    fn bin(
        self,
        phase: &mut BinnedRenderPhase<Opaque3d>,
        view_key: MeshPipelineKey,
        pipelines: &mut ChunkPipelines<'_, M>,
        assets: &ChunkAssets<'_, M>,
    ) -> bool {
        let Some(prepared) = assets.materials.get(self.material) else {
            return false;
        };
        let Some(mesh_id) = assets
            .mesh_instances
            .render_mesh_queue_data(self.main_entity)
            .map(|instance| instance.mesh_asset_id())
        else {
            return false;
        };
        let (Some(mesh), Some(slabs)) = (
            assets.meshes.get(mesh_id),
            assets.mesh_allocator.mesh_slabs(&mesh_id),
        ) else {
            return false;
        };
        if slabs.index_slab_id.is_none() {
            return false;
        }
        let key = ChunkPipelineKey::<M> {
            mesh_key: view_key
                | MeshPipelineKey::from_primitive_topology_and_strip_index(
                    mesh.primitive_topology(),
                    mesh.index_format(),
                ),
            material_key: prepared.key.clone(),
        };
        let Some(pipeline) = pipelines
            .specialized
            .specialize(&pipelines.cache, &pipelines.pipeline, key, &mesh.layout)
            .inspect_err(|error| error!("chunk-instanced pipeline: {error}"))
            .ok()
            .filter(|id| ready(*id, &pipelines.cache))
        else {
            return false;
        };
        phase.add(
            Opaque3dBatchSetKey {
                pipeline,
                draw_function: prepared.draw_main,
                material_bind_group_index: Some(material_bind_group_index(self.material)),
                slabs: MeshSlabs {
                    vertex_slab_id: slabs.vertex_slab_id,
                    index_slab_id: slabs.index_slab_id,
                    ..default()
                },
                lightmap_slab: None,
            },
            Opaque3dBinKey {
                asset_id: mesh_id.into(),
            },
            (self.render_entity, self.main_entity),
            InputUniformIndex(0),
            BinnedRenderPhaseType::UnbatchableMesh,
        );
        true
    }
}

pub(super) fn queue_chunks<M: ChunkInstancedMaterial>(
    mut pipelines: ChunkPipelines<M>,
    assets: ChunkAssets<M>,
    chunks: Query<(Entity, &MainEntity, &ChunkMaterial<M>), With<ChunkGpu>>,
    mut phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    view_key_cache: Res<ViewKeyCache>,
    mut queued: ResMut<QueuedChunks<M>>,
) {
    for (view, visible) in &views {
        let retained = view.retained_view_entity;
        let (Some(phase), Some(&view_key), Some(visible)) = (
            phases.get_mut(&retained),
            view_key_cache.get(&retained),
            visible.get::<Mesh3d>(),
        ) else {
            continue;
        };
        let previously = queued.views.remove(&retained).unwrap_or_default();
        let mut now = HashSet::new();
        for (render_entity, &main_entity, material) in &chunks {
            // Bevy's own material queue dequeues every visible mesh it did
            // not specialize, so a chunk is re-binned every frame it shows.
            phase.remove(main_entity);
            let is_visible = visible
                .entities_cpu_culling
                .binary_search_by_key(&main_entity, |(_, visible)| *visible)
                .is_ok();
            let draw = ChunkDraw {
                render_entity,
                main_entity,
                material: &material.0,
            };
            if is_visible && draw.bin(phase, view_key, &mut pipelines, &assets) {
                now.insert(main_entity);
            }
        }
        // Chunks despawned since the last frame are no longer in the query.
        for gone in previously.difference(&now) {
            phase.remove(*gone);
        }
        queued.views.insert(retained, now);
    }
}

/// Whether the specialized pipeline has finished compiling.
fn ready(id: CachedRenderPipelineId, pipeline_cache: &PipelineCache) -> bool {
    pipeline_cache.get_render_pipeline(id).is_some()
}

/// Bins chunks that share a material together.
fn material_bind_group_index<M: ChunkInstancedMaterial>(material: &Handle<M>) -> u32 {
    let mut hasher = DefaultHasher::new();
    material.id().hash(&mut hasher);
    hasher.finish() as u32
}
