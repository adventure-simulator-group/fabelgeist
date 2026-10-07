//! Grow a scene without the simulation thread and capture one frame of it.

use std::path::PathBuf;
use std::time::Instant;

use adventuresim_environment::render::ForestFeed;
use anyhow::Result;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy_panorbit_camera::PanOrbitCamera;
use fabelgeist_botany::Ecosystem;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::sim::{self, Snapshot};
use fabelgeist_gpu::globals::WgpuContext;

use crate::stage;

/// Real time the renderer is given to compile its pipelines and settle
/// before the capture, seconds.
const WARMUP_SECONDS: f64 = 3.0;
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

/// The grown scene, waiting to be shown, and where the frame goes.
#[derive(Resource)]
struct Still {
    snapshot: Option<Snapshot>,
    pose: Pose,
    path: PathBuf,
    captured: bool,
}

/// Where the snapshot's camera stands.
#[derive(Clone, Copy)]
struct Pose {
    bounds: [[f32; 3]; 2],
    /// A close-up instead of the scene's framing.
    closeup: Option<Closeup>,
    yaw: Option<f32>,
    zoom: f32,
}

/// A camera on one plant: what it looks at and from how far, metres.
#[derive(Clone, Copy)]
struct Closeup {
    focus: Vec3,
    distance: f32,
}

/// Grow `scene` (or `ecosystem`) for `years` and set `app` to capture one
/// frame of it to `path`.
pub fn install(
    app: &mut App,
    path: PathBuf,
    scene: Scene,
    years: f32,
    profile: Profile,
    framing: &Framing,
    ecosystem: Option<Ecosystem>,
) -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    let mut ecosystem = ecosystem
        .unwrap_or_else(|| scene.build(1))
        .with_gpu(context)?;
    ecosystem.settings.dt = profile.step_years;
    let steps = (years / ecosystem.settings.dt).ceil() as usize;
    let started = Instant::now();
    for _ in 0..steps {
        pollster::block_on(ecosystem.step())?;
    }
    let snapshot = sim::capture(&ecosystem, &profile, 1, true, 0.0);
    println!(
        "{}: {:.0} years in {:.1} s, {} plants, {} segments, {} leaf clusters",
        scene.name(),
        ecosystem.time,
        started.elapsed().as_secs_f32(),
        snapshot.plants,
        snapshot.branches.len(),
        snapshot.leaves.len()
    );
    let tallest = ecosystem
        .plants
        .iter()
        .max_by(|a, b| a.height().total_cmp(&b.height()));
    let closeup = tallest.filter(|_| framing.closeup).map(|plant| {
        let height = plant.height().max(1.0);
        let p = plant.position;
        Closeup {
            focus: Vec3::new(p.x, p.y + height * framing.aim, p.z),
            distance: height * CLOSEUP_DISTANCE,
        }
    });
    app.insert_resource(Still {
        pose: Pose {
            bounds: snapshot.bounds,
            closeup,
            yaw: framing.yaw,
            zoom: framing.zoom,
        },
        snapshot: Some(snapshot),
        path,
        captured: false,
    })
    .add_systems(Update, (show, capture).chain());
    Ok(())
}

/// Show the grown scene once, and hold the camera on it.
fn show(
    mut still: ResMut<Still>,
    mut feed: ResMut<ForestFeed>,
    mut cameras: Query<&mut PanOrbitCamera>,
) {
    let Some(snapshot) = still.snapshot.take() else {
        return;
    };
    feed.show(snapshot);
    let pose = still.pose;
    for mut camera in &mut cameras {
        // A still is taken where the camera is put, not on its way there.
        camera.orbit_smoothness = 0.0;
        camera.pan_smoothness = 0.0;
        camera.zoom_smoothness = 0.0;
        stage::frame(&mut camera, pose.bounds);
        if let Some(closeup) = pose.closeup {
            stage::aim(&mut camera, closeup.focus, closeup.distance);
            camera.target_pitch = CLOSEUP_PITCH;
            camera.pitch = Some(CLOSEUP_PITCH);
        }
        if let Some(yaw) = pose.yaw {
            camera.target_yaw = yaw;
            camera.yaw = Some(yaw);
        }
        let distance = camera.target_radius / pose.zoom;
        let focus = camera.target_focus;
        stage::aim(&mut camera, focus, distance);
    }
}

/// Once the renderer has settled, save the window and leave.
fn capture(mut commands: Commands, mut still: ResMut<Still>, time: Res<Time<Real>>) {
    if still.captured || time.elapsed_secs_f64() < WARMUP_SECONDS {
        return;
    }
    still.captured = true;
    let path = still.path.clone();
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            save_to_disk(path.clone())(captured);
            println!("wrote {}", path.display());
            exit.write(AppExit::Success);
        },
    );
}
