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
#[cfg(feature = "grown-review")]
use bevy::ecs::schedule::common_conditions::not;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CaptureSurface {
    Window,
    Offscreen,
}

pub(super) enum CaptureWorkload {
    TerrainWireframe,
    TriangleCensus,
    LeafBenchmark,
    TreeLightingBenchmark,
    ScenePerformanceBenchmark,
    Review,
}

pub(super) fn install_capture(app: &mut App, workload: CaptureWorkload) {
    match workload {
        CaptureWorkload::TerrainWireframe => {
            app.add_systems(Last, super::capture_terrain_wireframe);
        }
        CaptureWorkload::TriangleCensus => {
            app.add_systems(Last, super::capture_triangle_census);
        }
        CaptureWorkload::LeafBenchmark => {
            app.add_systems(Last, super::benchmark_leaf_representations);
        }
        CaptureWorkload::TreeLightingBenchmark => {
            app.add_systems(Last, super::benchmark_tree_lighting);
        }
        CaptureWorkload::ScenePerformanceBenchmark => {
            app.add_systems(
                Last,
                (
                    super::benchmark_scene_performance,
                    super::benchmark_progress::report,
                ),
            );
        }
        CaptureWorkload::Review => install_review(app),
    }
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

fn install_review(app: &mut App) {
    let capture = super::capture_views
        .run_if(super::building_review::ready)
        .run_if(super::furniture_readiness::ready);
    #[cfg(feature = "grown-review")]
    let capture = capture.run_if(super::grown_capture::ready).run_if(not(
        super::grown_capture::generated::pending_adaptive_configuration,
    ));
    app.add_systems(Last, capture);
    #[cfg(feature = "grown-review")]
    app.add_systems(
        PreUpdate,
        super::capture_views
            .run_if(super::building_review::ready)
            .run_if(super::furniture_readiness::ready)
            .run_if(super::grown_capture::ready)
            .run_if(super::grown_capture::generated::pending_adaptive_configuration)
            .after(super::grown_capture::runtime::advance),
    );
}
