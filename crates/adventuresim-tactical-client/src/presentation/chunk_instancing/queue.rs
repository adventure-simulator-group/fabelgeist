//! Queues every chunk Bevy's CPU culling left visible into each camera's main
//! pass and, when the camera has one, its depth prepass.
use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    core_pipeline::{
        core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey},
        prepass::{
            DepthPrepass, MotionVectorPrepass, NormalPrepass, Opaque3dPrepass,
            OpaqueNoLightmap3dBatchSetKey, OpaqueNoLightmap3dBinKey,
        },
    },
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
        render_resource::{
            CachedRenderPipelineId, PipelineCache, SpecializedMeshPipelineError,
            SpecializedMeshPipelines,
        },
        sync_world::MainEntity,
        view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity},
    },
};

use super::{
    ChunkInstancedMaterial, ChunkMaterial,
    draw::ChunkGpu,
    material::PreparedChunkMaterial,
    pipeline::{ChunkMaterialPipeline, ChunkPipelineKey, ChunkPrepassPipeline},
};

/// The chunks each view binned last frame, so chunks that left the view or
/// were despawned leave its retained phases.
#[derive(Resource)]
pub(super) struct QueuedChunks<M> {
    views: HashMap<RetainedViewEntity, HashSet<MainEntity>>,
    material: PhantomData<M>,
}

/// The pipelines one material specializes per chunk mesh.
#[derive(SystemParam)]
pub(super) struct ChunkPipelines<'w, M: ChunkInstancedMaterial> {
    main: Res<'w, ChunkMaterialPipeline<M>>,
    prepass: Res<'w, ChunkPrepassPipeline<M>>,
    main_specialized: ResMut<'w, SpecializedMeshPipelines<ChunkMaterialPipeline<M>>>,
    prepass_specialized: ResMut<'w, SpecializedMeshPipelines<ChunkPrepassPipeline<M>>>,
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

/// One camera's render phases and prepass configuration.
struct ViewPhases<'a> {
    main: &'a mut BinnedRenderPhase<Opaque3d>,
    view_key: MeshPipelineKey,
    prepass: Option<ViewPrepass<'a>>,
}

struct ViewPrepass<'a> {
    phase: &'a mut BinnedRenderPhase<Opaque3dPrepass>,
    key: MeshPipelineKey,
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

