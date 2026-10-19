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

mod benchmark;
mod frame_times;
mod hotbar;
mod hud;
mod icons;
mod input;
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
use std::time::Instant;

use adventuresim_environment::prepared::VariantBudget;
use adventuresim_environment::render::benchmark::ValidationPolicy;
use adventuresim_environment::render::benchmark::profile::ProfileMode;
use adventuresim_environment::render::trees::TreeLayout;
use adventuresim_environment::render::{ForestPlugin, Level, LevelSelection};
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

fn main() -> anyhow::Result<()> {
    let started = Instant::now();
    let options = Options::parse();
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
            options.tree_layout.is_none()
                || matches!(options.representation, Some(input::Representation::Shared)),
            "--tree-layout requires --representation shared"
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
            path: path.clone(),
            tree_layout: options.tree_layout.unwrap_or_default(),
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
            benchmark: options
                .benchmark_report
                .map(|report| benchmark::Request { report, started }),
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

impl Options {
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
