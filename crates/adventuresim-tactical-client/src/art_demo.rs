//! Fixed exhibits composed from shipped equipment and production generators.

mod camera;
mod district;
mod exhibits;
mod scenery;

use std::sync::Mutex;

use adventuresim_tactical_core::physics::AdventureSimulatorPhysicsPlugin;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::presentation::{TacticalCameraSetup, TacticalPresentationPlugin};
use camera::OrbitView;
use exhibits::{Exhibit, ExhibitId};

static PENDING: Mutex<Vec<DemoCommand>> = Mutex::new(Vec::new());
static STATUS: Mutex<DemoStatus> = Mutex::new(DemoStatus::Starting);

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
enum DemoCommand {
    Show {
        exhibit: ExhibitId,
    },
    Orbit {
        delta_x: f32,
        delta_y: f32,
    },
    Zoom {
        delta: f32,
    },
    Pan {
        right: f32,
        forward: f32,
        seconds: f32,
    },
}

#[derive(Clone, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
enum DemoStatus {
    Starting,
    Loading {
        exhibit: ExhibitId,
        completed: usize,
        total: usize,
    },
    Ready {
        exhibit: ExhibitId,
    },
    Failed {
        message: String,
    },
}

#[derive(Component)]
struct DemoEntity;

#[derive(Resource)]
struct CurrentExhibit {
    id: ExhibitId,
    scene: Option<Handle<WorldAsset>>,
    frames: u32,
}

#[derive(Resource)]
struct PendingExhibit {
    id: ExhibitId,
    retire_frames: u8,
}

pub(super) fn queue(json: &str) -> Result<(), String> {
    let command: DemoCommand = serde_json::from_str(json).map_err(|error| error.to_string())?;
    match command {
        DemoCommand::Orbit { delta_x, delta_y } if !delta_x.is_finite() || !delta_y.is_finite() => {
            return Err("orbit requires finite deltas".into());
        }
        DemoCommand::Zoom { delta } if !delta.is_finite() => {
            return Err("zoom requires a finite delta".into());
        }
        DemoCommand::Pan {
            right,
            forward,
            seconds,
        } if !right.is_finite() || !forward.is_finite() || !seconds.is_finite() => {
            return Err("pan requires finite inputs".into());
        }
        _ => {}
    }
    let mut pending = PENDING.lock().map_err(|error| error.to_string())?;
    if matches!(command, DemoCommand::Show { .. }) {
        pending.clear();
    }
    pending.push(command);
    Ok(())
}

pub(super) fn status() -> String {
    serde_json::to_string(&*STATUS.lock().expect("demo status lock")).expect("demo status JSON")
}

pub(super) fn renderer_failed() {
    if let Ok(mut status) = STATUS.lock() {
        *status = DemoStatus::Failed {
            message:
                "The 3D renderer stopped. Reload to retry; details are in the browser console."
                    .into(),
        };
    }
}

fn report_exit(mut exits: MessageReader<AppExit>) {
    if exits.read().next().is_some() {
        renderer_failed();
    }
}

pub(super) fn run() {
    let mut app = App::new();
    let mut presentation = TacticalPresentationPlugin::default();
    // WebGPU guarantees four samples, but does not support the desktop two-sample preset.
    const WEBGPU_MSAA_SAMPLES: u8 = 4;
    presentation.config.rendering.anti_aliasing = crate::presentation::AntiAliasingConfig::Msaa {
        samples: WEBGPU_MSAA_SAMPLES,
    };
    let mut window = Window {
        title: "Fabelgeist · Procedural art".into(),
        resolution: (1280, 800).into(),
        ..default()
    };
    let mut assets = AssetPlugin::default();
    #[cfg(target_family = "wasm")]
    {
        window.canvas = Some("#game-canvas".into());
        window.fit_canvas_to_parent = true;
        assets.file_path = "/tactical/assets".into();
        assets.meta_check = bevy::asset::AssetMetaCheck::Never;
    }
    #[cfg(not(target_family = "wasm"))]
    {
        assets.file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets")
            .to_string_lossy()
            .into_owned();
        window.present_mode = bevy::window::PresentMode::AutoVsync;
    }
    app.add_plugins(DefaultPlugins.set(assets).set(WindowPlugin {
        primary_window: Some(window),
        ..default()
    }))
    .add_plugins(AdventureSimulatorPhysicsPlugin {
        enable_simulation: false,
        enable_presentation_simulation: false,
    })
    .add_plugins(presentation)
    .insert_resource(TacticalCameraSetup {
        vertical_fov_degrees: camera::VERTICAL_FOV_DEGREES,
        ..default()
    })
    .init_resource::<OrbitView>()
    .insert_resource(ClearColor(Color::srgb_u8(23, 27, 29)))
    .add_systems(PreUpdate, (drain, spawn_pending).chain())
    .add_systems(
        PostUpdate,
        (camera::apply, report_readiness)
            .chain()
            .before(bevy::transform::TransformSystems::Propagate),
    )
    .add_systems(Last, (camera::studio_exposure, report_exit));
    #[cfg(feature = "debug")]
    app.insert_gizmo_config(
        adventuresim_tactical_core::prelude::PhysicsGizmos::default(),
        GizmoConfig {
            enabled: false,
            ..default()
        },
    );
    #[cfg(not(target_family = "wasm"))]
    app.add_systems(Update, camera::native_input);
    PENDING
        .lock()
        .expect("demo command lock")
        .push(DemoCommand::Show {
            exhibit: ExhibitId::Henry,
        });
    app.run();
}

