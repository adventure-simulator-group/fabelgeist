//! Grow a scene without the simulation thread and render it headless: no
//! window, the camera drawing into an image that is saved once. With
//! `--frames` the same view is first drawn that many times, waiting for the
//! GPU after each, and the time a frame takes is reported with the GPU's
//! time for each render pass. `--orbit` turns the camera once round the
//! scene meanwhile, as a player looking round does, and `--refeed` hands
//! the forest the snapshot again every so many frames, as the simulation
//! hands it new ones.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use adventuresim_environment::render::benchmark::profile::{ProfileMode, RenderProfile};
use adventuresim_environment::render::benchmark::{ForestBenchmark, ValidationPolicy, WindowStart};
use adventuresim_environment::render::{
    ForestDetail, ForestFeed, ForestPlugin, LevelSelection,
    trees::{TreeFeed, TreeForestPlugin, TreeLayout, TreeMeshStats},
};
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

use crate::benchmark::{self, ArrivalPath, Measurement, PixelStability, Readback};
use crate::capture_view::CaptureView;
use crate::input::{self, FrozenForest, Source};
use crate::stage::{self, Orbit};

mod profiling;
mod sampling;

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
    pub view: Option<PathBuf>,
    pub tree_layout: TreeLayout,
    pub validation: ValidationPolicy,
    pub render_profile: Option<ProfileMode>,
    pub path: PathBuf,
    pub source: Source,
    pub level: LevelSelection,
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
    view: CaptureView,
    forest: FrozenForest,
    started: Instant,
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
    timed_elapsed: Option<Duration>,
    captured: bool,
}

#[derive(SystemParam)]
struct CaptureObservation<'w> {
    diagnostics: Option<Res<'w, DiagnosticsStore>>,
    measurement: Option<ResMut<'w, Measurement>>,
    probe: Option<Res<'w, ForestBenchmark>>,
    profile: Option<Res<'w, RenderProfile>>,
    exit: MessageWriter<'w, AppExit>,
}

/// Grow the scene and render it headless, then leave.
pub fn run(mut request: Request) -> Result<()> {
    let explicit_view = request.view.as_deref().map(CaptureView::read).transpose()?;
    let prepared = matches!(&request.source, Source::Prepared { .. });
    let input::Loaded {
        forest,
        frame,
        preparation,
    } = input::load(request.source)?;
    let shared = matches!(&forest, FrozenForest::Shared(_));
    let arrival = if shared {
        ArrivalPath::PreparedShared
    } else if prepared {
        ArrivalPath::PreparedUnique
    } else {
        ArrivalPath::GrowthReplay
    };
    let view = explicit_view.unwrap_or_else(|| CaptureView::framed(request.framing.orbit(&frame)));
    let orbit = view.orbit;
    let mut detail = frame.detail;
    detail.level = request.level;
    let probe = (prepared || request.benchmark.is_some()).then(|| {
        let started = request
            .benchmark
            .as_ref()
            .map_or_else(Instant::now, |r| r.started);
        let probe = ForestBenchmark::new(started, request.validation);
        let mut instances = forest.expected_instances();
        if !detail.leaves {
            instances.leaves = 0;
        }
        probe.expect_instances(instances);
        probe
    });
    let measurement = request.benchmark.take().map(|request| {
        Measurement::new(request, preparation, &frame, detail, view.clone(), arrival)
    });
    let mut app = capture_app(detail, request.tree_layout, arrival);
    app.insert_resource(Still {
        view,
        forest,
        started: Instant::now(),
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
        timed_elapsed: None,
        captured: false,
    })
    .add_systems(Startup, (setup, stage::sun))
    .add_systems(Update, (show, step).chain());
    let observed = probe.is_some();
    if let Some(probe) = probe {
        if shared {
            probe.install_shared(&mut app);
        } else {
            probe.install(&mut app);
        }
    }
    if let Some(measurement) = measurement {
        app.insert_resource(measurement);
    } else if prepared {
        app.init_resource::<PixelStability>();
    }
    if request.frames.is_some() || request.render_profile.is_some() {
        app.add_plugins(RenderDiagnosticsPlugin);
        if !observed && let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.add_systems(Render, wait_for_gpu.in_set(RenderSystems::Cleanup));
        }
    }
    profiling::install(&mut app, request.render_profile, arrival)?;
    let outcome = app.run();
    anyhow::ensure!(
        matches!(outcome, AppExit::Success),
        "forest capture failed: {outcome:?}"
    );
    Ok(())
}

/// Install the native offscreen renderer for the selected arrival path.
fn capture_app(detail: ForestDetail, layout: TreeLayout, arrival: ArrivalPath) -> App {
    let mut app = App::new();
    app.insert_resource(ClearColor(stage::SKY))
        .insert_resource(layout)
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
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO));
    match arrival {
        ArrivalPath::PreparedShared => {
            app.add_plugins(TreeForestPlugin);
        }
        ArrivalPath::GrowthReplay | ArrivalPath::PreparedUnique => {
            app.add_plugins(ForestPlugin);
        }
    }
    app
}

/// The comparison camera follows source metadata, regardless of representation.
impl Framing {
    fn orbit(&self, frame: &adventuresim_environment::prepared::ForestFrame) -> Orbit {
        let mut orbit = Orbit::framing(frame.bounds);
        if let Some(focus) = frame.focus.filter(|_| self.closeup) {
            let height = focus.height.metres().max(1.0);
            let [x, y, z] = focus.root.map(fabelgeist_math::units::Metres::metres);
            orbit.focus = Vec3::new(x, y + height * self.aim, z);
            orbit.radius = height * CLOSEUP_DISTANCE;
            orbit.pitch = CLOSEUP_PITCH;
        }
        if let Some(yaw) = self.yaw {
            orbit.yaw = yaw;
        }
        orbit.radius /= self.zoom;
        orbit
    }
}

