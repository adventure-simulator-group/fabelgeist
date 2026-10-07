//! The hair node stack.
//!
//! DiffLocks' dataset pipeline is a stack of Blender geometry nodes over a
//! few dozen hand-placed guide strands: interpolate them into ~100K strands
//! across the scalp, then clump, curl, frizz and noise them -- each several
//! times at different scales and strengths -- shrinkwrap them off the body,
//! and drape them under gravity and wind. The node groups it uses are
//! Blender's procedural hair assets, some in revised copies (`Interpolate
//! Hair Curves V2`, `Clump Hair Curves_V2`, `Create Guide Index Map`,
//! `Blend Hair Curves`, `Shrinkwrap Hair Curves`, `Restore Curve Segment
//! Length`, `Set Hair Curve Profile`).
//!
//! Each operation here is one of those groups, written out from its node
//! graph: the same inputs under the same names, the same fields sampled from
//! the same places. [`Op`] is one entry of a stack; [`crate::Groom`] runs a
//! stack. The operations share a few pieces the node groups share:
//! [`shape_range`] (the `shape_range` group every falloff goes through),
//! [`GuideMap`] (`Create Guide Index Map`: which strand leads each clump)
//! and [`restore_lengths`] (`Restore Curve Segment Length`).
//!
//! Every operation reads its input from a snapshot and writes a strand per
//! task, so a stack runs in parallel over strands.

mod clump;
mod curl;
mod displace;
mod interpolate;
mod orient;
mod settle;
mod shrinkwrap;
mod trim;

pub use clump::Clump;
pub use curl::Curl;
pub use displace::{Frizz, Noise, Smooth, Wave};
pub use interpolate::{DensityMask, Interpolate, Parting};
pub use orient::Orient;
pub use settle::{STANDARD_GRAVITY, Settle};
pub use shrinkwrap::Shrinkwrap;
pub use trim::Trim;

use fabelgeist_math::Vec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::collider::Collider;
use crate::grid::PointGrid;
use crate::rng;
use crate::strands::{CurveSampler, Strands};
use crate::surface::Surface;

/// What operations may look at besides the strands.
pub struct Context<'a> {
    /// The surface strands grow from.
    pub scalp: &'a Surface,
    /// What strands stay out of.
    pub collider: &'a dyn Collider,
}

/// One operation of a groom's stack.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Interpolate(Interpolate),
    Orient(Orient),
    Trim(Trim),
    Clump(Clump),
    Curl(Curl),
    Wave(Wave),
    Frizz(Frizz),
    Noise(Noise),
    Smooth(Smooth),
    Shrinkwrap(Shrinkwrap),
    Settle(Settle),
    Profile(Profile),
}

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Op::Interpolate(_) => "Interpolate",
            Op::Orient(_) => "Orient",
            Op::Trim(_) => "Trim",
            Op::Clump(_) => "Clump",
            Op::Curl(_) => "Curl",
            Op::Wave(_) => "Wave",
            Op::Frizz(_) => "Frizz",
            Op::Noise(_) => "Noise",
            Op::Smooth(_) => "Smooth",
            Op::Shrinkwrap(_) => "Shrinkwrap",
            Op::Settle(_) => "Settle",
            Op::Profile(_) => "Profile",
        }
    }

    /// Run on `strands`. Interpolation replaces them (guides in, the full
    /// groom out); everything else moves points.
    pub fn apply(&self, strands: &mut Strands, context: &Context) {
        match self {
            Op::Interpolate(op) => *strands = op.apply(strands, context),
            Op::Orient(op) => op.apply(strands),
            Op::Trim(op) => op.apply(strands),
            Op::Clump(op) => op.apply(strands),
            Op::Curl(op) => op.apply(strands),
            Op::Wave(op) => op.apply(strands),
            Op::Frizz(op) => op.apply(strands),
            Op::Noise(op) => op.apply(strands),
            Op::Smooth(op) => op.apply(strands),
            Op::Shrinkwrap(op) => op.apply(strands, context),
            Op::Settle(op) => op.apply(strands, context),
            Op::Profile(op) => op.apply(strands),
        }
    }
}