fn drain(world: &mut World) {
    let pending = std::mem::take(&mut *PENDING.lock().expect("demo command lock"));
    for command in pending {
        match command {
            DemoCommand::Show { exhibit } => {
                *STATUS.lock().expect("demo status lock") = DemoStatus::Loading {
                    exhibit,
                    completed: 0,
                    total: 0,
                };
                retire(world, exhibit);
            }
            DemoCommand::Orbit { delta_x, delta_y } => {
                world.resource_mut::<OrbitView>().orbit(delta_x, delta_y);
            }
            DemoCommand::Zoom { delta } => world.resource_mut::<OrbitView>().zoom(delta),
            DemoCommand::Pan {
                right,
                forward,
                seconds,
            } => {
                world
                    .resource_mut::<OrbitView>()
                    .pan(right, forward, seconds);
            }
        }
    }
}

fn retire(world: &mut World, id: ExhibitId) {
    world.remove_resource::<CurrentExhibit>();
    crate::presentation::clear_demo_scene(world);
    let entities = world
        .query_filtered::<Entity, With<DemoEntity>>()
        .iter(world)
        .collect::<Vec<_>>();
    for entity in entities {
        if let Ok(entity) = world.get_entity_mut(entity) {
            entity.despawn();
        }
    }
    // Asset handle drops and render-world removals are processed across frames.
    // Finish those before allocating another scenery exhibit's meshes and masks.
    const ASSET_RETIRE_FRAMES: u8 = 4;
    world.insert_resource(PendingExhibit {
        id,
        retire_frames: ASSET_RETIRE_FRAMES,
    });
}

fn spawn_pending(world: &mut World) {
    let Some(mut pending) = world.get_resource_mut::<PendingExhibit>() else {
        return;
    };
    if pending.retire_frames > 0 {
        pending.retire_frames -= 1;
        return;
    }
    let id = pending.id;
    world.remove_resource::<PendingExhibit>();
    if let Err(message) = show(world, id) {
        *STATUS.lock().expect("demo status lock") = DemoStatus::Failed { message };
    }
}

fn show(world: &mut World, id: ExhibitId) -> Result<(), String> {
    let exhibit = Exhibit::get(id);
    *world.resource_mut::<OrbitView>() = exhibit.view();
    let scene = exhibit.spawn(world)?;
    world.flush();
    world.insert_resource(CurrentExhibit {
        id,
        scene,
        frames: 0,
    });
    Ok(())
}

fn report_readiness(
    current: Option<ResMut<CurrentExhibit>>,
    assets: Res<AssetServer>,
    pending: Option<Res<crate::presentation::PendingCityBuildings>>,
) {
    let Some(mut current) = current else {
        return;
    };
    if let Some(pending) = pending {
        *STATUS.lock().expect("demo status lock") = DemoStatus::Loading {
            exhibit: current.id,
            completed: pending.completed(),
            total: pending.total,
        };
        return;
    }
    if let Some(scene) = &current.scene {
        if let Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) =
            assets.get_recursive_dependency_load_state(scene.id())
        {
            *STATUS.lock().expect("demo status lock") = DemoStatus::Failed {
                message: error.to_string(),
            };
            return;
        }
        if let Some(bevy::asset::LoadState::Failed(error)) = assets.get_load_state(scene.id()) {
            *STATUS.lock().expect("demo status lock") = DemoStatus::Failed {
                message: error.to_string(),
            };
            return;
        }
        if !assets.is_loaded_with_dependencies(scene.id()) {
            return;
        }
    }
    current.frames += 1;
    // Allow scene instantiation and extraction before dismissing the loading plate.
    const PRESENTATION_SETTLE_FRAMES: u32 = 12;
    if current.frames == PRESENTATION_SETTLE_FRAMES {
        *STATUS.lock().expect("demo status lock") = DemoStatus::Ready {
            exhibit: current.id,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_rejects_unknown_assets_and_keeps_latest_navigation() {
        assert!(queue(r#"{"type":"show","exhibit":"../../secret"}"#).is_err());
        assert!(queue(r#"{"type":"zoom","delta":1e300}"#).is_err());
        queue(r#"{"type":"show","exhibit":"city"}"#).unwrap();
        queue(r#"{"type":"orbit","delta_x":20,"delta_y":0}"#).unwrap();
        queue(r#"{"type":"show","exhibit":"oak"}"#).unwrap();
        let pending = std::mem::take(&mut *PENDING.lock().unwrap());
        assert!(matches!(
            pending.as_slice(),
            [DemoCommand::Show {
                exhibit: ExhibitId::Oak
            }]
        ));
    }
}
