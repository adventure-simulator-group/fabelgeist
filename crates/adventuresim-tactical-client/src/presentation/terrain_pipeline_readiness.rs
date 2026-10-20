//! Prepare required detail-terrain color pipelines before their first visibility.
//!
//! The detail patch's depth bias is intentional raster state. Its permutation
//! belongs to scene initialization, even from a distant camera. This reserves
//! ordinary Bevy pipelines; it never draws hidden geometry. It does not prove
//! uploads, prepass/shadow readiness, pixels or the inclusive arrival budget.

use super::terrain::{ScenePresentationOf, TacticalTerrainMaterial, TerrainDetailPatch};
use super::{ActiveTacticalScene, TacticalGameplayCamera, TerrainTriangleCount};
use bevy::{
    camera::{CameraUpdateSystems, visibility::VisibilityRange},
    ecs::query::QueryData,
    mesh::{morph::MeshMorphWeights, skinning::SkinnedMesh},
    pbr::{Lightmap, MeshPipelineKey},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::CachedRenderPipelineId,
        sync_world::RenderEntity,
    },
};
use std::sync::{Arc, Mutex};

mod prepare;

#[derive(Resource, Clone, Default, ExtractResource)]
pub(crate) struct TerrainPipelineReadiness {
    request: Option<Arc<RequiredPipelines>>,
}

pub(super) struct TerrainPipelinePreparationPlugin;

#[derive(Clone, PartialEq, Eq)]
struct DetailTarget {
    entity: Entity,
    mesh: AssetId<Mesh>,
    material: AssetId<TacticalTerrainMaterial>,
}

struct RequiredPipelines {
    scene: Entity,
    camera: Entity,
    render_camera: RenderEntity,
    targets: Vec<DetailTarget>,
    state: Mutex<Preparation>,
}

struct PreparedDetail {
    target: DetailTarget,
    view_key: MeshPipelineKey,
    pipeline: CachedRenderPipelineId,
    depth_bias: f32,
}

#[derive(QueryData)]
struct DetailPatch {
    entity: Entity,
    owner: &'static ScenePresentationOf,
    mesh: &'static Mesh3d,
    material: &'static MeshMaterial3d<TacticalTerrainMaterial>,
    triangles: &'static TerrainTriangleCount,
    skinned: Has<SkinnedMesh>,
    morph: Has<MeshMorphWeights>,
    lightmap: Has<Lightmap>,
    range: Has<VisibilityRange>,
}

#[derive(Default)]
enum Preparation {
    #[default]
    Pending,
    Ready(Vec<PreparedDetail>),
}

impl TerrainPipelineReadiness {
    /// New scene or camera ownership cannot reuse an older acknowledgement.
    pub(crate) fn is_ready_for(&self, scene: Entity, camera: Entity) -> bool {
        self.request.as_ref().is_some_and(|request| {
            request.scene == scene
                && request.camera == camera
                && request
                    .state
                    .lock()
                    .is_ok_and(|state| matches!(*state, Preparation::Ready(_)))
        })
    }

    /// Last render preparation only; this is not a pixel receipt.
    pub(crate) fn report(&self) -> serde_json::Value {
        let Some(request) = &self.request else {
            return serde_json::json!({"ready":false,"reason":"no current scene/camera demand"});
        };
        let Ok(state) = request.state.lock() else {
            return serde_json::json!({"ready":false,"reason":"terrain preparation lock poisoned"});
        };
        let Preparation::Ready(pipelines) = &*state else {
            return serde_json::json!({"ready":false,"scene_bits":request.scene.to_bits(),"camera_bits":request.camera.to_bits()});
        };
        serde_json::json!({"ready":true,"scene_bits":request.scene.to_bits(),"camera_bits":request.camera.to_bits(),
            "render_camera_bits":request.render_camera.id().to_bits(),"color_pipelines":pipelines.iter().map(|prepared|serde_json::json!({
                "entity_bits":prepared.target.entity.to_bits(),"mesh":format!("{:?}",prepared.target.mesh),
                "material":format!("{:?}",prepared.target.material),"view_key":format!("{:?}",prepared.view_key),
                "pipeline":format!("{:?}",prepared.pipeline),"depth_bias_texture_units":prepared.depth_bias,
            })).collect::<Vec<_>>(),"scope":"required static detail-terrain color permutations in the shared Bevy cache; no hidden drawing, prepass/shadow, upload, pixel or arrival acceptance"})
    }
}

impl Plugin for TerrainPipelinePreparationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainPipelineReadiness>()
            .add_plugins(ExtractResourcePlugin::<TerrainPipelineReadiness>::default())
            .add_systems(PostUpdate, sync_request.after(CameraUpdateSystems));
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.add_systems(
                Render,
                prepare::prepare
                    .in_set(RenderSystems::Specialize)
                    .after(bevy::pbr::check_views_need_specialization),
            );
        }
    }
}

fn sync_request(
    active: Res<ActiveTacticalScene>,
    cameras: Query<(Entity, &Camera, &RenderEntity), With<TacticalGameplayCamera>>,
    patches: Query<DetailPatch, With<TerrainDetailPatch>>,
    mut readiness: ResMut<TerrainPipelineReadiness>,
) -> bevy::prelude::Result {
    let mut cameras = cameras.iter().filter(|(_, camera, _)| camera.is_active);
    let Some(scene) = active.entity else {
        readiness.request = None;
        return Ok(());
    };
    let Some((camera, _, render_camera)) = cameras.next() else {
        readiness.request = None;
        return Ok(());
    };
    if cameras.next().is_some() {
        readiness.request = None;
        return Ok(());
    }
    let mut seen = false;
    let mut targets = Vec::new();
    for patch in &patches {
        if patch.owner.0 != scene {
            continue;
        }
        seen = true;
        if patch.skinned || patch.morph || patch.lightmap || patch.range {
            return Err("required terrain pipeline supports only static unlightmapped non-crossfading detail patches".into());
        }
        if patch.triangles.0 > 0 {
            targets.push(DetailTarget {
                entity: patch.entity,
                mesh: patch.mesh.0.id(),
                material: patch.material.0.id(),
            });
        }
    }
    if !seen {
        readiness.request = None;
        return Ok(());
    }
    targets.sort_unstable_by_key(|target| target.entity.to_bits());
    if readiness.request.as_ref().is_some_and(|request| {
        request.scene == scene
            && request.camera == camera
            && request.render_camera == *render_camera
            && request.targets == targets
    }) {
        return Ok(());
    }
    readiness.request = Some(Arc::new(RequiredPipelines {
        scene,
        camera,
        render_camera: *render_camera,
        targets,
        state: Mutex::new(Preparation::Pending),
    }));
    Ok(())
}
