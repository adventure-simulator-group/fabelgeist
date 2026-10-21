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
//! the tool down, or opens the menu; C keeps distant leaves' coverage; F3
//! shows the figures.
//!
//! With `--snapshot out.png` it grows the scene for `--years` first, without
//! the simulation thread, and renders it once headless, without a window;
//! `--frames 300` first times that many frames of it. `--closeup`
//! frames the tallest plant instead of the whole scene, looking at `--aim`
//! of its height; `--solo conifer` grows one plant of a species alone.
//!
//! In the browser it is the live viewer alone, of the mixed forest, in the
//! page's `#bevy` canvas: the captures, benchmarks and asset preparation
//! read and write files, and stay native.

#[cfg(not(target_arch = "wasm32"))]
mod benchmark;
#[cfg(not(target_arch = "wasm32"))]
mod capture_view;
mod frame_times;
mod hotbar;
mod hud;
mod icons;
#[cfg(not(target_arch = "wasm32"))]
mod input;
mod live;
mod menu;
mod pointer;
mod profile_controls;
#[cfg(not(target_arch = "wasm32"))]
mod snapshot;
mod stage;
mod stats;
mod theme;
mod tools;
mod widgets;
mod world;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::prepared::VariantBudget;
#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::render::benchmark::ValidationPolicy;
#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::render::benchmark::profile::ProfileMode;
#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::render::trees::{TreeBatching, TreeLayout, TreeResidencyPolicy};
use adventuresim_environment::render::ForestPlugin;
#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::render::{Level, LevelSelection};
#[cfg(not(target_arch = "wasm32"))]
use adventuresim_environment::{SoloSpecies, scene_named};
#[cfg(not(target_arch = "wasm32"))]
use anyhow::anyhow;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
use fabelgeist_botany::scenes::Scene;
#[cfg(not(target_arch = "wasm32"))]
use fabelgeist_botany::viewer::profile::Profile;

#[cfg(not(target_arch = "wasm32"))]
use snapshot::Framing;

/// The window's size, logical pixels.
const WINDOW_SIZE: [u32; 2] = [1480, 900];

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser)]
#[command(about = "Watch a botany ecosystem grow, or capture one frame of it")]
#[command(group(clap::ArgGroup::new("prepared_input").args(["prepared_forest", "arrival_sequence"])))]
struct Options {
    /// Run serial A/B/A arrivals in one renderer using a checked JSON manifest.
    #[arg(long, requires = "sequence_output", conflicts_with_all = ["prepare_forest", "prepared_forest", "snapshot", "representation", "frames", "fixed_lod", "probe_validation", "render_profile", "benchmark_report", "scene", "solo", "years", "profile", "closeup", "aim", "yaw", "zoom", "orbit", "refeed", "capture_view", "tree_layout", "all_plants", "variants_per_species"])]
    arrival_sequence: Option<PathBuf>,
    #[arg(long, requires = "arrival_sequence")]
    sequence_output: Option<PathBuf>,
    #[arg(long, value_enum, requires = "arrival_sequence")]
    bank_residency: Option<TreeResidencyPolicy>,
    /// Grow and freeze render assets offline; this never times arrival.
    #[arg(long, conflicts_with_all = ["snapshot", "prepared_forest", "frames", "benchmark_report", "orbit", "refeed"]) ]
    prepare_forest: Option<PathBuf>,
    /// Maximum representatives per species, preserving leafed/bare states.
    #[arg(long, requires = "prepare_forest", conflicts_with = "all_plants")]
    variants_per_species: Option<std::num::NonZeroU16>,
    /// One model per source plant for a small renderer parity fixture.
    #[arg(long, requires = "prepare_forest")]
    all_plants: bool,
    /// Load frozen assets directly, without growing an ecosystem.
    #[arg(long, requires_all = ["snapshot", "representation"], conflicts_with_all = ["scene", "solo", "years", "profile", "refeed", "orbit"]) ]
    prepared_forest: Option<PathBuf>,
    #[arg(long, value_enum, requires = "prepared_forest")]
    representation: Option<input::Representation>,
    /// Force a common geometric level for controlled capture comparisons.
    #[arg(long, value_enum, requires = "snapshot")]
    fixed_lod: Option<Level>,
    /// Shared-tree diagnostic: hold model mesh/batch partitions fixed.
    #[arg(long, value_enum, requires = "prepared_forest")]
    tree_layout: Option<TreeLayout>,
    /// Shared-tree spatial cells; regional cells trade finer culling for reuse.
    #[arg(long, value_enum, requires = "prepared_input")]
    tree_batching: Option<TreeBatching>,
    /// Diagnose repeated probe checks on a frozen snapshot with fixed LOD.
    #[arg(long, value_enum, requires_all = ["prepared_forest", "fixed_lod"])]
    probe_validation: Option<ValidationPolicy>,
    /// Diagnose CPU render phases and native GPU tree-culling spans.
    #[arg(long, value_enum, requires_all = ["prepared_forest", "fixed_lod"])]
    render_profile: Option<ProfileMode>,
    /// The scene, by the start of its name (`boreal`, `castle-unplanted`...).
    #[arg(long)]
    scene: Option<String>,
    /// Grow the scene and save one frame here instead of watching it grow.
    #[arg(long)]
    snapshot: Option<PathBuf>,
    /// Explicit eye/target, field of view, clip distance and fog JSON.
    #[arg(long, requires = "snapshot", conflicts_with_all = ["closeup", "aim", "yaw", "zoom", "orbit"])]
    capture_view: Option<PathBuf>,
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
    /// Save stage timings and two stable readbacks for one frozen snapshot.
    #[arg(long, requires = "snapshot", conflicts_with_all = ["refeed", "orbit"])]
    benchmark_report: Option<PathBuf>,
}

