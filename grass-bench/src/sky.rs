//! The diffuse-convolved gradient sky cubemap the line-boil and custom
//! materials sample (their whole ambient term). Same bake as the game
//! client's `bake_sky_diffuse`, run on every target at startup: 32² × 6 faces
//! × 1024 samples is well under a tenth of a second.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

pub const SKY_DIFFUSE_SIZE: u32 = 32;

pub fn bake_sky_image(ground: Vec3) -> Image {
    let bytes = bake_sky_diffuse(ground);
    Image {
        data: Some(bytes),
        data_order: TextureDataOrder::MipMajor,
        texture_descriptor: TextureDescriptor {
            label: Some("sky_diffuse"),
            size: Extent3d {
                width: SKY_DIFFUSE_SIZE,
                height: SKY_DIFFUSE_SIZE,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::linear(),
        texture_view_descriptor: Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        }),
        asset_usage: RenderAssetUsages::RENDER_WORLD,
        ..default()
    }
}

/// Standard wgpu cubemap face layout: +X, -X, +Y, -Y, +Z, -Z.
fn cube_face_dir(face: u32, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
    .normalize()
}

/// Sky in sRGB: zenith deep blue, horizon bright cyan, ground bounce below.
fn sky_srgb(dir: Vec3, ground: Vec3) -> Vec3 {
    let zenith = Vec3::new(0.10, 0.32, 0.95);
    let horizon = Vec3::new(0.55, 0.82, 1.0);
    if dir.y >= 0.0 {
        horizon.lerp(zenith, dir.y.powf(0.6))
    } else {
        horizon.lerp(ground, (-dir.y).powf(0.4))
    }
}

fn sky_linear(dir: Vec3, ground: Vec3) -> Vec3 {
    fn decode(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    Vec3::from_array(sky_srgb(dir, ground).to_array().map(decode))
}

fn onb(n: Vec3) -> (Vec3, Vec3) {
    let up = if n.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
    let t = up.cross(n).normalize();
    (t, n.cross(t))
}

fn push_f16_rgba(out: &mut Vec<u8>, c: Vec3) {
    fn f16_bits(x: f32) -> u16 {
        let b = x.to_bits();
        let sign = ((b >> 16) & 0x8000) as u16;
        let exp = ((b >> 23) & 0xff) as i32 - 112;
        if exp <= 0 {
            return sign;
        }
        if exp >= 31 {
            return sign | 0x7bff;
        }
        sign | ((exp as u16) << 10) | ((b >> 13) & 0x3ff) as u16
    }
    for ch in [c.x, c.y, c.z, 1.0] {
        out.extend_from_slice(&f16_bits(ch).to_le_bytes());
    }
}

/// Cosine-convolved irradiance cubemap (cosine-weighted average = E/pi).
fn bake_sky_diffuse(ground: Vec3) -> Vec<u8> {
    const SAMPLES: u32 = 1024;
    const GOLDEN_ANGLE: f32 = 2.399_963;
    let size = SKY_DIFFUSE_SIZE;
    let mut out = Vec::with_capacity((size * size * 6 * 8) as usize);
    for face in 0..6 {
        for y in 0..size {
            for x in 0..size {
                let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let v = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let n = cube_face_dir(face, u, v);
                let (t, b) = onb(n);
                let mut sum = Vec3::ZERO;
                for i in 0..SAMPLES {
                    let s = (i as f32 + 0.5) / SAMPLES as f32;
                    let (r, phi) = (s.sqrt(), i as f32 * GOLDEN_ANGLE);
                    let dir = t * (r * phi.cos()) + b * (r * phi.sin()) + n * (1.0 - s).sqrt();
                    sum += sky_linear(dir, ground);
                }
                push_f16_rgba(&mut out, sum / SAMPLES as f32);
            }
        }
    }
    out
}
