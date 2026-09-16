//! Fixed exhibits composed from shipped equipment and production generators.

mod camera;
mod district;
mod exhibits;
mod residency;
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
    Prefetch {
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
    Unavailable {
        exhibit: ExhibitId,
        message: String,
    },
    Failed {
        message: String,
    },
}

#[derive(Component)]
struct DemoEntity(ExhibitId);

#[derive(Component)]
struct StudioEntity;

#[derive(Resource)]
struct CurrentExhibit {
    id: ExhibitId,
    scene: Result<Option<Handle<WorldAsset>>, String>,
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

fn render_error(
    _error: &bevy::render::error_handler::RenderError,
    world: &mut World,
    _render_world: &mut World,
) -> bevy::render::error_handler::RenderErrorPolicy {
    // Render errors arrive after Last. Publish the terminal status here, before
    // Bevy's runner exits and can no longer run report_exit or readiness systems.
    renderer_failed();
    world.write_message(AppExit::error());
    bevy::render::error_handler::RenderErrorPolicy::StopRendering
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
    .insert_resource(bevy::render::error_handler::RenderErrorHandler(
        render_error,
    ))
    .insert_resource(TacticalCameraSetup {
        vertical_fov_degrees: camera::VERTICAL_FOV_DEGREES,
        ..default()
    })
    .init_resource::<OrbitView>()
    .init_resource::<residency::ExhibitCache>()
    .insert_resource(ClearColor(Color::srgb_u8(23, 27, 29)))
    .add_systems(PreUpdate, (drain, residency::spawn_pending).chain())
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
                residency::show(world, exhibit);
            }
            DemoCommand::Prefetch { exhibit } => residency::prefetch(world, exhibit),
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

fn report_readiness(
    current: Option<Res<CurrentExhibit>>,
    assets: Res<AssetServer>,
    pending: Option<Res<crate::presentation::PendingCityBuildings>>,
    scenery: Option<Res<residency::PendingScenery>>,
) {
    let Some(current) = current else {
        return;
    };
    let mut status = DemoStatus::Ready {
        exhibit: current.id,
    };
    if scenery.is_some() {
        status = DemoStatus::Loading {
            exhibit: current.id,
            completed: 0,
            total: 0,
        };
    }
    if let Some(pending) = pending {
        if !pending.finished() {
            status = DemoStatus::Loading {
                exhibit: current.id,
                completed: pending.completed(),
                total: pending.total,
            };
        } else if let Some(message) = pending.failure() {
            status = DemoStatus::Unavailable {
                exhibit: current.id,
                message: message.into(),
            };
        }
    }
    match &current.scene {
        Err(message) => {
            status = DemoStatus::Unavailable {
                exhibit: current.id,
                message: message.clone(),
            }
        }
        Ok(Some(scene)) => {
            let error = match assets.get_load_state(scene.id()) {
                Some(bevy::asset::LoadState::Failed(error)) => Some(error),
                _ => match assets.get_recursive_dependency_load_state(scene.id()) {
                    Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) => Some(error),
                    _ => None,
                },
            };
            if let Some(error) = error {
                status = DemoStatus::Unavailable {
                    exhibit: current.id,
                    message: error.to_string(),
                };
            } else if !assets.is_loaded_with_dependencies(scene.id()) {
                status = DemoStatus::Loading {
                    exhibit: current.id,
                    completed: 0,
                    total: 0,
                };
            }
        }
        Ok(None) => {}
    }
    *STATUS.lock().expect("demo status lock") = status;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_failure_reports_terminal_status_before_the_runner_exits() {
        let mut app = App::new();
        let error = bevy::render::error_handler::RenderError {
            ty: bevy::render::error_handler::ErrorType::Validation,
            description: "test validation failure".into(),
            source: None,
        };
        render_error(&error, app.world_mut(), &mut World::new());
        assert!(matches!(*STATUS.lock().unwrap(), DemoStatus::Failed { .. }));
        assert!(app.should_exit().is_some());
    }

    #[test]
    fn boundary_rejects_unknown_assets_and_keeps_latest_navigation() {
        assert!(queue(r#"{"type":"show","exhibit":"../../secret"}"#).is_err());
        assert!(queue(r#"{"type":"zoom","delta":1e300}"#).is_err());
        queue(r#"{"type":"show","exhibit":"city"}"#).unwrap();
        queue(r#"{"type":"orbit","delta_x":20,"delta_y":0}"#).unwrap();
        queue(r#"{"type":"prefetch","exhibit":"nuremberg"}"#).unwrap();
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
