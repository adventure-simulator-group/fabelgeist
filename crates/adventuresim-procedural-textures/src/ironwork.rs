//! Seamless forged iron: overlapping die faces, clustered scale loss and pitting.
//! U follows the long forging direction. Fasteners and object-edge wear belong to meshes.
//!
//! Original design evidence:
//! prior-art/ironwork/prior-art.md.
//! That report records a research snapshot; this module and its tests define
//! current behavior.

use super::{SurfaceTextureSet, image_rgba_mipped, palette::albedo_image};
use bevy::{asset::Assets, image::Image, math::Vec3};

pub const IRONWORK_TEXTURE_SIZE: u32 = 1024;
pub const IRONWORK_TILE_METRES: f32 = 0.64;
pub const IRONWORK_HEIGHT_RANGE_METRES: f32 = 0.0018;
mod controls;
mod field;
#[cfg(test)]
mod tests;
pub use controls::Parameters;

fn smooth(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn response(params: &crate::TextureParameters, sample: field::Sample, oxide: f32) -> [u8; 4] {
    let p = &params.ironwork;
    let polish = smooth(
        ((sample.crown - (1.0 - p.polish_fraction)) / p.polish_fraction.max(f32::EPSILON))
            .clamp(0.0, 1.0),
    ) * (1.0 - sample.cavity);
    let roughness = p.bare_roughness + (p.oxide_roughness - p.bare_roughness) * oxide;
    let roughness = roughness + (p.polish_roughness - roughness) * polish * (1.0 - oxide);
    let roughness = roughness + (p.cavity_roughness - roughness) * sample.cavity;
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [
        byte(1.0 - sample.cavity * p.cavity_occlusion),
        byte(roughness),
        byte(1.0 - oxide),
        255,
    ]
}

pub fn generate_ironwork_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> SurfaceTextureSet {
    let size = params.size(IRONWORK_TEXTURE_SIZE);
    let samples = (0..size)
        .flat_map(|y| {
            (0..size).map(move |x| {
                field::sample(
                    params,
                    (x as f32 + 0.5) / size as f32,
                    (y as f32 + 0.5) / size as f32,
                )
            })
        })
        .collect::<Vec<_>>();
    let capacity = (size * size * 4) as usize;
    let mut albedo = Vec::with_capacity(capacity);
    let mut normal = Vec::with_capacity(capacity);
    let mut height = Vec::with_capacity(capacity);
    let mut arm = Vec::with_capacity(capacity);
    let slope_scale =
        params.ironwork.height_range_metres / (2.0 * params.ironwork.tile_metres / size as f32);
    let height_at = |x: i32, y: i32| {
        samples[(y.rem_euclid(size as i32) * size as i32 + x.rem_euclid(size as i32)) as usize]
            .height
    };
    for y in 0..size {
        for x in 0..size {
            let sample = samples[(y * size + x) as usize];
            let oxide = field::oxide_coverage(
                params,
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            let color = params
                .ironwork
                .bare_srgb
                .covered_by(params.ironwork.oxide_srgb, oxide);
            albedo.extend_from_slice(&[color.0[0], color.0[1], color.0[2], 255]);
            let dx = height_at(x as i32 + 1, y as i32) - height_at(x as i32 - 1, y as i32);
            let dy = height_at(x as i32, y as i32 + 1) - height_at(x as i32, y as i32 - 1);
            let n = crate::normal::from_image_gradient(dx * slope_scale, dy * slope_scale);
            let encoded = ((n + Vec3::ONE) * 127.5)
                .round()
                .clamp(Vec3::ZERO, Vec3::splat(255.0));
            normal.extend_from_slice(&[encoded.x as u8, encoded.y as u8, encoded.z as u8, 255]);
            let h = (sample.height * 255.0).round() as u8;
            height.extend_from_slice(&[h, h, h, 255]);
            arm.extend_from_slice(&response(params, sample, oxide));
        }
    }
    SurfaceTextureSet {
        albedo: images.add(albedo_image(albedo, size)),
        normal_gl: images.add(image_rgba_mipped(normal, size, true)),
        height: images.add(image_rgba_mipped(height, size, true)),
        arm: images.add(image_rgba_mipped(arm, size, true)),
    }
}
