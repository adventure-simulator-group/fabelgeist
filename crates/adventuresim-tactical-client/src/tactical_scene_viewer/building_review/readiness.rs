//! Observe production output; never create replacement render assets here.
use super::*;
use crate::presentation::{
    PresentedBuildingMesh, PresentedSign, ProceduralTextureAssets, TacticalBuildingMaterials,
    TacticalGraphicsSettings,
};
use crate::tactical_scene_viewer::capture_state::{CapturePhase, SceneCaptureState};
use adventuresim_building_generator::BuildingLodMaterial;
use adventuresim_tactical_core::prelude::{SceneBuilding, SceneDoor, SceneWindow};
use bevy::{ecs::system::SystemParam, render::renderer::RenderAdapterInfo};

const MAX_ASSET_WAIT_SECONDS: f64 = 120.0;

pub(in crate::tactical_scene_viewer) struct BuildingReviewPlugin;
impl Plugin for BuildingReviewPlugin {
    fn build(&self, app: &mut App) {
        crate::tactical_scene_viewer::gpu_readiness::GpuReadiness::install(app);
        app.init_resource::<Readiness>().add_systems(
            Last,
            observe.before(crate::tactical_scene_viewer::capture_views),
        );
    }
}

#[derive(Resource, Default)]
pub(in crate::tactical_scene_viewer) struct Readiness {
    ready: bool,
    pending_since: Option<f64>,
    recorded: std::collections::BTreeSet<usize>,
}

pub(in crate::tactical_scene_viewer) fn ready(
    state: Option<Res<SceneCaptureState>>,
    requirements: Option<Res<ReviewRequirements>>,
    readiness: Option<Res<Readiness>>,
) -> bool {
    requirements.is_none()
        || state.is_none_or(|state| state.phase == CapturePhase::Configure)
        || readiness.is_some_and(|readiness| readiness.ready)
}

