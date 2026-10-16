//! The scalp's textures: the hair lying on it, baked from the groom.
//!
//! Painted one flat colour, the scalp shows between cards and strands as a
//! smooth, dead patch. Over a real scalp the eye meets the hair nearest it --
//! strands lying along the skin in the groom's own direction, parted where
//! the groom parts and spiralling out of its whorl. Each strand's first few
//! millimetres off the skin are drawn into the scalp's texture
//! ([`ScalpMap`]) on the bust the hair grew on, the topmost strand over each
//! texel showing: lighter the further it lies off the skin, darker where it
//! meets it, and running the way it runs, so the scalp is shaded as the
//! strands are. Single strands are far finer than a pixel at any distance
//! the head is seen from, so what shows is locks: strands rooted together
//! share a tone, lighter or darker than the next lock, and differ a little
//! among themselves.
//!
//! Where no strand lies, the scalp is either in the shadow of hair standing
//! further off it -- between an afro's coils, below the coils further out --
//! or bare: at a parting, at a whorl's eye, along a thinning hairline. How
//! bare the scalp is round each texel, over a couple of millimetres and
//! weighted to the nearest, makes the texel clear in the texture's alpha, so
//! the skin beneath the cap shows there, lit as the rest of the body: hair
//! thins into a parting over those millimetres, as it does at a real one,
//! its last strands lying over the skin, while a texel or two between coils
//! shows no skin at all. Shadowed scalp
//! is the hair's colour in shade.

use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use fabelgeist_hair::{Head, Strands};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::AtlasLevel;
use super::scalp_map::ScalpMap;

/// The texture's side, texels: about an eighth of a millimetre a texel,
/// near a hair's own width, so lying strands read as lines and a parting's
/// edge as strands' ends.
const SIZE: usize = 2048;
/// The longest step along a strand mapped as straight, metres: the map
/// curves, and a strand drawn straight between points a centimetre apart
/// would cut its corners.
const STEP: f32 = 0.002;
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
/// The side of the blocks shadowing is judged over, metres: one strand
/// passing over a block shadows all of it, as hair further off the skin
/// shadows more than the line of its own width.
const SHADOW_BLOCK: f32 = 0.0015;
/// How far round each texel the bare scalp is measured, metres, twice over
/// so the nearest counts most: hair thins into a parting over about twice
/// this.
const BARE_REACH: f32 = 0.0008;
/// The share of bare scalp round a texel at which skin starts to show
/// there, and shows fully: most of a parting is bare, the gaps within and
/// between an afro's coils single texels among strands.
const BARE_START: f32 = 0.1;
const BARE_FULL: f32 = 0.75;
/// How much of a strand still shows where the scalp round it is bare: at
/// a parting the hair thins strand by strand, the last strands' ends lying
/// over the skin, rather than fading as a whole.
const STRAND_OVER_SKIN: f32 = 0.5;
/// The scalp's shade where no strand lies on it but hair lies over it.
const SHADOWED: f32 = 0.35;
/// Bits of a texel's packed strand: the topmost strand wins, so its height
/// off the skin comes first, then its direction across the texture and its
/// shade.
const HEIGHT_BITS: u32 = 12;
const DIRECTION_BITS: u32 = 12;
const SHADE_BITS: u32 = 8;

/// The hair lying on the scalp, each a mip chain, largest first, over the
/// cap's texture coordinates.
pub struct ScalpTexture {
    /// The scalp's shade as grey, as a share of the hair's colour, and
    /// whether hair covers it as alpha.
    pub shade: Vec<AtlasLevel>,
    /// The direction the topmost strand runs across the texture, along `u`
    /// and `v`, each from 0 backwards to 1 forwards (a half each where no
    /// strand lies), and the shade again.
    pub strands: Vec<AtlasLevel>,
}

/// One texel's topmost strand.
#[derive(Clone, Copy)]
struct Topmost {
    /// Off the skin, as a share of [`REACH`].
    height: f32,
    /// Across the texture, radians from `+u` towards `+v`.
    direction: f32,
    /// As a share of the hair's colour.
    shade: f32,
}

impl Topmost {
    /// This strand in one value, ordered by height; never 0, which is no
    /// strand.
    fn pack(self) -> u32 {
        let turn = self.direction.rem_euclid(TAU) / TAU;
        (quantize(self.height, HEIGHT_BITS) << (DIRECTION_BITS + SHADE_BITS)
            | quantize(turn, DIRECTION_BITS) << SHADE_BITS
            | quantize(self.shade, SHADE_BITS))
        .max(1)
    }

    /// The strand a packed value holds, if any does.
    fn unpack(packed: u32) -> Option<Self> {
        (packed != 0).then(|| Self {
            height: field(packed, DIRECTION_BITS + SHADE_BITS, HEIGHT_BITS),
            direction: field(packed, SHADE_BITS, DIRECTION_BITS) * TAU,
            shade: field(packed, 0, SHADE_BITS),
        })
    }
}

/// `value`, 0 to 1, in `bits` bits.
fn quantize(value: f32, bits: u32) -> u32 {
    (value.clamp(0.0, 1.0) * ((1u32 << bits) - 1) as f32).round() as u32
}

/// The `bits` bits of `packed` from `shift` up, 0 to 1.
fn field(packed: u32, shift: u32, bits: u32) -> f32 {
    let most = (1u32 << bits) - 1;
    ((packed >> shift) & most) as f32 / most as f32
}