/// `Set Hair Curve Profile`, as far as the strands carry it: each strand's
/// root radius, with some variation. How the radius falls off to the tip is
/// the material's ([`crate::HairMaterial::taper`]), applied where strands
/// are drawn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    /// Root radius, metres. Human hair is 17 to 90 microns across.
    pub radius: f32,
    /// Each strand's radius is scaled by up to this much either way.
    pub random: f32,
    pub seed: i32,
    /// Scale each strand's radius by `radius` over the one it was grown
    /// with instead of setting it, keeping what interpolation made finer
    /// (the hairline's fine hair).
    #[serde(default)]
    pub relative_to: Option<f32>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            radius: 35e-6,
            random: 0.25,
            seed: 0,
            relative_to: None,
        }
    }
}

impl Profile {
    pub fn apply(&self, strands: &mut Strands) {
        let seed = rng::seed(self.seed, 51_823);
        for (r, &id) in strands.radius.iter_mut().zip(&strands.id) {
            let base = match self.relative_to {
                Some(grown) if grown > 0.0 => self.radius * *r / grown,
                _ => self.radius,
            };
            *r = base * (1.0 + self.random * rng::signed(id, seed)).max(0.05);
        }
    }
}

/// The `shape_range` node group: `value` mapped from `[min, max]` to `[0,
/// 1]`, bent by `shape`, and mapped back. `shape` 0 is constant 1 (the
/// effect everywhere), 0.5 is linear, towards 1 the effect gathers at the
/// top of the range; negative shapes mirror it. `base` sets how strongly
/// the shape bends (2 in every use).
pub fn shape_range(value: f32, min: f32, max: f32, shape: f32, base: f32) -> f32 {
    if shape.abs() < 1e-5 {
        return 1.0;
    }
    let x = if (max - min).abs() > 1e-12 {
        (value - min) / (max - min)
    } else {
        0.0
    };
    let b = base.max(1.00001);
    let a = shape.abs().min(0.999999);
    let negative = shape < 0.0;
    let flip = if a > 0.5 { !negative } else { negative };
    let xx = if flip { 1.0 - x } else { x };
    let pingpong = if a <= 0.5 { a } else { 1.0 - a };
    let m = pingpong * 2.0 * (1.0 - 1.0 / b);
    let exponent = -(1.0 - m).max(1e-5).ln() / b.max(1.000_001).ln();
    let y = xx.max(0.0).powf(exponent);
    let y = if a > 0.5 { 1.0 - y } else { y };
    (min + y.clamp(0.0, 1.0) * (max - min)).clamp(min.min(max), min.max(max))
}

