//! Bounded readiness and timing telemetry consumed by the browser benchmark.
use super::{RetainedScene, SceneModel, protocol::StrategicView};
use crate::presentation::ownership::PresentationOwner;
use adventuresim_procedural_textures::{PROCEDURAL_TEXTURE_CATALOGUE, ProceduralTextureResidency};
use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems, render_resource::PipelineCache},
};
use serde::Serialize;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicUsize, Ordering},
};

static STATUS: OnceLock<Mutex<String>> = OnceLock::new();
static WAITING_PIPELINES: AtomicUsize = AtomicUsize::new(0);
const SETTLED_RENDER_FRAMES: usize = 4;

#[derive(Resource, Default)]
pub(super) struct SceneAssetsReady(pub bool);

/// CPU installation and final lighting, independent of mesh pipeline preparation.
#[derive(Resource, Default)]
pub(super) struct SceneInstallationReady(pub bool);

#[derive(Default, Serialize)]
struct Status {
    revision: u64,
    ready: bool,
    assets_ready: bool,
    installed_ready: bool,
    snapshots_pending: usize,
    settled_render_frames: usize,
    buildings: usize,
    street: Option<super::street::Street>,
    characters: usize,
    loaded_characters: usize,
    posed_characters: usize,
    textures_ready: bool,
    sky_ready: bool,
    terrain_pending: usize,
    city_pending: usize,
    waiting_pipelines: usize,
    equipment_ready: bool,
    equipment_pending: Option<String>,
    daylight: Vec4,
    view_lighting: Vec<super::lighting_diagnostics::ViewLighting>,
    error: Option<String>,
}

impl Status {
    fn publish(self) {
        *STATUS
            .get_or_init(|| Mutex::new(String::new()))
            .lock()
            .expect("renderer status lock") =
            serde_json::to_string(&self).expect("renderer status serializes");
    }
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<SceneAssetsReady>()
        .init_resource::<SceneInstallationReady>()
        .init_resource::<SceneAssetFailure>()
        .add_systems(
            Last,
            (
                capture_asset_failures,
                update_status,
                super::render_diagnostics::census,
            )
                .chain(),
        );
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(Render, pipeline_status.after(RenderSystems::Prepare));
    }
}

#[derive(Resource, Default)]
pub(super) struct SceneAssetFailure(Option<String>);

fn capture_asset_failures(
    mut events: MessageReader<bevy::asset::UntypedAssetLoadFailedEvent>,
    mut failure: ResMut<SceneAssetFailure>,
) {
    for event in events.read() {
        let path = event.path.path();
        if path.starts_with("textures/procedural")
            || path.ends_with("unarmed/base.glb")
            || path.ends_with("unarmed/idle_relaxed.glb")
        {
            failure.0 = Some(format!("Could not load scene asset {}", path.display()));
        }
    }
}

fn pipeline_status(cache: Res<PipelineCache>) {
    WAITING_PIPELINES.store(cache.waiting_pipelines().count(), Ordering::Relaxed);
}

/// Shared render-device telemetry, independent of actor installation readiness.
pub(crate) fn waiting_pipelines() -> usize {
    WAITING_PIPELINES.load(Ordering::Relaxed)
}

pub(crate) fn json() -> String {
    STATUS
        .get_or_init(|| Mutex::new("{\"ready\":false}".into()))
        .lock()
        .expect("renderer status lock")
        .clone()
}

#[expect(
    clippy::too_many_arguments,
    reason = "Readiness joins assets, rigs, equipment and render pipeline state"
)]
pub(super) fn update_status(
    view: Option<Res<StrategicView>>,
    scene: Res<RetainedScene>,
    models: Query<(), (With<SceneModel>, With<crate::animation::HumanoidRig>)>,
    poses: Query<&crate::animation::pose_buffer::PoseBufferRig, With<SceneModel>>,
    textures: Res<ProceduralTextureResidency>,
    failure: Res<SceneAssetFailure>,
    equipment: super::equipment_readiness::EquipmentReadiness,
    items: Query<Entity, With<super::equipment::SceneEquipment>>,
    mut settled: Local<usize>,
    (mut assets_ready, mut installed_ready): (
        ResMut<SceneAssetsReady>,
        ResMut<SceneInstallationReady>,
    ),
    cache: Res<super::cached_views::CachedViews>,
    city: Option<Res<crate::presentation::PendingCityBuildings>>,
    terrain: Query<(), With<crate::presentation::PendingTerrainPresentation>>,
    sky: Res<crate::presentation::AtmosphereIblCache>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
    lighting: super::lighting_diagnostics::LightingDiagnostics,
) {
    let waiting = WAITING_PIPELINES.load(Ordering::Relaxed);
    let equipment_state = equipment.check(items.iter());
    let equipment_ready = equipment_state == Ok(true);
    let loaded = models.iter().count();
    let posed = poses
        .iter()
        .filter(|pose| pose.has_presented_pose())
        .count();
    let textures_ready =
        textures.is_ready(PROCEDURAL_TEXTURE_CATALOGUE.iter().map(|recipe| recipe.id));
    if view.as_ref().is_some_and(|view| view.is_changed()) {
        *settled = 0;
    }
    let installed = view.as_ref().is_some_and(|view| {
        scene.location == view.location
            && scene.next_place == view.places.len()
            && scene.pending.is_empty()
            && view.people.iter().all(|p| scene.people.contains_key(&p.id))
    }) && loaded == scene.people.len()
        && posed == loaded
        && equipment_ready
        && textures_ready
        && city.as_ref().is_none_or(|city| city.finished())
        && terrain.is_empty()
        && sky.is_ready(&settings)
        && failure.0.is_none()
        && scene.error.is_none();
    installed_ready.0 = installed;
    let complete = installed
        && waiting == 0
        && city.as_ref().is_none_or(|city| {
            city.total == 0 || crate::presentation::city_gpu_ready(PresentationOwner::Scene)
        });
    assets_ready.0 = complete;
    if complete && cache.is_ready() {
        *settled += 1;
    } else {
        *settled = 0;
    }
    let status = Status {
        daylight: lighting.gpu.daylight(),
        view_lighting: lighting.views(),
        revision: view.as_ref().map_or(0, |view| view.revision),
        ready: *settled >= SETTLED_RENDER_FRAMES,
        assets_ready: complete,
        installed_ready: installed,
        snapshots_pending: cache.pending_captures(),
        settled_render_frames: *settled,
        buildings: scene.next_place,
        street: scene.street.clone(),
        characters: scene.people.len(),
        loaded_characters: loaded,
        posed_characters: posed,
        textures_ready,
        sky_ready: sky.is_ready(&settings),
        terrain_pending: terrain.iter().count(),
        city_pending: city
            .as_ref()
            .map_or(0, |city| city.total - city.completed()),
        waiting_pipelines: waiting,
        equipment_ready,
        equipment_pending: (!equipment_ready).then(|| equipment.pending_summary(items.iter())),
        error: scene
            .error
            .clone()
            .or_else(|| failure.0.clone())
            .or_else(|| {
                city.as_ref()
                    .and_then(|city| city.failure().map(str::to_owned))
            })
            .or_else(|| equipment_state.err().map(str::to_owned)),
    };
    status.publish();
}
