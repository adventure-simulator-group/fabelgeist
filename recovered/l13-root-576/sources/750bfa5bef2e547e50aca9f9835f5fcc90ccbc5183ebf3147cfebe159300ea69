//! Linear radiance in a latitude-longitude studio light map, shared with armor baking.

use glam::Vec3;
use std::{f32::consts::PI, sync::LazyLock};
use tiny_skia::Pixmap;

const MAP_WIDTH: usize = 128;
const MAP_HEIGHT: usize = 64;
const EXPOSURE: f32 = 1.4;
const REFLECTION_SAMPLES: usize = 12;

static ENVIRONMENT: LazyLock<Vec<Vec3>> = LazyLock::new(|| {
    (0..MAP_WIDTH * MAP_HEIGHT)
        .map(|index| {
            let longitude = ((index % MAP_WIDTH) as f32 + 0.5) / MAP_WIDTH as f32;
            let latitude = ((index / MAP_WIDTH) as f32 + 0.5) / MAP_HEIGHT as f32;
            let theta = latitude * PI;
            let phi = longitude * 2.0 * PI;
            let direction = Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            );
            let sky = Vec3::new(0.16, 0.19, 0.25);
            let floor = Vec3::new(0.035, 0.028, 0.022);
            let surround = floor.lerp(sky, direction.y * 0.5 + 0.5);
            let key = direction
                .dot(Vec3::new(-0.4, 0.4, 0.8).normalize())
                .max(0.0)
                .powi(12);
            let rim = direction
                .dot(Vec3::new(0.85, 0.25, -0.45).normalize())
                .max(0.0)
                .powi(48);
            surround
                + Vec3::new(1.0, 0.92, 0.78) * key * 3.0
                + Vec3::new(0.65, 0.8, 1.0) * rim * 2.0
        })
        .collect()
});

fn sample(direction: Vec3) -> Vec3 {
    let u = direction.z.atan2(direction.x).rem_euclid(2.0 * PI) / (2.0 * PI);
    let v = direction.y.clamp(-1.0, 1.0).acos() / PI;
    let x = (u * MAP_WIDTH as f32) as usize % MAP_WIDTH;
    let y = ((v * MAP_HEIGHT as f32) as usize).min(MAP_HEIGHT - 1);
    ENVIRONMENT[y * MAP_WIDTH + x]
}

fn filtered(direction: Vec3, spread: f32) -> Vec3 {
    let tangent = direction.any_orthonormal_vector();
    let bitangent = direction.cross(tangent);
    let mut color = sample(direction);
    for index in 0..REFLECTION_SAMPLES {
        let angle = index as f32 * 2.399_963_1;
        let radius = ((index as f32 + 0.5) / REFLECTION_SAMPLES as f32).sqrt() * spread;
        color += sample(
            (direction + radius * (angle.cos() * tangent + angle.sin() * bitangent)).normalize(),
        );
    }
    color / (REFLECTION_SAMPLES + 1) as f32
}

pub(super) fn shade(normal: Vec3, view: Vec3, albedo: Vec3, metallic: f32, roughness: f32) -> Vec3 {
    let normal = if normal.dot(view) < 0.0 {
        -normal
    } else {
        normal
    };
    let reflection = 2.0 * normal.dot(view) * normal - view;
    let fresnel = albedo.lerp(Vec3::splat(0.04), 1.0 - metallic);
    let fresnel = fresnel + (Vec3::ONE - fresnel) * (1.0 - normal.dot(view).max(0.0)).powi(5);
    let diffuse = filtered(normal, 1.5) * albedo * (1.0 - metallic);
    let specular = filtered(reflection.normalize(), roughness * roughness * 2.0) * fresnel;
    (diffuse + specular) * EXPOSURE
}

pub(super) fn srgb(linear: f32) -> u8 {
    // Reinhard highlight compression followed by the display transfer function.
    let value = linear.max(0.0) / (1.0 + linear.max(0.0));
    let encoded = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

/// Export the same linear studio radiance as RGBE for offline armor rendering.
pub fn environment_hdr() -> Vec<u8> {
    let mut output =
        format!("#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {MAP_HEIGHT} +X {MAP_WIDTH}\n")
            .into_bytes();
    for color in ENVIRONMENT.iter() {
        let exponent = color.max_element().log2().floor() as i32 + 1;
        let scale = 256.0 / 2.0_f32.powi(exponent);
        output.extend([
            (color.x * scale) as u8,
            (color.y * scale) as u8,
            (color.z * scale) as u8,
            (exponent + 128) as u8,
        ]);
    }
    output
}

pub(super) fn encode_png(size: u16, rgba: Vec<u8>) -> Result<Vec<u8>, super::IconError> {
    let dimensions = tiny_skia::IntSize::from_wh(u32::from(size), u32::from(size))
        .ok_or(super::IconError::InvalidSpec)?;
    Pixmap::from_vec(rgba, dimensions)
        .ok_or(super::IconError::Rasterization)?
        .encode_png()
        .map_err(|error| super::IconError::Png(error.to_string()))
}
