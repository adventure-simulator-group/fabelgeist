//! Grow a scene without the simulation thread and render it headless: no
//! window, the camera drawing into an image that is saved once. With
//! `--frames` the same view is first drawn that many times, waiting for the
//! GPU after each, and the time a frame takes is reported with the GPU's
//! time for each render pass. `--orbit` turns the camera once round the
//! scene meanwhile, as a player looking round does, and `--refeed` hands
//! the forest the snapshot again every so many frames, as the simulation
//! hands it new ones.

use std::f32::consts::TAU;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use adventuresim_environment::render::{ForestFeed, ForestPlugin};
use anyhow::Result;
use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::camera::RenderTarget;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::render_resource::TextureFormat;
use bevy::render::render_resource::TextureUsages;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use fabelgeist_botany::Ecosystem;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::sim::{self, Snapshot};
use fabelgeist_gpu::globals::WgpuContext;

use crate::stage::{self, Orbit};

/// The image's size, pixels.
const SIZE: UVec2 = UVec2::new(1600, 1000);
/// Frames drawn, and time waited, once the forest is drawn and before the
/// capture or the timing: for the renderer to compile its pipelines and
/// settle.
const WARMUP_FRAMES: u32 = 30;
const WARMUP: Duration = Duration::from_secs(1);
/// Pitch of the close-up camera above the horizon, radians.
const CLOSEUP_PITCH: f32 = 0.15;
/// Distance of the close-up camera, in heights of the plant it frames.
const CLOSEUP_DISTANCE: f32 = 1.6;

/// Where a snapshot's camera looks.
pub struct Framing {
    /// At the tallest plant rather than the whole scene.
    pub closeup: bool,
    /// Share of the plant's height to look at, closing up.
    pub aim: f32,
    pub yaw: Option<f32>,
    /// How much closer than the default framing.
    pub zoom: f32,
}

/// What to grow and how to show it.
pub struct Request {
    pub path: PathBuf,
    pub scene: Scene,
    /// Grown instead of the scene, when given.
    pub ecosystem: Option<Ecosystem>,
    pub years: f32,
    pub profile: Profile,
    pub framing: Framing,
    /// Frames to time before the capture.
    pub frames: Option<u32>,
    /// Turn the camera once round the scene over the timed frames.
    pub orbit: bool,
    /// Hand the forest the snapshot again every this many timed frames.
    pub refeed: Option<u32>,
}

/// The grown scene, waiting to be shown, and where the frame goes.
#[derive(Resource)]
struct Still {
    snapshot: Arc<Snapshot>,
    shown: bool,
    orbit: Orbit,
    path: PathBuf,
    frames: Option<u32>,
    turning: bool,
    refeed: Option<u32>,
    target: Option<Handle<Image>>,
    /// When the forest was first drawn, and the frames since.
    drawn: Option<Instant>,
    settling: u32,
    /// When the timing started, and the frames timed since.
    timing: Option<Instant>,
    timed: u32,
    captured: bool,
}

/// Grow the scene and render it headless, then leave.
pub fn run(mut request: Request) -> Result<()> {
    let ecosystem = request
        .ecosystem
        .take()
        .unwrap_or_else(|| request.scene.build(1));
    let (snapshot, orbit) = grow(ecosystem, &request)?;
    let mut app = App::new();
    app.insert_resource(ClearColor(stage::SKY))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins((ScheduleRunnerPlugin::run_loop(Duration::ZERO), ForestPlugin))
        .insert_resource(Still {
            snapshot: Arc::new(snapshot),
            shown: false,
            orbit,
            path: request.path,
            frames: request.frames,
            turning: request.orbit,
            refeed: request.refeed,
            target: None,
            drawn: None,
            settling: 0,
            timing: None,
            timed: 0,
            captured: false,
        })
        .add_systems(Startup, (setup, stage::sun))
        .add_systems(Update, (show, step).chain());
    if request.frames.is_some() {
        app.add_plugins(RenderDiagnosticsPlugin);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.add_systems(Render, wait_for_gpu.in_set(RenderSystems::Cleanup));
        }
    }
    app.run();
    Ok(())
}

