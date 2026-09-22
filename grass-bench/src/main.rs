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
use bevy::window::{PresentMode, WindowResolution};

fn main() {
    let settings = settings::BenchSettings::load();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::audio::AudioPlugin>()
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "GPU bench".into(),
                    resolution: WindowResolution::new(1280, 720),
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
        bevy_egui::EguiPlugin::default(),
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
    app.run();
}
