//! A live view of an ecosystem growing.
//!
//! ```text
//! cargo run --release -p adventuresim-environment --features viewer --bin environment-viewer
//! ```
//!
//! The simulation runs on its own thread; the window draws its latest
//! snapshot every frame with botany's instanced renderer, and the panel on
//! the right steers it: the scene on one tab, and on the other the profile
//! -- every knob that trades detail for speed, from a preset.
//!
//! Drag to orbit, right-drag to pan, wheel to zoom. Space plays and pauses,
//! G cuts a gap in the canopy where the camera is looking, R resets the
//! scene.
//!
//! With `--snapshot out.png` it opens no window: it grows the scene for
//! `--years`, renders one frame and writes it. `--closeup` frames the
//! tallest plant instead of the whole scene, looking at `--aim` of its
//! height; `--solo conifer` grows one plant of a species alone.
//!
//! The simulation, the renderer and the camera are botany's own
//! (`fabelgeist_botany::viewer`); only this interface over them lives here.

mod app;
mod frame_times;
mod navigation;
mod profile_controls;
mod scene_controls;
mod snapshot;
mod stats;
mod view_texture;

use std::path::PathBuf;

use adventuresim_environment::{SoloSpecies, scene_named};
use anyhow::{Context as _, anyhow};
use clap::Parser;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;

use app::EnvironmentViewer;
use snapshot::Framing;

/// The window's size when it opens, logical pixels.
const WINDOW_SIZE: [f32; 2] = [1480.0, 900.0];

#[derive(Parser)]
#[command(about = "Watch a botany ecosystem grow, or render one frame of it")]
struct Options {
    /// The scene, by the start of its name (`boreal`, `savanna`...).
    #[arg(long)]
    scene: Option<String>,
    /// Grow the scene and write one rendered frame here instead of opening a window.
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
    if let Some(path) = &options.snapshot {
        let profile = Profile::named(&options.profile)
            .ok_or_else(|| anyhow!("unknown profile `{}`", options.profile))?;
        let framing = Framing {
            closeup: options.closeup,
            aim: options.aim,
            yaw: options.yaw,
            zoom: options.zoom,
        };
        let ecosystem = options.solo.map(SoloSpecies::ecosystem);
        return snapshot::render(path, scene, options.years, profile, &framing, ecosystem);
    }

    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Synthetic Silviculture")
            .with_inner_size(WINDOW_SIZE),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: view_texture::wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        "Synthetic Silviculture",
        native,
        Box::new(move |creation| Ok(Box::new(EnvironmentViewer::new(creation, scene)?))),
    )
    .map_err(|error| anyhow!("{error}"))
    .context("the viewer window failed")
}
