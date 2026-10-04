//! Persistent browser renderer shared by strategic scenes and tactical play.
//!
//! JavaScript queues typed scene commands; Bevy retains strategic venues,
//! character rigs and camera views alongside the tactical plugin graph.
//! Changing modes preserves the Wasm application, WebGPU device and assets.

mod forge_view;
mod quote;
pub(crate) use quote::quote_design_json;
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

use adventuresim_tactical_netcode::prelude::AdventureSimulatorClient;
use adventuresim_weapon_model::{WeaponDesign, default_design, encode, generate};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use serde::Deserialize;

use crate::weapon_preview_material::preview_material;

use crate::{
    Args, player::LocalCharacterId, presentation::TacticalGameplayCamera, ui::TacticalUiRoot,
};

static COMMANDS: OnceLock<Mutex<VecDeque<BrowserCommand>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub(crate) enum BrowserMode {
    #[default]
    Strategic,
    Tactical,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum BrowserCommand {
    PrepareStrategicScene {
        location: String,
        input_json: String,
    },
    SyncStrategicView {
        view: crate::strategic_scene::protocol::StrategicView,
    },
    ShowStrategicScene {
        scene: StrategicScene,
    },
    OrbitForge {
        delta_x: f32,
        delta_y: f32,
    },
    ZoomForge {
        delta: f32,
    },
    HideForgePreview,
    EnterTactical {
        server_addr: String,
        character_id: u64,
    },
    ExitTactical,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum StrategicScene {
    Forge {
        #[serde(default = "default_catalog_id")]
        catalog_id: String,
        design_json: Option<String>,
    },
}

fn default_catalog_id() -> String {
    "longsword".into()
}

#[derive(Component)]
struct StrategicSceneEntity;

#[derive(Component)]
struct StrategicSceneRoot;

#[derive(Resource)]
struct ForgePreviewView {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for ForgePreviewView {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 1.75,
        }
    }
}

impl ForgePreviewView {
    fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * 0.008;
        self.pitch = (self.pitch + delta_y * 0.008).clamp(-1.2, 1.2);
    }

    fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta * 0.002).clamp(0.9, 4.5);
    }
}

pub(crate) struct BrowserRuntimePlugin {
    initial_mode: BrowserMode,
}

impl BrowserRuntimePlugin {
    pub(crate) fn new(initial_tactical: bool) -> Self {
        Self {
            initial_mode: if initial_tactical {
                BrowserMode::Tactical
            } else {
                BrowserMode::Strategic
            },
        }
    }
}

