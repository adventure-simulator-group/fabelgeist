//! Verify production recipe/material bindings and GPU residency before review readbacks.
use adventuresim_tactical_core::prelude::SceneFurniture;
use bevy::{ecs::system::SystemParam, prelude::*, render::renderer::RenderAdapterInfo};

use super::{
    capture_state::{CapturePhase, SceneCaptureState},
    gpu_readiness::GpuReadiness,
};
use crate::presentation::{CityGroundMaterial, PresentedFurnitureMesh, TacticalBuildingMaterials};

const MAX_ASSET_WAIT_SECONDS: f64 = 120.0;

pub(super) fn install(app: &mut App, profile: &str) {
    if super::building_review::is_profile(profile) {
        app.add_plugins(super::building_review::BuildingReviewPlugin);
    } else if profile == super::furniture_capture::PROFILE {
        GpuReadiness::install(app);
        app.init_resource::<Readiness>()
            .add_systems(Last, observe.before(super::capture_views));
    }
}

#[derive(Resource)]
pub(super) struct ExpectedFurniture {
    pub(super) instances: usize,
    pub(super) batches: usize,
}

#[derive(Default, Resource)]
pub(super) struct Readiness {
    ready: bool,
    pending_since: Option<f64>,
    recorded: std::collections::BTreeSet<usize>,
}

pub(super) fn ready(
    state: Option<Res<SceneCaptureState>>,
    readiness: Option<Res<Readiness>>,
) -> bool {
    readiness.is_none_or(|readiness| readiness.ready)
        || state.is_none_or(|state| state.phase == CapturePhase::Configure)
}

#[derive(SystemParam)]
struct Observation<'w, 's> {
    furniture: Query<'w, 's, &'static SceneFurniture>,
    batches: Query<
        'w,
        's,
        (
            &'static PresentedFurnitureMesh,
            &'static ChildOf,
            &'static Mesh3d,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    materials: Res<'w, Assets<StandardMaterial>>,
    ground: Query<'w, 's, (&'static Mesh3d, &'static MeshMaterial3d<CityGroundMaterial>)>,
    ground_materials: Res<'w, Assets<CityGroundMaterial>>,
    palette: Res<'w, TacticalBuildingMaterials>,
    gpu: Res<'w, GpuReadiness>,
    adapter: Res<'w, RenderAdapterInfo>,
}

impl Observation<'_, '_> {
    fn check(&self, expected: &ExpectedFurniture) -> bool {
        !self.ground.is_empty()
            && self.ground.iter().all(|(mesh, handle)| {
                self.ground_materials
                    .get(&handle.0)
                    .is_some_and(|material| {
                        self.gpu.contains(&mesh.0, material.extension.texture_ids())
                    })
            })
            && self.furniture.iter().count() == expected.instances
            && self.batches.iter().count() == expected.batches
            && self.batches.iter().all(|(batch, parent, mesh, handle)| {
                let Ok(instance) = self.furniture.get(parent.parent()) else {
                    return false;
                };
                if handle.0 != self.palette.get_for_building(instance.id.0, batch.material) {
                    return false;
                }
                self.materials.get(&handle.0).is_some_and(|material| {
                    self.gpu.contains(
                        &mesh.0,
                        [
                            &material.base_color_texture,
                            &material.normal_map_texture,
                            &material.metallic_roughness_texture,
                        ]
                        .into_iter()
                        .flatten()
                        .map(Handle::id),
                    )
                })
            })
    }
}

fn observe(
    mut readiness: ResMut<Readiness>,
    expected: Option<Res<ExpectedFurniture>>,
    state: Option<Res<SceneCaptureState>>,
    time: Res<Time<Real>>,
    observation: Observation,
) {
    let (Some(expected), Some(state)) = (expected, state) else {
        return;
    };
    if state.phase == CapturePhase::Configure {
        readiness.ready = false;
        readiness.pending_since = None;
        return;
    }
    readiness.ready = observation.check(&expected);
    if !readiness.ready {
        let since = *readiness
            .pending_since
            .get_or_insert(time.elapsed_secs_f64());
        assert!(
            time.elapsed_secs_f64() - since < MAX_ASSET_WAIT_SECONDS,
            "furniture production meshes or textures did not become GPU resident"
        );
        return;
    }
    if readiness.recorded.insert(state.view) {
        let evidence = serde_json::json!({
            "renderer": "TacticalPresentationPlugin", "backend": format!("{:?}", observation.adapter.backend),
            "adapter": observation.adapter.name, "instances": expected.instances, "batches": expected.batches,
            "material_bindings_verified": true, "ground_traffic_masks_gpu_resident": true, "captured_views_ready": readiness.recorded,
        });
        std::fs::write(
            state.output.join("furniture-presentation.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .expect("write production furniture evidence");
    }
}
