//! Antialiased coverage, categorical pigment, and height-derived front/back normals.
use super::{kernel::Kernel, relief::soften, uniform::LeafUniform};
use crate::{LeafRecipe, LeafSpecies, LeafTextureSet, TextureParameters, leaf_pixels::LeafPixels};
use bevy::{asset::Assets, image::Image, math::Vec2};
const EDGE_SAMPLES: usize = 2;
struct Coverage {
    blade: f32,
    vein: f32,
}
impl Coverage {
    fn alpha(&self) -> f32 {
        self.blade + self.vein
    }
    fn color(&self, blade: [u8; 3], vein: [u8; 3]) -> [u8; 4] {
        let a = self.alpha();
        if a == 0.0 {
            return [0; 4];
        }
        let mut rgba = [0; 4];
        for i in 0..3 {
            rgba[i] =
                ((blade[i] as f32 * self.blade + vein[i] as f32 * self.vein) / a).round() as u8;
        }
        rgba[3] = (a * 255.0).round() as u8;
        rgba
    }
}
fn coverage(kernel: &Kernel, size: usize) -> Vec<Coverage> {
    let mut pixels = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let mut counts = [0; 3];
            for sy in 0..EDGE_SAMPLES {
                for sx in 0..EDGE_SAMPLES {
                    let p = (Vec2::new(
                        x as f32 + (sx as f32 + 0.5) / EDGE_SAMPLES as f32,
                        y as f32 + (sy as f32 + 0.5) / EDGE_SAMPLES as f32,
                    ) / size as f32
                        - Vec2::splat(0.5))
                        * 2.0;
                    counts[kernel.class(p) as usize] += 1;
                }
            }
            let samples = (EDGE_SAMPLES * EDGE_SAMPLES) as f32;
            pixels.push(Coverage {
                blade: counts[1] as f32 / samples,
                vein: counts[2] as f32 / samples,
            });
        }
    }
    pixels
}
pub(crate) fn generate(
    params: &TextureParameters,
    images: &mut Assets<Image>,
    species: LeafSpecies,
    colors: LeafRecipe,
) -> LeafTextureSet {
    let leaf = params.leaves.get(species);
    let size = params.size(crate::TEXTURE_SIZE) as usize;
    let kernel = Kernel::from(LeafUniform::new(&leaf.shape, size as u32, params.seed));
    let pixels = coverage(&kernel, size);
    let vein = soften(
        &pixels.iter().map(|p| p.vein).collect::<Vec<_>>(),
        size,
        (leaf.relief.vein_softness * size as f32).max(0.5),
    );
    let mut front_height = Vec::with_capacity(size * size);
    let mut back_height = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let p =
                (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / size as f32 - Vec2::splat(0.5)) * 2.0;
            front_height.push(
                leaf.relief
                    .height(p, &leaf.shape, vein[y * size + x], false),
            );
            back_height.push(leaf.relief.height(p, &leaf.shape, vein[y * size + x], true));
        }
    }
    let mut maps = LeafPixels {
        opacity: Vec::new(),
        front: Vec::new(),
        back: Vec::new(),
        normal_front: Vec::new(),
        normal_back: Vec::new(),
        height_map: Vec::new(),
        arm: Vec::new(),
    };
    for y in 0..size {
        for x in 0..size {
            let i = y * size + x;
            let pixel = &pixels[i];
            maps.opacity
                .extend_from_slice(&[(pixel.alpha() * 255.0).round() as u8; 4]);
            maps.front
                .extend_from_slice(&pixel.color(colors.blade, colors.vein));
            maps.back
                .extend_from_slice(&pixel.color(colors.back_blade, colors.vein));
            maps.normal_front.extend_from_slice(&leaf.relief.normal(
                &front_height,
                x,
                y,
                size,
                false,
            ));
            maps.normal_back
                .extend_from_slice(&leaf.relief.normal(&back_height, x, y, size, true));
            let h = (front_height[i] * 255.0).round() as u8;
            maps.height_map.extend_from_slice(&[h, h, h, 255]);
            let ao = ((1.0 - vein[i] * leaf.relief.ao_strength) * 255.0).round() as u8;
            maps.arm.extend_from_slice(&[
                ao,
                (f32::from(colors.roughness) * leaf.relief.roughness_gain).round() as u8,
                0,
                255,
            ]);
        }
    }
    maps.upload(params, images)
}
