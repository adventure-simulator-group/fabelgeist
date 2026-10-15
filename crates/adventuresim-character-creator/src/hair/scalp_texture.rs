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
//! lock, and differ a little among themselves.
//!
//! Where no strand lies, the scalp is either in the shadow of hair standing
//! further off it -- between an afro's coils, below the coils further out --
//! or bare: at a parting, at a whorl's eye, along a thinning hairline.
//! Where the scalp round a texel is mostly bare, the texel is clear in the
//! texture's alpha, so the skin beneath the cap shows there, lit as the rest
//! of the body: a parting shows as a soft line of skin, while a texel or two
//! between coils does not. Shadowed scalp is the hair's colour in shade.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

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
/// How far off the skin hair still shadows it, metres: hair this near
/// over a texel no strand lies on keeps it from showing bare.
const SHADOW_REACH: f32 = 0.025;
/// The side of the blocks of texels shadowing is judged over: one strand
/// passing over a block shadows all of it, as hair further off the skin
/// shadows more than the line of its own width.
const SHADOW_BLOCK: usize = 8;
/// How far round each texel the bare scalp is measured, texels: about a
/// millimetre across, as wide as a parting.
const BARE_REACH: usize = 2;
/// The share of bare texels round a texel at which skin starts to show
/// there, and shows fully: a parting is mostly bare, the gaps within and
/// between an afro's coils single texels among strands.
const BARE_START: f32 = 0.25;
const BARE_FULL: f32 = 0.6;
/// The scalp's shade where no strand lies on it but hair lies over it.
const SHADOWED: f32 = 0.35;
/// Steps of a strand's height, and of its shade, in a texel's packed value:
/// the topmost strand wins, carrying its shade.
const STEPS: f32 = 65_535.0;

/// The texture's mip chain, largest first: the scalp's shade as grey, as a
/// share of the hair's colour, and whether hair covers it as alpha.
pub(super) fn bake(strands: &Strands, head: &Head) -> Vec<AtlasLevel> {
    let topmost: Vec<AtomicU32> = (0..SIZE * SIZE).map(|_| AtomicU32::new(0)).collect();
    let blocks = SIZE / SHADOW_BLOCK;
    let shadowed: Vec<AtomicBool> = (0..blocks * blocks)
        .map(|_| AtomicBool::new(false))
        .collect();
    (0..strands.len()).into_par_iter().for_each(|strand| {
        let lock = lock_noise(strands.root(strand) * (1.0 / LOCK_WIDTH));
        let tone = 1.0 - LOCK_SPREAD * lock - STRAND_SPREAD * unit(strands.id[strand]);
        let mut previous: Option<([f32; 2], f32)> = None;
        for &point in strands.strand(strand) {
            let height = head.distance(point).max(0.0);
            let [u, v] = head.scalp_uv(point);
            let inside = (0.0..1.0).contains(&u) && (0.0..1.0).contains(&v);
            let here = (height < SHADOW_REACH && inside)
                .then_some(([u * SIZE as f32, v * SIZE as f32], height));
            if let (Some(from), Some(to)) = (previous, here) {
                if from.1.min(to.1) < REACH {
                    draw(&topmost, from, to, tone);
                } else {
                    shadow(&shadowed, from.0, to.0);
                }
            }
            previous = here;
        }
    });
    let bare: Vec<bool> = topmost
        .iter()
        .enumerate()
        .map(|(texel, packed)| {
            let (row, column) = (texel / SIZE, texel % SIZE);
            let block = (row / SHADOW_BLOCK) * blocks + column / SHADOW_BLOCK;
            packed.load(Ordering::Relaxed) == 0 && !shadowed[block].load(Ordering::Relaxed)
        })
        .collect();
    let bare_share = share_around(&bare, BARE_REACH);
    let base = AtlasLevel {
        width: SIZE as u32,
        height: SIZE as u32,
        rgba: topmost
            .iter()
            .zip(bare_share)
            .flat_map(|(packed, share)| {
                let packed = packed.load(Ordering::Relaxed);
                let shade = if packed == 0 {
                    SHADOWED
                } else {
                    (packed & 0xffff) as f32 / STEPS
                };
                let t = ((share - BARE_START) / (BARE_FULL - BARE_START)).clamp(0.0, 1.0);
                let skin = t * t * (3.0 - 2.0 * t);
                let grey = (shade * 255.0).round() as u8;
                [grey, grey, grey, ((1.0 - skin) * 255.0).round() as u8]
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
    let (h0, h1) = (from.1, to.1);
    trace(from.0, to.0, |texel, t| {
        let height = ((h0 + (h1 - h0) * t) / REACH).min(1.0);
        let shade = (AT_SKIN + (AT_REACH - AT_SKIN) * height) * tone;
        let packed = ((height * STEPS) as u32) << 16 | (shade * STEPS) as u32;
        topmost[texel[1] * SIZE + texel[0]].fetch_max(packed.max(1), Ordering::Relaxed);
    });
}

/// Mark the blocks a segment of hair from `from` to `to`, texel positions,
/// passes over as shadowed.
fn shadow(shadowed: &[AtomicBool], from: [f32; 2], to: [f32; 2]) {
    let blocks = SIZE / SHADOW_BLOCK;
    trace(from, to, |[column, row], _| {
        let block = (row / SHADOW_BLOCK) * blocks + column / SHADOW_BLOCK;
        shadowed[block].store(true, Ordering::Relaxed);
    });
}

/// Visit each texel along the line from `from` to `to`, texel positions,
/// with how far along it lies, 0 to 1.
fn trace(from: [f32; 2], to: [f32; 2], mut visit: impl FnMut([usize; 2], f32)) {
    let ([x0, y0], [x1, y1]) = (from, to);
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let (column, row) = ((x0 + (x1 - x0) * t) as usize, (y0 + (y1 - y0) * t) as usize);
        if column < SIZE && row < SIZE {
            visit([column, row], t);
        }
    }
}

/// For each texel, the share of `mask` set within a square `2 * reach + 1`
/// across about it.
fn share_around(mask: &[bool], reach: usize) -> Vec<f32> {
    // Along rows, then along columns: `stride` apart, one texel or a row.
    let along = |values: &[f32], stride: usize| -> Vec<f32> {
        (0..values.len())
            .into_par_iter()
            .map(|texel| {
                let position = (texel / stride) % SIZE;
                let start = texel - position * stride;
                let first = position.saturating_sub(reach);
                let last = (position + reach).min(SIZE - 1);
                let sum: f32 = (first..=last)
                    .map(|other| values[start + other * stride])
                    .sum();
                sum / (last - first + 1) as f32
            })
            .collect()
    };
    let values: Vec<f32> = mask.iter().map(|&set| f32::from(u8::from(set))).collect();
    along(&along(&values, 1), SIZE)
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
