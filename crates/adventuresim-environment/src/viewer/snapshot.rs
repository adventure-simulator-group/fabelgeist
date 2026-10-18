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

use adventuresim_environment::render::benchmark::ForestBenchmark;
use adventuresim_environment::render::{ForestDetail, ForestFeed, ForestPlugin};
use anyhow::Result;
use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::camera::RenderTarget;
use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::system::SystemParam;
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

use crate::benchmark::{self, Measurement, Preparation, Readback};
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
    pub benchmark: Option<benchmark::Request>,
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

#[derive(SystemParam)]
struct CaptureObservation<'w> {
    diagnostics: Option<Res<'w, DiagnosticsStore>>,
    measurement: Option<ResMut<'w, Measurement>>,
    probe: Option<Res<'w, ForestBenchmark>>,
    exit: MessageWriter<'w, AppExit>,
}

/// Grow the scene and render it headless, then leave.
pub fn run(mut request: Request) -> Result<()> {
    let scene_started = Instant::now();
    let ecosystem = request.ecosystem.take().unwrap_or_else(|| {
        request
            .scene
            .build(fabelgeist_determinism::Seed::from_u64(1))
    });
    let scene_time = scene_started.elapsed();
    let (snapshot, orbit, mut preparation) = grow(ecosystem, &request)?;
    preparation.scene = scene_time;
    let detail = ForestDetail {
        tube_sides: request.profile.tube_sides,
        tube_spans: request.profile.tube_spans,
        leaf_blades: request.profile.leaf_blades,
        full_pixels: request.profile.full_pixels,
        middle_pixels: request.profile.middle_pixels,
        leaves: true,
    };
    let measurement = request.benchmark.take().map(|benchmark| {
        let probe = ForestBenchmark::new(benchmark.started);
        probe.expect_snapshot(snapshot.branches.len(), snapshot.leaves.len());
        let measurement = Measurement::new(
            benchmark,
            preparation,
            request.scene.name().into(),
            &snapshot,
            detail,
            orbit,
        );
        (probe, measurement)
    });
    let mut app = App::new();
    app.insert_resource(ClearColor(stage::SKY))
        .insert_resource(detail)
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
    let observed = measurement.is_some();
    if let Some((probe, measurement)) = measurement {
        probe.install(&mut app);
        app.insert_resource(measurement);
    }
    if request.frames.is_some() {
        app.add_plugins(RenderDiagnosticsPlugin);
        if !observed && let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.add_systems(Render, wait_for_gpu.in_set(RenderSystems::Cleanup));
        }
    }
    app.run();
    Ok(())
}

/// Grow `ecosystem` for the years asked, and frame its snapshot.
fn grow(ecosystem: Ecosystem, request: &Request) -> Result<(Snapshot, Orbit, Preparation)> {
    let gpu_started = Instant::now();
    let context = pollster::block_on(WgpuContext::new())?;
    let mut ecosystem = ecosystem.with_gpu(context)?;
    let gpu_setup = gpu_started.elapsed();
    ecosystem.settings.dt = request.profile.step_years;
    let steps = (request.years / ecosystem.settings.dt.duration().years()).ceil() as usize;
    let started = Instant::now();
    for _ in 0..steps {
        pollster::block_on(ecosystem.step())?;
    }
    let growth = started.elapsed();
    let packing_started = Instant::now();
    let snapshot = sim::capture(&ecosystem, &request.profile, 1, true, 0.0);
    let packing = packing_started.elapsed();
    println!(
        "{}: {:.0} years in {:.1} s, {} plants, {} segments, {} leaf clusters",
        request.scene.name(),
        ecosystem.time.years(),
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
    Ok((
        snapshot,
        orbit,
        Preparation {
            gpu_setup,
            growth,
            packing,
            ..Default::default()
        },
    ))
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
    observation: CaptureObservation,
) {
    let CaptureObservation {
        diagnostics,
        mut measurement,
        probe,
        mut exit,
    } = observation;
    if let (Some(measurement), Some(probe)) = (measurement.as_ref(), probe.as_ref())
        && let Err(error) = measurement.check(probe)
    {
        error!("{error}");
        exit.write(AppExit::error());
        return;
    }
    if still.captured || (still.drawn.is_none() && !feed.is_drawn()) {
        return;
    }
    if let Some(measurement) = measurement.as_mut() {
        measurement.handoff_finished();
    }
    if probe
        .as_ref()
        .is_some_and(|probe| probe.counts().first_submission_micros.is_none())
    {
        return;
    }
    let drawn = *still.drawn.get_or_insert_with(Instant::now);
    if still.settling < WARMUP_FRAMES || drawn.elapsed() < WARMUP {
        still.settling += 1;
        return;
    }
    if let Some(frames) = still.frames {
        if let Some(measurement) = measurement.as_mut() {
            measurement.frame();
        }
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
    capture(&mut commands, target, path);
}

fn capture(commands: &mut Commands, target: Handle<Image>, path: PathBuf) {
    commands.spawn(Screenshot::image(target.clone())).observe(
        move |captured: On<ScreenshotCaptured>,
              mut commands: Commands,
              mut measurement: Option<ResMut<Measurement>>,
              probe: Option<Res<ForestBenchmark>>,
              mut exit: MessageWriter<AppExit>| {
            if let Some(measurement) = measurement.as_mut() {
                let Some(probe) = probe.as_ref() else {
                    error!("the foliage benchmark probe is missing");
                    exit.write(AppExit::error());
                    return;
                };
                match measurement.readback(&captured.image, probe) {
                    Ok(Readback::Repeat) => {
                        let attempt = measurement.readback_number();
                        save_to_disk(path.with_extension(format!("readback-{attempt:02}.png")))(
                            captured,
                        );
                        capture(&mut commands, target.clone(), path.clone());
                        return;
                    }
                    Ok(Readback::Stable) => {}
                    Err(error) => {
                        error!("{error}");
                        exit.write(AppExit::error());
                        return;
                    }
                }
            }
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