/// Grow `ecosystem` for the years asked, and frame its snapshot.
fn grow(ecosystem: Ecosystem, request: &Request) -> Result<(Snapshot, Orbit)> {
    let context = pollster::block_on(WgpuContext::new())?;
    let mut ecosystem = ecosystem.with_gpu(context)?;
    ecosystem.settings.dt = request.profile.step_years;
    let steps = (request.years / ecosystem.settings.dt).ceil() as usize;
    let started = Instant::now();
    for _ in 0..steps {
        pollster::block_on(ecosystem.step())?;
    }
    let snapshot = sim::capture(&ecosystem, &request.profile, 1, true, 0.0);
    println!(
        "{}: {:.0} years in {:.1} s, {} plants, {} segments, {} leaf clusters",
        request.scene.name(),
        ecosystem.time,
        started.elapsed().as_secs_f32(),
        snapshot.plants,
        snapshot.branches.len(),
        snapshot.leaves.len()
    );
    let framing = &request.framing;
    let mut orbit = Orbit::framing(snapshot.bounds);
    let tallest = ecosystem
        .plants
        .iter()
        .max_by(|a, b| a.height().total_cmp(&b.height()));
    if let Some(plant) = tallest.filter(|_| framing.closeup) {
        let height = plant.height().max(1.0);
        let p = plant.position;
        orbit.focus = Vec3::new(p.x, p.y + height * framing.aim, p.z);
        orbit.radius = height * CLOSEUP_DISTANCE;
        orbit.pitch = CLOSEUP_PITCH;
    }
    if let Some(yaw) = framing.yaw {
        orbit.yaw = yaw;
    }
    orbit.radius /= framing.zoom;
    Ok((snapshot, orbit))
}

/// A camera drawing into an image instead of a window.
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut still: ResMut<Still>) {
    let mut image = Image::new_target_texture(SIZE.x, SIZE.y, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = images.add(image);
    commands.spawn((
        stage::camera(),
        RenderTarget::Image(target.clone().into()),
        still.orbit.transform(),
    ));
    still.target = Some(target);
}

/// Show the grown scene once.
fn show(mut still: ResMut<Still>, mut feed: ResMut<ForestFeed>) {
    if !still.shown {
        still.shown = true;
        feed.show(still.snapshot.clone());
    }
}

/// Settle, time the frames asked for, then save the image and leave.
fn step(
    mut commands: Commands,
    mut still: ResMut<Still>,
    mut feed: ResMut<ForestFeed>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
    diagnostics: Option<Res<DiagnosticsStore>>,
) {
    if still.captured || (still.drawn.is_none() && !feed.is_drawn()) {
        return;
    }
    let drawn = *still.drawn.get_or_insert_with(Instant::now);
    if still.settling < WARMUP_FRAMES || drawn.elapsed() < WARMUP {
        still.settling += 1;
        return;
    }
    if let Some(frames) = still.frames {
        let started = *still.timing.get_or_insert_with(Instant::now);
        let timed = still.timed;
        still.timed += 1;
        if timed < frames {
            if still.turning {
                let mut orbit = still.orbit;
                orbit.yaw += TAU * timed as f32 / frames as f32;
                for mut transform in &mut cameras {
                    *transform = orbit.transform();
                }
            }
            if still
                .refeed
                .is_some_and(|every| timed.is_multiple_of(every.max(1)))
            {
                feed.show(still.snapshot.clone());
            }
            return;
        }
        report(started.elapsed(), frames, diagnostics.as_deref());
    }
    let Some(target) = still.target.clone() else {
        return;
    };
    still.captured = true;
    let path = still.path.clone();
    commands.spawn(Screenshot::image(target)).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            save_to_disk(path.clone())(captured);
            println!("wrote {}", path.display());
            exit.write(AppExit::Success);
        },
    );
}

/// Print the time a frame took, and each render pass's GPU time.
fn report(elapsed: Duration, frames: u32, diagnostics: Option<&DiagnosticsStore>) {
    let milliseconds = elapsed.as_secs_f64() * 1000.0 / f64::from(frames);
    println!("{milliseconds:.2} ms a frame over {frames} frames, waiting for the GPU");
    let Some(diagnostics) = diagnostics else {
        return;
    };
    let mut passes: Vec<(String, f64)> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.path().as_str().ends_with("elapsed_gpu"))
        .filter_map(|diagnostic| {
            let average = diagnostic.average()?;
            Some((diagnostic.path().as_str().to_owned(), average))
        })
        .collect();
    passes.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (path, average) in passes {
        println!("  {average:7.3} ms  {path}");
    }
}

/// Wait for the GPU to finish each frame, so the time a frame takes counts
/// the GPU's work.
fn wait_for_gpu(device: Res<RenderDevice>) {
    let _ = device.poll(bevy::render::render_resource::PollType::wait_indefinitely());
}
