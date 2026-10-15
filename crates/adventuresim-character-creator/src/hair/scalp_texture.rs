//! The scalp's texture: the hair lying on it, baked from the groom.
//!
//! Painted one flat colour, the scalp shows between cards and strands as a
//! smooth, dead patch. Over a real scalp the eye meets the hair nearest it --
//! strands lying along the skin in the groom's own direction, parted where
//! the groom parts and spiralling out of its whorl. Each strand's first few
//! millimetres off the skin are drawn into the scalp's UV map (the hair
//! crate's projection about the crown) on the bust the hair grew on, the
//! topmost strand over each texel showing: lighter the further it lies off
//! the skin, darker where it meets it. Single strands are far finer than a
//! pixel at any distance the head is seen from, so what shows is locks:
//! strands rooted together share a tone, lighter or darker than the next
//! lock, and differ a little among themselves. Where no strand lies, the
//! scalp is in the hair's shadow.

use std::sync::atomic::{AtomicU32, Ordering};

use fabelgeist_hair::{Head, Strands};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::AtlasLevel;

/// The texture's side, texels: about a fifth of a millimetre a texel over
/// the crown, finer down the sides, so lying strands read as lines.
const SIZE: usize = 2048;
/// How far off the skin a strand is drawn, metres: the layer of hair the
/// eye meets between cards.
const REACH: f32 = 0.008;
/// A strand's shade where it meets the skin, and at [`REACH`] off it, as a
/// share of the hair's colour.
const AT_SKIN: f32 = 0.25;
const AT_REACH: f32 = 0.9;
/// How much darker a lock may be than the lightest, and a strand than the
/// rest of its lock.
const LOCK_SPREAD: f32 = 0.5;
const STRAND_SPREAD: f32 = 0.15;
/// About how wide a lock is, metres: the cell of the noise its tone is
/// drawn from, by where its strands are rooted.
const LOCK_WIDTH: f32 = 0.004;
/// The scalp's shade where no strand lies over it: in the hair's shadow.
const BARE: f32 = 0.35;
/// Steps of a strand's height, and of its shade, in a texel's packed value:
/// the topmost strand wins, carrying its shade.
const STEPS: f32 = 65_535.0;

/// The texture's mip chain, largest first: the scalp's shade as grey, as a
/// share of the hair's colour.
pub(super) fn bake(strands: &Strands, head: &Head) -> Vec<AtlasLevel> {
    let topmost: Vec<AtomicU32> = (0..SIZE * SIZE).map(|_| AtomicU32::new(0)).collect();
    (0..strands.len()).into_par_iter().for_each(|strand| {
        let lock = lock_noise(strands.root(strand) * (1.0 / LOCK_WIDTH));
        let tone = 1.0 - LOCK_SPREAD * lock - STRAND_SPREAD * unit(strands.id[strand]);
        let mut previous: Option<([f32; 2], f32)> = None;
        for &point in strands.strand(strand) {
            let height = head.distance(point).max(0.0);
            let [u, v] = head.scalp_uv(point);
            let inside = (0.0..1.0).contains(&u) && (0.0..1.0).contains(&v);
            let here =
                (height < REACH && inside).then_some(([u * SIZE as f32, v * SIZE as f32], height));
            if let (Some(from), Some(to)) = (previous, here) {
                draw(&topmost, from, to, tone);
            }
            previous = here;
        }
    });
    let base = AtlasLevel {
        width: SIZE as u32,
        height: SIZE as u32,
        rgba: topmost
            .iter()
            .flat_map(|packed| {
                let packed = packed.load(Ordering::Relaxed);
                let shade = if packed == 0 {
                    BARE
                } else {
                    (packed & 0xffff) as f32 / STEPS
                };
                let grey = (shade * 255.0).round() as u8;
                [grey, grey, grey, u8::MAX]
            })
            .collect(),
    };
    let mut levels = vec![base];
    while let Some(smaller) = levels.last().and_then(halve) {
        levels.push(smaller);
    }
    levels
}

/// Draw a strand's segment from `from` to `to`, each a texel position and
/// a height off the skin, into `topmost`, where it lies over what is there.
fn draw(topmost: &[AtomicU32], from: ([f32; 2], f32), to: ([f32; 2], f32), tone: f32) {
    let ([x0, y0], h0) = from;
    let ([x1, y1], h1) = to;
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
        let height = (h0 + (h1 - h0) * t) / REACH;
        let shade = (AT_SKIN + (AT_REACH - AT_SKIN) * height) * tone;
        let packed = ((height * STEPS) as u32) << 16 | (shade * STEPS) as u32;
        let (column, row) = (x as usize, y as usize);
        if column < SIZE && row < SIZE {
            topmost[row * SIZE + column].fetch_max(packed.max(1), Ordering::Relaxed);
        }
    }
}

/// Smooth noise, 0 to 1, varying over about a unit of `p`: values hashed
/// at the corners of unit cells, blended smoothly between.
fn lock_noise(p: Vec3) -> f32 {
    let base = [p.x.floor(), p.y.floor(), p.z.floor()];
    let fraction = [p.x - base[0], p.y - base[1], p.z - base[2]].map(|t| t * t * (3.0 - 2.0 * t));
    let mut sum = 0.0;
    for corner in 0..8u32 {
        let offset = [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1];
        let key = (0..3).fold(0u32, |key, axis| {
            let cell = base[axis] as i32 + offset[axis] as i32;
            mix(key ^ cell as u32)
        });
        let weight: f32 = (0..3)
            .map(|axis| {
                if offset[axis] == 1 {
                    fraction[axis]
                } else {
                    1.0 - fraction[axis]
                }
            })
            .product();
        sum += weight * unit(key);
    }
    sum
}

/// A strand's own value, 0 to 1, from its stable ID.
fn unit(id: u32) -> f32 {
    mix(id) as f32 / u32::MAX as f32
}

/// A well-mixed 32-bit hash (the "lowbias32" of Chris Wellons).
fn mix(value: u32) -> u32 {
    let mut x = value.wrapping_add(0x9e37_79b9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// `level` at half its size, each texel the mean of four, or nothing past
/// a single texel.
fn halve(level: &AtlasLevel) -> Option<AtlasLevel> {
    if level.width == 1 && level.height == 1 {
        return None;
    }
    let (width, height) = ((level.width / 2).max(1), (level.height / 2).max(1));
    let texel = |x: u32, y: u32, channel: usize| {
        let (x, y) = (x.min(level.width - 1), y.min(level.height - 1));
        u32::from(level.rgba[((y * level.width + x) * 4) as usize + channel])
    };
    let rgba = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            std::array::from_fn::<u8, 4, _>(|channel| {
                let sum = texel(2 * x, 2 * y, channel)
                    + texel(2 * x + 1, 2 * y, channel)
                    + texel(2 * x, 2 * y + 1, channel)
                    + texel(2 * x + 1, 2 * y + 1, channel);
                (sum / 4) as u8
            })
        })
        .collect();
    Some(AtlasLevel {
        width,
        height,
        rgba,
    })
}
