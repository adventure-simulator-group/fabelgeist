//! A live view of an ecosystem growing, in Bevy.
//!
//! ```text
//! cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
//! ```
//!
//! The simulation runs on its own thread; each snapshot it hands over is
//! drawn by the environment crate's forest renderer (GPU-instanced branches
//! and leaves), and the panel on the right steers it: the scene on one tab,
//! and on the other how the simulation steps and how finely the forest is
//! built.
//!
//! Drag to orbit, right-drag to pan, wheel to zoom. Space plays and pauses,
//! G cuts a gap in the canopy where the camera is looking, R resets the
//! scene. With sowing on (S), a click sows the species picked in the panel
//! where the cursor points: on the ground, or at the foot of a wall.
//!
//! With `--snapshot out.png` it grows the scene for `--years` first, without
//! the simulation thread, shows it, and saves the window once. `--closeup`
//! frames the tallest plant instead of the whole scene, looking at `--aim`
//! of its height; `--solo conifer` grows one plant of a species alone.

mod frame_times;
mod live;
mod profile_controls;
mod scene_controls;
mod snapshot;
mod sowing;
mod stage;
mod stats;

use std::path::PathBuf;

use adventuresim_environment::render::ForestPlugin;
use adventuresim_environment::{SoloSpecies, scene_named};
use anyhow::anyhow;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use clap::Parser;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;

use snapshot::Framing;

/// The window's size, logical pixels.
const WINDOW_SIZE: [u32; 2] = [1480, 900];

#[derive(Parser)]
#[command(about = "Watch a botany ecosystem grow, or capture one frame of it")]
struct Options {
    /// The scene, by the start of its name (`boreal`, `castle-unplanted`...).
    #[arg(long)]
    scene: Option<String>,
    /// Grow the scene and save one frame here instead of watching it grow.
    #[arg(long)]
    snapshot: Option<PathBuf>,
    /// Years to grow before the snapshot.
    #[arg(long, default_value_t = 120.0)]
    years: f32,
    /// The snapshot's profile preset: `fast` or `detailed`.
    #[arg(long, default_value = "detailed")]
    profile: String,
    /// Frame the tallest plant rather than the whole scene.
    #[arg(long)]
    closeup: bool,
    /// Share of the tallest plant's height to look at, closing up.
    #[arg(long, default_value_t = 0.45)]
    aim: f32,
    /// Camera yaw, radians.
    #[arg(long)]
    yaw: Option<f32>,
    /// How much closer than the default framing.
    #[arg(long, default_value_t = 1.0)]
    zoom: f32,
    /// Grow one plant of this species alone.
    #[arg(long, value_enum)]
    solo: Option<SoloSpecies>,
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();
    let scene = match &options.scene {
        Some(name) => scene_named(name)?,
        None => Scene::MixedForest,
    };
    let [width, height] = WINDOW_SIZE;
    let mut app = App::new();
    app.insert_resource(ClearColor(stage::SKY))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Synthetic Silviculture".into(),
                resolution: WindowResolution::new(width, height),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), PanOrbitCameraPlugin, ForestPlugin))
        .add_systems(Startup, stage::setup);
    match &options.snapshot {
        Some(path) => {
            let profile = Profile::named(&options.profile)
                .ok_or_else(|| anyhow!("unknown profile `{}`", options.profile))?;
            let framing = Framing {
                closeup: options.closeup,
                aim: options.aim,
                yaw: options.yaw,
                zoom: options.zoom,
            };
            let ecosystem = options.solo.map(SoloSpecies::ecosystem);
            snapshot::install(
                &mut app,
                path.clone(),
                scene,
                options.years,
                profile_controls::drawable(profile),
                &framing,
                ecosystem,
            )?;
        }
        None => live::install(&mut app, scene),
    }
    app.run();
    Ok(())
}