#[derive(SystemParam)]
struct Observation<'w, 's> {
    buildings: Query<'w, 's, &'static SceneBuilding>,
    batches: Query<
        'w,
        's,
        (
            &'static PresentedBuildingMesh,
            &'static ChildOf,
            &'static Mesh3d,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    doors: Query<
        'w,
        's,
        (
            &'static SceneDoor,
            &'static Mesh3d,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    windows: Query<
        'w,
        's,
        (
            &'static SceneWindow,
            &'static Mesh3d,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
    >,
    signs: Query<
        'w,
        's,
        (
            &'static PresentedSign,
            &'static ChildOf,
            &'static GlobalTransform,
        ),
    >,
    parts: Query<
        'w,
        's,
        (
            &'static Mesh3d,
            &'static MeshMaterial3d<StandardMaterial>,
            &'static ChildOf,
        ),
    >,
    parents: Query<'w, 's, &'static ChildOf>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<Camera3d>>,
    palette: Res<'w, TacticalBuildingMaterials>,
    textures: Res<'w, ProceduralTextureAssets>,
    materials: Res<'w, Assets<StandardMaterial>>,
    gpu: Res<'w, crate::tactical_scene_viewer::gpu_readiness::GpuReadiness>,
    adapter: Res<'w, RenderAdapterInfo>,
    graphics: Res<'w, TacticalGraphicsSettings>,
}

impl Observation<'_, '_> {
    fn assets_ready(&self, mesh: &Mesh3d, handle: &MeshMaterial3d<StandardMaterial>) -> bool {
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
    }

    fn check(&self, requirements: &ReviewRequirements, fixture: &ReviewFixture) -> bool {
        if self.buildings.iter().count() != requirements.buildings
            || self.doors.iter().count() != requirements.doors
            || self.windows.iter().count() != requirements.windows
        {
            return false;
        }
        if !self
            .batches
            .iter()
            .all(|(_, _, mesh, material)| self.assets_ready(mesh, material))
            || !self
                .doors
                .iter()
                .all(|(_, mesh, material)| self.assets_ready(mesh, material))
            || !self
                .windows
                .iter()
                .all(|(_, mesh, material)| self.assets_ready(mesh, material))
        {
            return false;
        }
        for (mesh, material, parent) in &self.parts {
            let is_sign = self.signs.contains(parent.parent())
                || self
                    .parents
                    .get(parent.parent())
                    .is_ok_and(|grandparent| self.signs.contains(grandparent.parent()));
            if is_sign && !self.assets_ready(mesh, material) {
                return false;
            }
        }
        let mut rendered_buildings = BTreeMap::<u64, std::collections::HashSet<_>>::new();
        for (batch, parent, _, material) in &self.batches {
            let Ok(building) = self.buildings.get(parent.parent()) else {
                continue;
            };
            assert_eq!(
                material.0,
                self.palette.get_for_building(building.id, batch.material),
                "capture bypassed production building material binding"
            );
            rendered_buildings
                .entry(building.id)
                .or_default()
                .insert(std::mem::discriminant(&batch.level));
        }
        if rendered_buildings.len() != requirements.buildings
            || rendered_buildings.values().any(|levels| levels.len() != 3)
        {
            return false;
        }
        for (window, _, material) in &self.windows {
            assert_eq!(
                material.0,
                self.palette
                    .get_for_building(window.building_id, BuildingLodMaterial::Glass)
            );
            let glass = self
                .materials
                .get(&material.0)
                .expect("ready glass material");
            assert_eq!(
                glass.base_color_texture.as_ref(),
                Some(&self.textures.window_glass.transmittance),
                "window must use production glass, never a masonry substitute"
            );
            assert!(
                glass.specular_transmission > 0.0,
                "glass must retain physical transmission"
            );
        }
        for (door, _, material) in &self.doors {
            assert_eq!(
                material.0,
                self.palette
                    .get_for_building(door.building_id, BuildingLodMaterial::Timber)
            );
        }
        for (&id, expected) in &fixture.signs {
            let Some((presented, _, transform)) = self.signs.iter().find(|(_, parent, _)| {
                self.buildings
                    .get(parent.parent())
                    .is_ok_and(|building| building.id == id)
            }) else {
                return false;
            };
            assert_eq!(presented.sign.name, expected.name);
            assert_eq!(presented.sign.font, expected.font);
            assert_eq!(presented.sign.mount, expected.mount);
            assert_eq!(presented.sign.finish, expected.finish);
            assert_eq!(
                presented.site.mounting.contact,
                requirements.sites[&id].mounting.contact
            );
            // Nearby lettering is streamed by production. The fixture check uses the
            // actual root placement, including orientation, in the saved camera inputs.
            let view_is_close = self.cameras.iter().any(|(camera, pose)| {
                camera.is_active && pose.translation().distance(transform.translation()) < 35.0
            });
            if view_is_close && presented.lettering.is_none() {
                return false;
            }
        }
        true
    }
}

fn observe(
    mut readiness: ResMut<Readiness>,
    requirements: Option<Res<ReviewRequirements>>,
    fixture: Option<Res<ReviewFixture>>,
    state: Option<Res<SceneCaptureState>>,
    time: Res<Time<Real>>,
    observation: Observation,
) {
    let (Some(requirements), Some(fixture), Some(state)) = (requirements, fixture, state) else {
        return;
    };
    if state.phase == CapturePhase::Configure {
        readiness.ready = false;
        readiness.pending_since = None;
        return;
    }
    readiness.ready = observation.check(&requirements, &fixture);
    if !readiness.ready {
        let since = *readiness
            .pending_since
            .get_or_insert(time.elapsed_secs_f64());
        assert!(
            time.elapsed_secs_f64() - since < MAX_ASSET_WAIT_SECONDS,
            "building review timed out waiting for production geometry, textures or lettering"
        );
        return;
    }
    if readiness.recorded.insert(state.view) {
        let evidence = serde_json::json!({
            "renderer": "TacticalPresentationPlugin", "backend": format!("{:?}", observation.adapter.backend),
            "adapter": observation.adapter.name, "graphics": format!("{:?}", observation.graphics.config),
            "buildings": requirements.buildings, "doors": requirements.doors, "windows": requirements.windows,
            "supported_signs": requirements.sites.keys().collect::<Vec<_>>(),
            "material_bindings_verified": true, "lod_levels_per_building": 3, "captured_views_ready": readiness.recorded,
        });
        std::fs::write(
            requirements.output.join("building-presentation.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .expect("write production presentation evidence");
    }
}