#[cfg(target_arch = "wasm32")]
fn main() {
    run_live(Scene::MixedForest);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let started = Instant::now();
    let options = Options::parse();
    if let Some(manifest) = &options.arrival_sequence {
        return options.capture_sequence(manifest, started);
    }
    let scene = match &options.scene {
        Some(name) => scene_named(name)?,
        None => Scene::MixedForest,
    };
    if let Some(directory) = &options.prepare_forest {
        let budget = match (options.all_plants, options.variants_per_species) {
            (true, None) => VariantBudget::AllPlants,
            (false, Some(count)) => VariantBudget::PerSpecies(count),
            _ => {
                return Err(anyhow!(
                    "preparation requires --variants-per-species or --all-plants"
                ));
            }
        };
        return input::prepare(options.growth(scene)?, directory.clone(), budget);
    }
    if let Some(path) = &options.snapshot {
        anyhow::ensure!(
            options
                .tree_layout
                .is_none_or(|layout| layout == TreeLayout::CapacityBatches)
                || options.fixed_lod.is_some(),
            "model-layout diagnostics require --fixed-lod"
        );
        anyhow::ensure!(
            (options.tree_layout.is_none() && options.tree_batching.is_none())
                || matches!(options.representation, Some(input::Representation::Shared)),
            "--tree-layout and --tree-batching require --representation shared"
        );
        let source = match (&options.prepared_forest, options.representation) {
            (Some(directory), Some(representation)) => input::Source::Prepared {
                directory: directory.clone(),
                representation,
            },
            (None, None) => input::Source::Growth(Box::new(options.growth(scene)?)),
            _ => {
                return Err(anyhow!(
                    "prepared capture requires a directory and representation"
                ));
            }
        };
        return snapshot::run(snapshot::Request {
            sequence: None,
            residency: TreeResidencyPolicy::default(),
            view: options.capture_view,
            path: path.clone(),
            tree_layout: options.tree_layout.unwrap_or_default(),
            tree_batching: options.tree_batching.unwrap_or_default(),
            validation: options.probe_validation.unwrap_or_default(),
            render_profile: options.render_profile,
            source,
            level: options
                .fixed_lod
                .map_or(LevelSelection::Adaptive, LevelSelection::Fixed),
            framing: Framing {
                closeup: options.closeup,
                aim: options.aim,
                yaw: options.yaw,
                zoom: options.zoom,
            },
            frames: options.frames,
            orbit: options.orbit,
            refeed: options.refeed,
            benchmark: options.benchmark_report.map(|report| benchmark::Request {
                report,
                started,
                identity: adventuresim_environment::render::benchmark::ForestRequestId::FIRST,
            }),
        });
    }
    run_live(scene);
    Ok(())
}

/// Watch `scene` grow in a window, or the page's canvas.
fn run_live(scene: Scene) {
    let [width, height] = WINDOW_SIZE;
    let mut app = App::new();
    app.insert_resource(ClearColor(stage::SKY))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Synthetic Silviculture".into(),
                resolution: WindowResolution::new(width, height),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#bevy".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), PanOrbitCameraPlugin, ForestPlugin));
    live::install(&mut app, scene);
    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
impl Options {
    fn capture_sequence(&self, manifest: &std::path::Path, started: Instant) -> anyhow::Result<()> {
        let output = self
            .sequence_output
            .clone()
            .ok_or_else(|| anyhow!("missing sequence output"))?;
        snapshot::sequence::run(
            manifest.to_path_buf(),
            output,
            self.tree_batching.unwrap_or_default(),
            self.bank_residency.unwrap_or_default(),
            started,
        )
    }

    fn growth(&self, scene: Scene) -> anyhow::Result<input::Growth> {
        let profile = Profile::named(&self.profile)
            .ok_or_else(|| anyhow!("unknown profile `{}`", self.profile))?;
        Ok(input::Growth {
            scene,
            ecosystem: self.solo.map(SoloSpecies::ecosystem),
            years: self.years,
            profile: profile_controls::drawable(profile),
        })
    }
}
