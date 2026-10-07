//! Grow a scene without a window and render one frame of it to a PNG.

use std::path::Path;
use std::time::Instant;

use anyhow::{Context as _, Result, anyhow};
use fabelgeist_botany::Ecosystem;
use fabelgeist_botany::scenes::Scene;
use fabelgeist_botany::viewer::camera::OrbitCamera;
use fabelgeist_botany::viewer::profile::Profile;
use fabelgeist_botany::viewer::renderer::{Renderer, View};
use fabelgeist_botany::viewer::sim;
use fabelgeist_gpu::globals::WgpuContext;

/// The snapshot's size, pixels.
const WIDTH: u32 = 1600;
const HEIGHT: u32 = 1000;
/// Bytes in one RGBA8 pixel.
const PIXEL_BYTES: u32 = 4;
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

/// Grow `scene` (or `ecosystem`, when given) for `years` and write one frame of
/// it to `path`.
pub fn render(
    path: &Path,
    scene: Scene,
    years: f32,
    profile: Profile,
    framing: &Framing,
    ecosystem: Option<Ecosystem>,
) -> Result<()> {
    let context = pollster::block_on(WgpuContext::new())?;
    let mut ecosystem = ecosystem
        .unwrap_or_else(|| scene.build(1))
        .with_gpu(context.clone())?;
    ecosystem.settings.dt = profile.step_years;
    let steps = (years / ecosystem.settings.dt).ceil() as usize;
    let started = Instant::now();
    for _ in 0..steps {
        pollster::block_on(ecosystem.step())?;
    }
    let captured = sim::capture(&ecosystem, &profile, 1, true, 0.0);
    println!(
        "{}: {:.0} years in {:.1} s, {} plants, {} segments, {} leaf clusters",
        scene.name(),
        ecosystem.time,
        started.elapsed().as_secs_f32(),
        captured.plants,
        captured.branches.len(),
        captured.leaves.len()
    );

    let mut renderer = Renderer::new(
        context.device.clone(),
        context.queue.clone(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )?;
    renderer.upload(&captured);
    let (texture, _, _) = renderer.resize(WIDTH, HEIGHT);
    let camera = framed_camera(&ecosystem, captured.bounds, framing);
    renderer.render(
        &camera,
        &View {
            sun_azimuth: 0.9,
            sun_elevation: 0.75,
            shadows: true,
            leaves: true,
        },
        &profile,
    );
    let pixels = read_texture(&context, &texture)?;
    image::RgbaImage::from_raw(WIDTH, HEIGHT, pixels)
        .ok_or_else(|| anyhow!("unexpected readback size"))?
        .save(path)
        .with_context(|| format!("could not write {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

fn framed_camera(ecosystem: &Ecosystem, bounds: [[f32; 3]; 2], framing: &Framing) -> OrbitCamera {
    let [lo, hi] = bounds;
    let mut camera = OrbitCamera::framing(lo, hi);
    let tallest = ecosystem
        .plants
        .iter()
        .max_by(|a, b| a.height().total_cmp(&b.height()));
    if let (true, Some(plant)) = (framing.closeup, tallest) {
        let height = plant.height().max(1.0);
        let p = plant.position;
        camera.target = [p.x, p.y + height * framing.aim, p.z];
        camera.pitch = CLOSEUP_PITCH;
        camera.distance = height * CLOSEUP_DISTANCE;
    }
    if let Some(yaw) = framing.yaw {
        camera.yaw = yaw;
    }
    camera.distance /= framing.zoom;
    camera
}

/// The rendered frame's RGBA8 pixels, row after row.
fn read_texture(context: &WgpuContext, texture: &wgpu::Texture) -> Result<Vec<u8>> {
    let row = WIDTH * PIXEL_BYTES;
    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("snapshot readback"),
        size: u64::from(row * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = context.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    context.queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    context.device.poll(wgpu::PollType::wait_indefinitely())?;
    receiver.recv()??;
    Ok(buffer.slice(..).get_mapped_range().to_vec())
}