impl Plugin for BrowserRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::strategic_scene::StrategicScenePlugin);
        app.insert_resource(self.initial_mode)
            .init_resource::<ForgePreviewView>()
            .add_systems(Startup, forge_view::setup)
            .add_systems(Update, forge_view::sync)
            .add_systems(
                Update,
                (drain_browser_commands, sync_tactical_ui_visibility),
            )
            .add_systems(
                PostUpdate,
                sync_gameplay_view
                    .before(bevy::camera::CameraUpdateSystems)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

fn sync_gameplay_view(
    mode: Res<BrowserMode>,
    sky: Res<crate::presentation::AtmosphereIblCache>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
    view: Option<Res<crate::strategic_scene::protocol::StrategicView>>,
    mut cameras: Query<&mut Camera, With<TacticalGameplayCamera>>,
) {
    for mut camera in &mut cameras {
        if *mode == BrowserMode::Tactical {
            camera.is_active = true;
            camera.viewport = None;
            camera.sub_camera_view = None;
        } else {
            // The atmosphere bake validates submissions from this exact camera.
            // Render a masked one-pixel view until its shared environment map exists.
            camera.is_active = view.is_some() && !sky.is_ready(&settings);
            camera.viewport = Some(bevy::camera::Viewport {
                physical_size: UVec2::ONE,
                ..default()
            });
            camera.sub_camera_view = None;
        }
    }
}

pub(crate) fn queue_json(json: &str) -> Result<(), String> {
    let command = serde_json::from_str(json)
        .map_err(|error| format!("invalid browser renderer command: {error}"))?;
    COMMANDS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .map_err(|_| "browser renderer command queue is unavailable".to_string())?
        .push_back(command);
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "the browser command bridge coordinates the complete renderer state in one exclusive drain"
)]
fn drain_browser_commands(
    mut commands: Commands,
    mut mode: ResMut<BrowserMode>,
    mut args: ResMut<Args>,
    mut local_character: ResMut<LocalCharacterId>,
    scene_entities: Query<Entity, With<StrategicSceneEntity>>,
    clients: Query<Entity, With<AdventureSimulatorClient>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_roots: Query<&mut Transform, (With<StrategicSceneRoot>, Without<Camera3d>)>,
    mut preview_view: ResMut<ForgePreviewView>,
) {
    let pending = pending_commands();

    for command in pending {
        match command {
            BrowserCommand::PrepareStrategicScene {
                location,
                input_json,
            } => prepare_city(&mut commands, location, input_json),
            BrowserCommand::SyncStrategicView { view } => {
                commands.insert_resource(view);
            }
            BrowserCommand::ShowStrategicScene { scene } => {
                *mode = BrowserMode::Strategic;
                despawn_forge_preview(&mut commands, &scene_entities);
                match scene {
                    StrategicScene::Forge {
                        catalog_id,
                        design_json,
                    } => spawn_forge_scene(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &catalog_id,
                        design_json.as_deref(),
                        &preview_view,
                    ),
                }
            }
            BrowserCommand::OrbitForge { delta_x, delta_y } => {
                preview_view.orbit(delta_x, delta_y);
                for mut root in &mut scene_roots {
                    apply_preview_view(&preview_view, &mut root);
                }
            }
            BrowserCommand::ZoomForge { delta } => {
                preview_view.zoom(delta);
                for mut root in &mut scene_roots {
                    apply_preview_view(&preview_view, &mut root);
                }
            }
            BrowserCommand::HideForgePreview => {
                despawn_forge_preview(&mut commands, &scene_entities);
                *preview_view = ForgePreviewView::default();
            }
            BrowserCommand::EnterTactical {
                server_addr,
                character_id,
            } => {
                commands.queue(crate::strategic_scene::release_scene);
                commands.remove_resource::<crate::strategic_scene::protocol::StrategicView>();
                despawn_forge_preview(&mut commands, &scene_entities);
                if clients.is_empty() {
                    args.id = character_id;
                    args.server_addr.clone_from(&server_addr);
                    local_character.0 = character_id;
                    commands.spawn(AdventureSimulatorClient {
                        player_id: character_id,
                        server_url: server_addr,
                    });
                }
                *mode = BrowserMode::Tactical;
            }
            BrowserCommand::ExitTactical => {
                for entity in &clients {
                    commands.entity(entity).despawn();
                }
                *mode = BrowserMode::Strategic;
            }
        }
    }
}

fn pending_commands() -> Vec<BrowserCommand> {
    COMMANDS
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .map(|mut queue| queue.drain(..).collect())
        .unwrap_or_default()
}

fn despawn_forge_preview(
    commands: &mut Commands,
    scene_entities: &Query<Entity, With<StrategicSceneEntity>>,
) {
    for entity in scene_entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_forge_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    catalog_id: &str,
    design_json: Option<&str>,
    view: &ForgePreviewView,
) {
    let design = match design_json {
        Some(json) => match serde_json::from_str::<WeaponDesign>(json) {
            Ok(design) => Some(design),
            Err(error) => {
                warn!(catalog_id, %error, "browser forge preview rejected invalid recipe");
                return;
            }
        },
        None => default_design(catalog_id),
    };
    let Some(design) = design else {
        warn!(
            catalog_id,
            "browser forge preview rejected unknown weapon chassis"
        );
        return;
    };
    let Ok(generated) = generate(&design) else {
        warn!(
            catalog_id,
            "browser forge preview failed to generate weapon mesh"
        );
        return;
    };

    let center =
        (Vec3::from_array(generated.bounds.min) + Vec3::from_array(generated.bounds.max)) * 0.5;
    let extent = Vec3::from_array(generated.bounds.max) - Vec3::from_array(generated.bounds.min);
    let scale = 1.45 / extent.max_element().max(0.001);
    let root = commands
        .spawn((
            Name::new("Strategic forge weapon preview"),
            Visibility::default(),
            StrategicSceneEntity,
            StrategicSceneRoot,
            preview_transform(view, scale),
        ))
        .id();

    for part in generated.parts {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals);
        mesh.insert_indices(Indices::U32(part.indices));
        commands.spawn((
            Name::new(format!("Strategic forge part {}", part.component_id)),
            Mesh3d(meshes.add(mesh)),
            bevy::camera::visibility::RenderLayers::layer(
                crate::strategic_scene::protocol::FORGE_LAYER,
            ),
            MeshMaterial3d(materials.add(preview_material(part.material))),
            Transform::from_translation(-center),
            ChildOf(root),
        ));
    }

    commands.spawn((
        Name::new("Strategic forge preview light"),
        bevy::camera::visibility::RenderLayers::layer(
            crate::strategic_scene::protocol::FORGE_LAYER,
        ),
        StrategicSceneEntity,
        DirectionalLight {
            illuminance: 60_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 4.0, 2.0).looking_at(Vec3::new(0.0, 0.0, -2.2), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Strategic forge preview fill light"),
        bevy::camera::visibility::RenderLayers::layer(
            crate::strategic_scene::protocol::FORGE_LAYER,
        ),
        StrategicSceneEntity,
        PointLight {
            intensity: 12_000.0,
            range: 8.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.0, 1.5, 0.5),
    ));
}