/// Blender's `Map Range` with smootherstep interpolation, clamped.
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = if (edge1 - edge0).abs() > 1e-12 {
        ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Blender's `Map Range` with smoothstep interpolation, clamped.
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = if (edge1 - edge0).abs() > 1e-12 {
        ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    t * t * (3.0 - 2.0 * t)
}

/// Which strand leads each strand's clump: `Create Guide Index Map`.
///
/// The node group scatters the roots, keeps each with probability `mask`,
/// merges them by `distance` -- what is left are the clump guides, spaced at
/// least that far apart -- and gives every strand the index of the guide
/// nearest its root, within its parting group. A distance of zero makes
/// every strand its own guide.
#[derive(Clone, Debug)]
pub struct GuideMap {
    /// For each strand, its guide's strand index.
    pub guide: Vec<u32>,
    /// The guides, in the order they were chosen.
    pub guides: Vec<u32>,
}

impl GuideMap {
    pub fn new(strands: &Strands, distance: f32, mask: f32, seed: u32) -> Self {
        let n = strands.len();
        if distance <= 0.0 || n == 0 {
            return Self {
                guide: (0..n as u32).collect(),
                guides: (0..n as u32).collect(),
            };
        }
        // Candidates in a random order; each kept with probability `mask`.
        let mut order: Vec<u32> = (0..n as u32).collect();
        order.sort_by_key(|&s| rng::hash(strands.id[s as usize], seed));
        let mut grid = PointGrid::new(distance);
        let mut guides = Vec::new();
        for &s in &order {
            if rng::unit(strands.id[s as usize], seed ^ 0x2F0B_1A3D) >= mask {
                continue;
            }
            let root = strands.root(s as usize);
            let group = strands.group[s as usize];
            if !grid.any_within(root, distance, |g| g == group) {
                grid.insert(root, group);
                guides.push(s);
            }
        }
        if guides.is_empty() {
            let s = order[0];
            grid.insert(strands.root(s as usize), strands.group[s as usize]);
            guides.push(s);
        }
        let guide = (0..n)
            .into_par_iter()
            .map(|s| {
                let root = strands.root(s);
                let group = strands.group[s];
                let nearest = grid
                    .nearest(root, distance * 8.0, |g| g == group)
                    .or_else(|| grid.nearest(root, f32::INFINITY, |_| true));
                nearest.map_or(s as u32, |(i, _)| guides[i as usize])
            })
            .collect();
        Self { guide, guides }
    }

    /// The guide map `strands` carry from an earlier operation, if asked to
    /// reuse it and there is one; otherwise a new one, which the strands
    /// then carry (Blender stores it as `guide_curve_index`).
    pub fn get(strands: &mut Strands, reuse: bool, distance: f32, mask: f32, seed: u32) -> Self {
        if reuse && let Some(map) = &strands.guide_map {
            return map.clone();
        }
        let map = Self::new(strands, distance, mask, seed);
        strands.guide_map = Some(map.clone());
        map
    }

    /// A sampler for every guide, indexed by strand (only guides filled).
    pub fn samplers(&self, strands: &Strands) -> Vec<Option<CurveSampler>> {
        let mut is_guide = vec![false; strands.len()];
        for &g in &self.guides {
            is_guide[g as usize] = true;
        }
        (0..strands.len())
            .into_par_iter()
            .map(|s| is_guide[s].then(|| CurveSampler::new(strands.strand(s))))
            .collect()
    }
}

/// `Restore Curve Segment Length`: keep each segment's current direction
/// and give it back (by `factor`) the length it has in `reference`, walking
/// out from the root.
pub fn restore_lengths(strand: &mut [Vec3], reference: &[Vec3], factor: f32) {
    let mut previous = strand[0];
    let mut previous_old = strand[0];
    for i in 1..strand.len() {
        let current = strand[i];
        let direction = current - previous_old;
        let current_length = direction.length();
        let rest = (reference[i] - reference[i - 1]).length();
        let length = current_length + (rest - current_length) * factor;
        let direction = if current_length > 1e-12 {
            direction * (1.0 / current_length)
        } else {
            (reference[i] - reference[i - 1]).normalize()
        };
        previous_old = current;
        strand[i] = previous + direction * length;
        previous = strand[i];
    }
}

/// Per point: its factor along the strand (arc length over total) and its
/// arc length.
pub(crate) fn parameters(strand: &[Vec3], lengths: &mut Vec<f32>, factors: &mut Vec<f32>) {
    crate::strands::arc_lengths(strand, lengths);
    let total = lengths.last().copied().unwrap_or(0.0).max(1e-12);
    factors.clear();
    factors.extend(lengths.iter().map(|l| l / total));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_range_spans_constant_linear_and_late() {
        assert_eq!(shape_range(0.3, 0.0, 1.0, 0.0, 2.0), 1.0);
        for x in [0.0, 0.25, 0.5, 1.0] {
            assert!((shape_range(x, 0.0, 1.0, 0.5, 2.0) - x).abs() < 1e-4);
        }
        // Towards 1 the effect waits for the end.
        assert!(shape_range(0.5, 0.0, 1.0, 0.95, 2.0) < 0.1);
        assert!(shape_range(1.0, 0.0, 1.0, 0.95, 2.0) > 0.99);
        // Small shapes act early.
        assert!(shape_range(0.2, 0.0, 1.0, 0.1, 2.0) > 0.7);
    }

    #[test]
    fn restoring_lengths_undoes_a_stretch() {
        let reference: Vec<Vec3> = (0..5).map(|i| Vec3::new(0.0, i as f32, 0.0)).collect();
        let mut stretched: Vec<Vec3> = reference.iter().map(|p| *p * 1.5).collect();
        restore_lengths(&mut stretched, &reference, 1.0);
        for (a, b) in stretched.iter().zip(&reference) {
            assert!((*a - *b).length() < 1e-5);
        }
    }
}