impl ScalpTexture {
    /// Bake `strands` grown on `head` into textures mapped by `map`.
    pub(super) fn bake(strands: &Strands, head: &Head, map: &ScalpMap) -> Self {
        let texel = map.texel(SIZE);
        let block_side = ((SHADOW_BLOCK / texel).round() as usize).max(1);
        let bare_reach = ((BARE_REACH / texel).round() as usize).max(1);
        let topmost: Vec<AtomicU32> = (0..SIZE * SIZE).map(|_| AtomicU32::new(0)).collect();
        let blocks = SIZE.div_ceil(block_side);
        let shadowed: Vec<AtomicBool> = (0..blocks * blocks)
            .map(|_| AtomicBool::new(false))
            .collect();
        (0..strands.len()).into_par_iter().for_each(|strand| {
            let lock = lock_noise(strands.root(strand) * (1.0 / LOCK_WIDTH));
            let tone = 1.0 - LOCK_SPREAD * lock - STRAND_SPREAD * unit(strands.id[strand]);
            let mut previous: Option<([f32; 2], f32)> = None;
            for point in steps(strands.strand(strand)) {
                let height = head.distance(point).max(0.0);
                let here = map
                    .uv(point)
                    .filter(|_| height < SHADOW_REACH)
                    .map(|[u, v]| ([u * SIZE as f32, v * SIZE as f32], height));
                if let (Some(from), Some(to)) = (previous, here) {
                    if from.1.min(to.1) < REACH {
                        draw(&topmost, from, to, tone);
                    } else {
                        shadow(&shadowed, from.0, to.0, block_side);
                    }
                }
                previous = here;
            }
        });
        let strands_at: Vec<Option<Topmost>> = topmost
            .iter()
            .map(|packed| Topmost::unpack(packed.load(Ordering::Relaxed)))
            .collect();
        let bare: Vec<f32> = strands_at
            .iter()
            .enumerate()
            .map(|(texel, strand)| {
                let (row, column) = (texel / SIZE, texel % SIZE);
                let block = (row / block_side) * blocks + column / block_side;
                let shaded = shadowed[block].load(Ordering::Relaxed);
                f32::from(u8::from(strand.is_none() && !shaded))
            })
            .collect();
        // Twice over, so the nearest scalp counts most and the share falls
        // smoothly rather than in a straight ramp.
        let bare_share = mean_around(&mean_around(&bare, bare_reach), bare_reach);
        let (shade, strand_texels): (Vec<[u8; 4]>, Vec<[u8; 4]>) = strands_at
            .iter()
            .zip(bare_share)
            .map(|(strand, share)| {
                let t = ((share - BARE_START) / (BARE_FULL - BARE_START)).clamp(0.0, 1.0);
                let skin = t * t * (3.0 - 2.0 * t);
                let tone = strand.map_or(SHADOWED, |strand| strand.shade);
                let clear = if strand.is_some() {
                    1.0 - STRAND_OVER_SKIN
                } else {
                    1.0
                };
                let [along_u, along_v] = strand.map_or([0.0; 2], |strand| {
                    [strand.direction.cos(), strand.direction.sin()]
                });
                let grey = to_byte(tone);
                (
                    [grey, grey, grey, to_byte(1.0 - skin * clear)],
                    [
                        to_byte(along_u * 0.5 + 0.5),
                        to_byte(along_v * 0.5 + 0.5),
                        grey,
                        u8::MAX,
                    ],
                )
            })
            .unzip();
        Self {
            shade: mip_chain(full_size(shade)),
            strands: mip_chain(full_size(strand_texels)),
        }
    }
}

/// `value`, 0 to 1, as a byte.
fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// The texture's largest level, of `texels`.
fn full_size(texels: Vec<[u8; 4]>) -> AtlasLevel {
    AtlasLevel {
        width: SIZE as u32,
        height: SIZE as u32,
        rgba: texels.into_flattened(),
    }
}

/// `base` and every level halved from it, down to a single texel.
fn mip_chain(base: AtlasLevel) -> Vec<AtlasLevel> {
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
    let direction = (to.0[1] - from.0[1]).atan2(to.0[0] - from.0[0]);
    trace(from.0, to.0, |texel, t| {
        let height = ((h0 + (h1 - h0) * t) / REACH).min(1.0);
        let strand = Topmost {
            height,
            direction,
            shade: (AT_SKIN + (AT_REACH - AT_SKIN) * height) * tone,
        };
        topmost[texel[1] * SIZE + texel[0]].fetch_max(strand.pack(), Ordering::Relaxed);
    });
}

/// Mark the blocks, `side` texels square, a segment of hair from `from` to
/// `to`, texel positions, passes over as shadowed.
fn shadow(shadowed: &[AtomicBool], from: [f32; 2], to: [f32; 2], side: usize) {
    let blocks = SIZE.div_ceil(side);
    trace(from, to, |[column, row], _| {
        let block = (row / side) * blocks + column / side;
        shadowed[block].store(true, Ordering::Relaxed);
    });
}

/// `strand`'s points, with more between any two further apart than
/// [`STEP`].
fn steps(strand: &[Vec3]) -> impl Iterator<Item = Vec3> + '_ {
    let first = strand.first().copied();
    first.into_iter().chain(strand.windows(2).flat_map(|pair| {
        let pieces = ((pair[1] - pair[0]).length() / STEP).ceil().max(1.0) as usize;
        (1..=pieces).map(move |piece| pair[0].lerp(pair[1], piece as f32 / pieces as f32))
    }))
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

/// For each texel, the mean of `values` within a square `2 * reach + 1`
/// across about it.
fn mean_around(values: &[f32], reach: usize) -> Vec<f32> {
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
    along(&along(values, 1), SIZE)
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
