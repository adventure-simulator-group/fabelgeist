//! GPU performance testbed for the tactical scene.
//!
//! One scene (ground, grass, trees, characters), every rendering choice on a
//! runtime switch (the always-on panel), frame statistics on screen and in a CSV. Runs native
//! and on WebGPU in the browser; the web is the product, the desktop the
//! development loop.

mod affectors;
mod custom_material;
mod displacement;
mod grass;
mod scene;
mod settings;
mod shading;
mod sky;
mod stats;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::error_handler::{RenderErrorHandler, RenderErrorPolicy};
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::window::{PresentMode, WindowResolution};

/// Keeps the app alive through wgpu validation errors instead of quitting.
///
/// The default handler exits, which is right for a game and useless here: on
/// a downlevel device (WebGL2, no storage buffers) one unsupported bind group
/// killed the whole bench before you could pick a mode that does work. The
/// modes that cannot exist there are compiled out, so anything that still
/// errors is worth seeing rather than dying over -- but the log is capped,
/// since a per-frame error would otherwise bury the console.
fn keep_rendering(
    error: &bevy::render::error_handler::RenderError,
    _main: &mut World,
    _render: &mut World,
) -> RenderErrorPolicy {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEEN: AtomicUsize = AtomicUsize::new(0);
    let seen = SEEN.fetch_add(1, Ordering::Relaxed);
    if seen < 8 {
        error!(
            "render error ignored ({:?}): {}",
            error.ty, error.description
        );
    } else if seen == 8 {
        error!("further render errors will not be logged");
    }
    RenderErrorPolicy::Ignore
}

fn main() {
    let settings = settings::BenchSettings::load();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::audio::AudioPlugin>()
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                    // BENCH_DOWNLEVEL=1 holds a desktop GPU to WebGL2's
                    // limits (no storage buffers, one cascade, and the rest),
                    // which is the only way to reproduce a weak laptop's
                    // failures without owning one.
                    constrained_limits: std::env::var("BENCH_DOWNLEVEL")
                        .is_ok()
                        .then(bevy::render::settings::WgpuLimits::downlevel_webgl2_defaults),
                    ..default()
                })),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "GPU bench".into(),
                    resolution: if cfg!(feature = "downlevel") {
                        WindowResolution::new(1280, 720).with_scale_factor_override(1.0)
                    } else {
                        WindowResolution::new(1280, 720)
                    },
                    present_mode: if settings.vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    },
                    #[cfg(target_family = "wasm")]
                    canvas: Some("#bevy".into()),
                    #[cfg(target_family = "wasm")]
                    fit_canvas_to_parent: true,
                    #[cfg(target_family = "wasm")]
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // No .meta files shipped: skip the guaranteed-404 probe per asset on wasm.
                meta_check: AssetMetaCheck::Never,
                // Web builds fetch assets under a content-hashed URL prefix
                // (set by scripts/build-wasm-client.sh, stripped by the dev
                // server): a changed asset is a changed URL, so browsers can
                // never serve a stale glb.
                file_path: option_env!("WEB_ASSET_PREFIX")
                    .map(|v| format!("{v}/assets"))
                    .unwrap_or_else(|| "assets".to_string()),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.58, 0.76, 0.95)))
    .insert_resource(settings)
    .insert_resource(bevy_egui::EguiGlobalSettings {
        auto_create_primary_context: false,
        ..default()
    })
    .add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy_egui::EguiPlugin {
            bindless_mode_array_size: if cfg!(feature = "downlevel") {
                None
            } else {
                std::num::NonZero::new(16)
            },
            ..default()
        },
        bevy_line_boil::LineBoilPlugin,
        affectors::AffectorsPlugin,
        custom_material::CustomMaterialPlugin,
        displacement::DisplacementPlugin,
        settings::SettingsPlugin,
        stats::StatsPlugin,
        shading::ShadingPlugin,
        scene::ScenePlugin,
        grass::GrassPlugin,
    ));
    #[cfg(not(target_family = "wasm"))]
    app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    app.insert_resource(RenderErrorHandler(keep_rendering));
    app.run();
}