/// A camera drawing into an image instead of a window.
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut still: ResMut<Still>) {
    let mut image = Image::new_target_texture(SIZE.x, SIZE.y, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = images.add(image);
    commands
        .spawn((
            stage::camera(),
            RenderTarget::Image(target.clone().into()),
            still.view.transform(),
        ))
        .insert(still.view.lens());
    still.target = Some(target);
}

/// Show the grown scene once.
fn show(
    mut still: ResMut<Still>,
    mut feed: ResMut<ForestFeed>,
    mut trees: Option<ResMut<TreeFeed>>,
    mut exit: MessageWriter<AppExit>,
) {
    if !still.shown {
        still.shown = true;
        match &still.forest {
            FrozenForest::Unique(snapshot) => feed.show(snapshot.clone()),
            FrozenForest::Shared(forest) => {
                let Some(trees) = trees.as_mut() else {
                    error!("shared-tree renderer missing");
                    exit.write(AppExit::error());
                    return;
                };
                trees.show(forest.clone());
            }
        }
    }
}

/// Settle, time the frames asked for, then save the image and leave.
fn step(
    mut commands: Commands,
    mut still: ResMut<Still>,
    mut feed: ResMut<ForestFeed>,
    mut trees: Option<ResMut<TreeFeed>>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
    observation: CaptureObservation,
) {
    let CaptureObservation {
        diagnostics,
        mut measurement,
        probe,
        profile,
        mut exit,
    } = observation;
    if profile.as_ref().is_some_and(|profile| !profile.healthy()) {
        error!("render profiling failed");
        exit.write(AppExit::error());
        return;
    }
    if let (Some(measurement), Some(probe)) = (measurement.as_ref(), probe.as_ref())
        && let Err(error) = measurement.check(probe)
    {
        error!("{error}");
        exit.write(AppExit::error());
        return;
    }
    if let Some(probe) = &probe {
        let status = probe.status();
        const CAPTURE_TIMEOUT: Duration = Duration::from_secs(240);
        if status.failed || still.started.elapsed() > CAPTURE_TIMEOUT {
            error!("forest capture readiness failed: {status:?}");
            exit.write(AppExit::error());
            return;
        }
    }
    let is_drawn = match &still.forest {
        FrozenForest::Unique(_) => feed.is_drawn(),
        FrozenForest::Shared(_) => trees.as_ref().is_some_and(|feed| feed.is_drawn()),
    };
    if still.captured || (still.drawn.is_none() && !is_drawn) {
        return;
    }
    if let Some(measurement) = measurement.as_mut() {
        measurement.handoff_finished();
    }
    if probe
        .as_ref()
        .is_some_and(|probe| probe.status().first_submission_micros.is_none())
    {
        return;
    }
    if !still.settle(&mut commands, measurement.as_deref_mut()) {
        return;
    }
    if let Some(probe) = &probe {
        match probe.validation().begin_sampling() {
            WindowStart::Waiting | WindowStart::Opened => return,
            WindowStart::Active => {}
        }
    }
    if profile.as_ref().is_some_and(|profile| !profile.ready()) {
        return;
    }
    if let Some(frames) = still.frames {
        if still.sample_frames(
            frames,
            measurement.as_deref_mut(),
            &mut feed,
            trees.as_deref_mut(),
            &mut cameras,
        ) {
            return;
        }
        if probe
            .as_ref()
            .is_some_and(|probe| !probe.validation().finish_sampling())
        {
            return;
        }
        if let Some(elapsed) = still.timed_elapsed {
            report(elapsed, frames, diagnostics.as_deref());
        }
    }
    if probe
        .as_ref()
        .is_some_and(|probe| !probe.validation().finish_sampling())
    {
        return;
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
              mut pixels: Option<ResMut<PixelStability>>,
              probe: Option<Res<ForestBenchmark>>,
              topology: Option<Res<TreeMeshStats>>,
              profile: Option<Res<RenderProfile>>,
              diagnostics: Option<Res<DiagnosticsStore>>,
              mut exit: MessageWriter<AppExit>| {
            if let Some(measurement) = measurement.as_mut() {
                let Some(probe) = probe.as_ref() else {
                    error!("the foliage benchmark probe is missing");
                    exit.write(AppExit::error());
                    return;
                };
                match measurement.readback(&captured.image, probe, topology.as_deref(), profile.as_deref(), diagnostics.as_deref()) {
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
            if let Some(pixels) = pixels.as_mut() {
                match pixels.observe(&captured.image) {
                    Ok(Readback::Repeat) => {
                        save_to_disk(path.with_extension(format!("readback-{:02}.png", pixels.attempts())))(captured);
                        capture(&mut commands, target.clone(), path.clone());
                        return;
                    }
                    Ok(Readback::Stable) => {}
                    Err(error) => { error!("{error}"); exit.write(AppExit::error()); return; }
                }
            }
            if let Err(error) = profiling::save(profile.as_deref(), diagnostics.as_deref(), &path) {
                error!("{error}"); exit.write(AppExit::error()); return;
            }
            if let Some(probe) = probe {
                let counts = probe.counts();
                println!("forest validation: {:?}", probe.validation().evidence());
                println!("forest ready: {}/{} writes, {}/{} branch/leaf instances, {} batches, {} meshes", counts.submitted_writes, counts.queued_writes, counts.expected_branch_instances, counts.expected_leaf_instances, counts.expected_batches, counts.expected_meshes);
            }
            if let Some(topology) = topology {
                match serde_json::to_string(&*topology) {
                    Ok(json) => println!("forest topology: {json}"),
                    Err(error) => { error!("{error}"); exit.write(AppExit::error()); return; }
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