fn preview_transform(view: &ForgePreviewView, scale: f32) -> Transform {
    Transform::from_xyz(0.0, 0.0, -view.distance)
        .with_rotation(Quat::from_euler(EulerRot::YXZ, view.yaw, view.pitch, -0.55))
        .with_scale(Vec3::splat(scale))
}

fn apply_preview_view(view: &ForgePreviewView, transform: &mut Transform) {
    let scale = transform.scale.x;
    *transform = preview_transform(view, scale);
}

pub(crate) fn default_design_json(catalog_id: &str) -> Result<String, String> {
    let design = default_design(catalog_id).ok_or("unknown melee weapon chassis")?;
    serde_json::to_string(&design).map_err(|error| error.to_string())
}

pub(crate) fn catalog_json() -> Result<String, String> {
    serde_json::to_string(adventuresim_weapon_model::MELEE_CATALOG_IDS)
        .map_err(|error| error.to_string())
}

pub(crate) fn encode_design_json(json: &str) -> Result<Vec<u8>, String> {
    let design: WeaponDesign = serde_json::from_str(json).map_err(|error| error.to_string())?;
    encode(&design).map_err(|error| error.to_string())
}

pub(crate) fn editor_fields_json(json: &str) -> Result<String, String> {
    let design: WeaponDesign = serde_json::from_str(json).map_err(|error| error.to_string())?;
    serde_json::to_string(&adventuresim_weapon_model::editor_fields(&design))
        .map_err(|error| error.to_string())
}

fn sync_tactical_ui_visibility(
    mode: Res<BrowserMode>,
    mut roots: Query<&mut Visibility, With<TacticalUiRoot>>,
) {
    let visibility = if *mode == BrowserMode::Tactical {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut current in &mut roots {
        *current = visibility;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_commands_are_closed_and_typed() {
        let forge: BrowserCommand =
            serde_json::from_str(r#"{"type":"show-strategic-scene","scene":{"type":"forge"}}"#)
                .unwrap();
        assert!(matches!(
            forge,
            BrowserCommand::ShowStrategicScene {
                scene: StrategicScene::Forge { catalog_id, .. }
            } if catalog_id == "longsword"
        ));
        assert!(serde_json::from_str::<BrowserCommand>(r#"{"type":"unknown"}"#).is_err());
        assert!(matches!(
            serde_json::from_str::<BrowserCommand>(
                r#"{"type":"orbit-forge","delta_x":4.0,"delta_y":-2.0}"#
            )
            .unwrap(),
            BrowserCommand::OrbitForge { .. }
        ));
        assert!(matches!(
            serde_json::from_str::<BrowserCommand>(r#"{"type":"zoom-forge","delta":12.0}"#)
                .unwrap(),
            BrowserCommand::ZoomForge { .. }
        ));
    }

    #[test]
    fn forge_view_is_manual_and_bounded() {
        let mut view = ForgePreviewView::default();
        view.orbit(20.0, 10_000.0);
        assert!(view.yaw > 0.0);
        assert_eq!(view.pitch, 1.2);
        view.zoom(-10_000.0);
        assert_eq!(view.distance, 0.9);
        view.zoom(10_000.0);
        assert_eq!(view.distance, 4.5);
    }
}

fn prepare_city(commands: &mut Commands, location: String, input_json: String) {
    commands.insert_resource(crate::strategic_scene::SceneDocument::parse(
        location,
        &input_json,
    ));
}