impl ViewPhases<'_> {
    fn remove(&mut self, entity: MainEntity) {
        self.main.remove(entity);
        if let Some(prepass) = self.prepass.as_mut() {
            prepass.phase.remove(entity);
        }
    }

    /// Bins a visible chunk once its main-pass pipeline has compiled. A chunk
    /// whose prepass pipeline is still compiling skips only the prepass: its
    /// main pass writes depth itself.
    fn bin<M: ChunkInstancedMaterial>(
        &mut self,
        draw: ChunkDraw<'_, M>,
        pipelines: &mut ChunkPipelines<'_, M>,
        assets: &ChunkAssets<'_, M>,
    ) -> bool {
        let Some(prepared) = assets.materials.get(draw.material) else {
            return false;
        };
        let Some(mesh_id) = assets
            .mesh_instances
            .render_mesh_queue_data(draw.main_entity)
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
        let topology = MeshPipelineKey::from_primitive_topology_and_strip_index(
            mesh.primitive_topology(),
            mesh.index_format(),
        );
        let key = |mesh_key| ChunkPipelineKey::<M> {
            mesh_key: mesh_key | topology,
            material_key: prepared.key.clone(),
        };
        let Some(main_id) = ready_pipeline(
            pipelines.main_specialized.specialize(
                &pipelines.cache,
                &pipelines.main,
                key(self.view_key),
                &mesh.layout,
            ),
            &pipelines.cache,
        ) else {
            return false;
        };
        let slabs = MeshSlabs {
            vertex_slab_id: slabs.vertex_slab_id,
            index_slab_id: slabs.index_slab_id,
            ..default()
        };
        let material_index = material_bind_group_index(draw.material);
        let entities = (draw.render_entity, draw.main_entity);
        self.main.add(
            Opaque3dBatchSetKey {
                pipeline: main_id,
                draw_function: prepared.draw_main,
                material_bind_group_index: Some(material_index),
                slabs,
                lightmap_slab: None,
            },
            Opaque3dBinKey {
                asset_id: mesh_id.into(),
            },
            entities,
            InputUniformIndex(0),
            BinnedRenderPhaseType::UnbatchableMesh,
        );
        let Some(prepass) = self.prepass.as_mut() else {
            return true;
        };
        if let Some(prepass_id) = ready_pipeline(
            pipelines.prepass_specialized.specialize(
                &pipelines.cache,
                &pipelines.prepass,
                key(prepass.key),
                &mesh.layout,
            ),
            &pipelines.cache,
        ) {
            prepass.phase.add(
                OpaqueNoLightmap3dBatchSetKey {
                    pipeline: prepass_id,
                    draw_function: prepared.draw_prepass,
                    material_bind_group_index: Some(material_index),
                    slabs,
                },
                OpaqueNoLightmap3dBinKey {
                    asset_id: mesh_id.into(),
                },
                entities,
                InputUniformIndex(0),
                BinnedRenderPhaseType::UnbatchableMesh,
            );
        }
        true
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a Bevy render system reads every pipeline, asset and phase resource it queues into"
)]
#[expect(
    clippy::type_complexity,
    reason = "the view query reads Bevy's per-camera prepass markers"
)]
pub(super) fn queue_chunks<M: ChunkInstancedMaterial>(
    mut pipelines: ChunkPipelines<M>,
    assets: ChunkAssets<M>,
    chunks: Query<(Entity, &MainEntity, &ChunkMaterial<M>), With<ChunkGpu>>,
    mut main_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    mut prepass_phases: ResMut<ViewBinnedRenderPhases<Opaque3dPrepass>>,
    views: Query<(
        &ExtractedView,
        &RenderVisibleEntities,
        &Msaa,
        Has<DepthPrepass>,
        Has<NormalPrepass>,
        Has<MotionVectorPrepass>,
    )>,
    view_key_cache: Res<ViewKeyCache>,
    mut queued: ResMut<QueuedChunks<M>>,
) {
    for (view, visible, msaa, depth_prepass, normal_prepass, motion_vectors) in &views {
        let retained = view.retained_view_entity;
        let (Some(main), Some(&view_key), Some(visible)) = (
            main_phases.get_mut(&retained),
            view_key_cache.get(&retained),
            visible.get::<Mesh3d>(),
        ) else {
            continue;
        };
        let mut prepass_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        prepass_key.set(MeshPipelineKey::DEPTH_PREPASS, depth_prepass);
        prepass_key.set(MeshPipelineKey::NORMAL_PREPASS, normal_prepass);
        prepass_key.set(MeshPipelineKey::MOTION_VECTOR_PREPASS, motion_vectors);
        let mut phases = ViewPhases {
            main,
            view_key,
            prepass: (depth_prepass || normal_prepass || motion_vectors)
                .then(|| prepass_phases.get_mut(&retained))
                .flatten()
                .map(|phase| ViewPrepass {
                    phase,
                    key: prepass_key,
                }),
        };
        let previously = queued.views.remove(&retained).unwrap_or_default();
        let mut now = HashSet::new();
        for (render_entity, &main_entity, material) in &chunks {
            // Bevy's own material queues dequeue every visible mesh they did
            // not specialize, so a chunk is re-binned every frame it shows.
            phases.remove(main_entity);
            let is_visible = visible
                .entities_cpu_culling
                .binary_search_by_key(&main_entity, |(_, visible)| *visible)
                .is_ok();
            let draw = ChunkDraw {
                render_entity,
                main_entity,
                material: &material.0,
            };
            if is_visible && phases.bin(draw, &mut pipelines, &assets) {
                now.insert(main_entity);
            }
        }
        // Chunks despawned since the last frame are no longer in the query.
        for gone in previously.difference(&now) {
            phases.remove(*gone);
        }
        queued.views.insert(retained, now);
    }
}

/// The specialized pipeline, once it has compiled.
fn ready_pipeline(
    specialized: Result<CachedRenderPipelineId, SpecializedMeshPipelineError>,
    pipeline_cache: &PipelineCache,
) -> Option<CachedRenderPipelineId> {
    let id = specialized
        .inspect_err(|error| error!("chunk-instanced pipeline: {error}"))
        .ok()?;
    pipeline_cache.get_render_pipeline(id).map(|_| id)
}

/// Bins chunks that share a material together.
fn material_bind_group_index<M: ChunkInstancedMaterial>(material: &Handle<M>) -> u32 {
    let mut hasher = DefaultHasher::new();
    material.id().hash(&mut hasher);
    hasher.finish() as u32
}
