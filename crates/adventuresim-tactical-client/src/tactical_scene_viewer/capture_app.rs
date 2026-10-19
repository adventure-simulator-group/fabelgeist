//! Native capture surface and production renderer assembly.

use std::{path::PathBuf, time::Duration};

use adventuresim_tactical_core::physics::AdventureSimulatorPhysicsPlugin;
use bevy::{
    app::ScheduleRunnerPlugin,
    prelude::*,
    window::{ExitCondition, PresentMode},
    winit::WinitPlugin,
};

use super::{capture_presentation_plugin, capture_resolution};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CaptureSurface {
    Window,
    Offscreen,
}

pub(super) fn new(asset_root: PathBuf, profile: &str, surface: CaptureSurface) -> App {
    let scene_performance_benchmarking = surface == CaptureSurface::Offscreen;
    let mut app = App::new();
    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: (!scene_performance_benchmarking).then(|| Window {
                visible: false,
                resolution: capture_resolution::window_resolution(profile),
                present_mode: PresentMode::AutoNoVsync,
                resizable: false,
                decorations: false,
                ..default()
            }),
            exit_condition: if scene_performance_benchmarking {
                ExitCondition::DontExit
            } else {
                ExitCondition::OnAllClosed
            },
            ..default()
        });
    if scene_performance_benchmarking {
        // Eliminate both swapchain presentation and the desktop event-loop
        // floor. The normal RenderPlugin still renders the production camera
        // to the offscreen texture; only frame scheduling becomes unpaced.
        app.add_plugins(default_plugins.disable::<WinitPlugin>())
            .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO));
    } else {
        app.add_plugins(default_plugins)
            .insert_resource(bevy::winit::WinitSettings::continuous());
    }
    app.add_plugins(AdventureSimulatorPhysicsPlugin {
        enable_simulation: false,
        enable_presentation_simulation: false,
    })
    // Visual-review plates use the exact production presentation defaults.
    // Diagnostics may hide named occluder layers, but never substitute a
    // cheaper lighting or post-processing pipeline.
    .add_plugins(capture_presentation_plugin())
    .insert_resource(ClearColor(Color::srgb_u8(158, 181, 195)));
    app
}
