//! A live view of an ecosystem growing, in Bevy.
//!
//! ```text
//! cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
//! ```
//!
//! The simulation runs on its own thread; each snapshot it hands over is
//! drawn by the environment crate's forest renderer (GPU-instanced branches
//! and leaves), under a game's HUD: the year and the population, the time
//! controls, a hotbar of tools and a menu (Esc) for the scene, the climate
//! and the display.
//!
//! Drag to orbit, right-drag to pan, wheel to zoom. Space plays and pauses.
//! The digits pick a plant from the hotbar, and a click then sows it where
//! the pointer rests: on the ground, or at the foot of a wall. G picks
//! Clear, which takes the plants out of the ring under the pointer. Esc puts
//! the tool down, or opens the menu; F3 shows the figures.
//!
//! With `--snapshot out.png` it grows the scene for `--years` first, without
//! the simulation thread, and renders it once headless, without a window;
//! `--frames 300` first times that many frames of it. `--closeup`
//! frames the tallest plant instead of the whole scene, looking at `--aim`
//! of its height; `--solo conifer` grows one plant of a species alone.

mod frame_times;
mod hotbar;
mod hud;
mod icons;
mod live;
mod menu;
mod pointer;
mod profile_controls;
mod snapshot;
mod stage;
mod stats;
mod theme;
mod tools;
mod widgets;
mod world;

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
    /// Time this many frames of the snapshot before saving it.
    #[arg(long)]
    frames: Option<u32>,
    /// Turn the camera once round the scene while timing.
    #[arg(long)]
    orbit: bool,
    /// Hand the forest the snapshot again every this many timed frames.
    #[arg(long)]
    refeed: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();
    let scene = match &options.scene {
        Some(name) => scene_named(name)?,
        None => Scene::MixedForest,
    };
    if let Some(path) = &options.snapshot {
        let profile = Profile::named(&options.profile)
            .ok_or_else(|| anyhow!("unknown profile `{}`", options.profile))?;
        return snapshot::run(snapshot::Request {
            path: path.clone(),
            scene,
            ecosystem: options.solo.map(SoloSpecies::ecosystem),
            years: options.years,
            profile: profile_controls::drawable(profile),
            framing: Framing {
                closeup: options.closeup,
                aim: options.aim,
                yaw: options.yaw,
                zoom: options.zoom,
            },
            frames: options.frames,
            orbit: options.orbit,
            refeed: options.refeed,
        });
    }
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
        .add_plugins((EguiPlugin::default(), PanOrbitCameraPlugin, ForestPlugin));
    live::install(&mut app, scene);
    app.run();
    Ok(())
}
