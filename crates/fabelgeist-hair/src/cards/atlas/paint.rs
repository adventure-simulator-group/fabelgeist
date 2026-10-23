//! Rasterize and assemble seeded strand bundles.

use super::{AtlasMip, AtlasMipLayout, AtlasResult, AtlasSettings, RgbaTexel, zeroed};
use crate::ops::smoothstep;
use crate::rng::Rng;
use fabelgeist_determinism::Seed;
use rayon::prelude::*;

const BUNDLE_SEED_STRIDE: u64 = 0x9E37;

pub(super) fn paint_base(settings: AtlasSettings, seed: Seed) -> AtlasResult<AtlasMip> {
    let layout = AtlasMipLayout::for_atlas(settings)?;
    let tile = AtlasMipLayout::for_tile(settings.tile)?;
    let mut base = AtlasMip::blank(layout)?;
    let mut columns = zeroed::<Vec<RgbaTexel>>(settings.variants.count() as usize)?;
    columns
        .par_iter_mut()
        .enumerate()
        .try_for_each(|(variant, column)| {
            // The enumeration is bounded by the admitted u32 variant count;
            // its product with this fixed seed mask fits u64 exactly.
            let seed = seed.xor_word(u64::from(variant as u32) * BUNDLE_SEED_STRIDE);
            *column = paint_bundle(tile, seed)?;
            Ok::<(), super::AtlasError>(())
        })?;
    // Successful completion filled every column to the admitted tile size.
    // Convert layout extents only at this host image indexing boundary.
    let width = tile.width() as usize;
    let atlas_width = layout.width() as usize;
    for (variant, column) in columns.iter().enumerate() {
        for y in 0..tile.height() as usize {
            let from = y * width;
            let to = y * atlas_width + variant * width;
            base.pixels[to..to + width].copy_from_slice(&column[from..from + width]);
        }
    }
    Ok(base)
}

/// One bundle: strands down a `width` by `height` column, root at the top.
fn paint_bundle(layout: AtlasMipLayout, seed: Seed) -> AtlasResult<Vec<RgbaTexel>> {
    let (width, height) = (layout.width(), layout.height());
    let mut rng = Rng::new(seed.to_u64());
    let mut coverage = zeroed::<f32>(layout.pixels())?;
    let mut shade = zeroed::<f32>(layout.pixels())?;
    let mut depth = zeroed::<f32>(layout.pixels())?;
    let strands = 26 + rng.index(20);
    let converge = rng.range(0.2, 0.6);
    for _ in 0..strands {
        let x0 = rng.range(0.1, 0.9);
        let amplitude = rng.range(0.0, 0.035);
        let frequency = rng.range(0.5, 3.0);
        let phase = rng.range(0.0, std::f32::consts::TAU);
        let end = rng.range(0.7, 1.0);
        let start = rng.range(0.0, 0.05);
        let root_width = rng.range(0.9, 1.6) * width as f32 / 64.0;
        let tone = rng.next_f32();
        let layer = rng.next_f32();
        let path = |v: f32| {
            let pull = smoothstep(0.2, 1.0, v) * converge;
            let x = x0
                + (0.5 - x0) * pull
                + amplitude * (frequency * v * std::f32::consts::TAU + phase).sin();
            [x * width as f32, v * height as f32]
        };
        let steps = height as usize * 2;
        for k in 0..steps {
            let v = start + (end - start) * k as f32 / steps as f32;
            let [px, py] = path(v);
            let taper = 1.0 - smoothstep(0.6, 1.0, (v - start) / (end - start)) * 0.7;
            let r = (root_width * 0.5 * taper).max(0.35);
            let (x0i, x1i) = ((px - r - 1.0).floor() as i32, (px + r + 1.0).ceil() as i32);
            let y = py as i32;
            if y < 0 || y >= height as i32 {
                continue;
            }
            for x in x0i..=x1i {
                // The column wraps, so bundles tile across wide cards.
                let xw = x.rem_euclid(width as i32) as u32;
                let d = (x as f32 + 0.5 - px).abs();
                let c = (r + 0.5 - d).clamp(0.0, 1.0);
                let i = y as usize * width as usize + xw as usize;
                if c > coverage[i] || (c > 0.0 && layer < depth[i]) {
                    coverage[i] = coverage[i].max(c);
                    shade[i] = tone;
                    depth[i] = layer;
                }
            }
        }
    }
    let mut out = zeroed::<RgbaTexel>(layout.pixels())?;
    for (i, texel) in out.iter_mut().enumerate() {
        let c = (coverage[i] * 255.0).round() as u8;
        *texel = [c, (shade[i] * 255.0) as u8, (depth[i] * 255.0) as u8, c];
    }
    Ok(out)
}
