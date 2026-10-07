//! How much hair covers the skin: what makes a scalp read as a head of hair
//! and not as bald skin between strands.
//!
//! Hair covers the scalp two ways. It grows from it -- a hundred and more
//! follicles a square centimetre, each dark with pigment and with the first
//! millimetre of its hair -- and it lies over it, in layers, so the skin
//! beneath is in the hair's shadow. Strands drawn as ribbons only ever show
//! the second, sparsely: between them the skin shows as if lit, bare and
//! pale, at every part and whorl and wherever locks gather. Every hair
//! renderer paints the scalp under the hair for this; here it is measured
//! from the groom. [`ScalpCover::at`] gives, for points on the skin, how
//! densely roots grow around each and how much hair lies over it (the
//! chance a line straight out from the skin meets a strand in the first
//! couple of centimetres), and the head is shaded by it.

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::grid::PointGrid;
use crate::strands::Strands;

/// Around how far from a point of skin its roots count, metres.
const ROOT_RADIUS: f32 = 0.004;
/// The hair density grid's cell, metres.
const CELL: f32 = 0.004;
/// How far out from the skin hair counts as over it, metres.
const REACH: f32 = 0.024;

pub struct ScalpCover {
    roots: PointGrid,
    lo: Vec3,
    dims: [usize; 3],
    /// Hair's optical density per cell: projected strand area per volume,
    /// per metre -- the expected number of strands a line crosses a metre.
    density: Vec<f32>,
}

impl ScalpCover {
    pub fn new(strands: &Strands) -> Self {
        let roots = PointGrid::build(
            &(0..strands.len())
                .map(|s| (strands.root(s), 0))
                .collect::<Vec<_>>(),
            ROOT_RADIUS,
        );
        let [lo, hi] = strands.bounds();
        let lo = lo - Vec3::splat(REACH);
        let hi = hi + Vec3::splat(REACH);
        let extent = hi - lo;
        let dims = [extent.x, extent.y, extent.z].map(|e| ((e / CELL).ceil() as usize).max(1));
        let mut density = vec![0.0f32; dims[0] * dims[1] * dims[2]];
        let volume = CELL * CELL * CELL;
        for s in 0..strands.len() {
            let diameter = 2.0 * strands.radius[s].max(1e-6);
            for pair in strands.strand(s).windows(2) {
                let mid = (pair[0] + pair[1]) * 0.5;
                if let Some(i) = index(lo, dims, mid) {
                    density[i] += (pair[1] - pair[0]).length() * diameter / volume;
                }
            }
        }
        Self {
            roots,
            lo,
            dims,
            density,
        }
    }

    /// How much hair covers each point of skin (normal `normals`), each 0
    /// to 1: how densely roots grow around it -- by the share of the densest
    /// growth over these points -- and how much of it the hair over it
    /// hides. Roots colour the skin, as a scalp; hair over any skin shades
    /// it, a scalp or a forehead under a fringe.
    pub fn at(&self, points: &[Vec3], normals: &[Vec3]) -> Vec<[f32; 2]> {
        let counts: Vec<usize> = points
            .par_iter()
            .map(|&p| self.roots.count_within(p, ROOT_RADIUS))
            .collect();
        let mut growing: Vec<usize> = counts.iter().copied().filter(|&c| c > 0).collect();
        growing.sort_unstable();
        let full = growing
            .get(growing.len() * 4 / 5)
            .copied()
            .unwrap_or(1)
            .max(1) as f32;
        points
            .par_iter()
            .zip(normals)
            .zip(&counts)
            .map(|((&p, &n), &count)| {
                let grown = (count as f32 / full).min(1.0);
                let steps = 8;
                let ds = REACH / steps as f32;
                let depth: f32 = (0..steps)
                    .map(|k| {
                        let q = p + n * (0.002 + (k as f32 + 0.5) * ds);
                        self.density_at(q) * ds
                    })
                    .sum();
                let over = 1.0 - (-depth).exp();
                [grown, grown.max(over)]
            })
            .collect()
    }

    /// The hair's density at `p`, trilinear between cell centres.
    fn density_at(&self, p: Vec3) -> f32 {
        let q = (p - self.lo) * (1.0 / CELL) - Vec3::splat(0.5);
        let base = [q.x.floor(), q.y.floor(), q.z.floor()];
        let f = [q.x - base[0], q.y - base[1], q.z - base[2]];
        let mut sum = 0.0;
        for corner in 0..8 {
            let o = [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1];
            let c: [i64; 3] = std::array::from_fn(|k| base[k] as i64 + o[k] as i64);
            if c.iter()
                .zip(self.dims)
                .any(|(&v, d)| v < 0 || v >= d as i64)
            {
                continue;
            }
            let w: f32 = (0..3)
                .map(|k| if o[k] == 1 { f[k] } else { 1.0 - f[k] })
                .product();
            let [x, y, z] = c.map(|v| v as usize);
            sum += w * self.density[(z * self.dims[1] + y) * self.dims[0] + x];
        }
        sum
    }
}

fn index(lo: Vec3, dims: [usize; 3], p: Vec3) -> Option<usize> {
    let q = (p - lo) * (1.0 / CELL);
    let c = [q.x, q.y, q.z];
    if c.iter().zip(dims).any(|(&v, d)| v < 0.0 || v >= d as f32) {
        return None;
    }
    let [x, y, z] = c.map(|v| v as usize);
    Some((z * dims[1] + y) * dims[0] + x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strands::Attachment;

    #[test]
    fn skin_under_hair_is_covered_and_bare_skin_is_not() {
        // A mat of strands lying along a flat scalp at y = 0, x < 0.
        let mut strands = Strands::new(12);
        for i in 0..400 {
            let (x, z) = (-0.05 + (i % 20) as f32 * 0.0025, (i / 20) as f32 * 0.0025);
            let positions: Vec<Vec3> = (0..12)
                .map(|j| Vec3::new(x, 0.001, z + j as f32 * 0.004))
                .collect();
            strands.push(
                &positions,
                Attachment {
                    radius: 4e-5,
                    ..Default::default()
                },
            );
        }
        let cover = ScalpCover::new(&strands);
        let at = cover.at(
            &[Vec3::new(-0.025, 0.0, 0.02), Vec3::new(0.05, 0.0, 0.02)],
            &[Vec3::up(), Vec3::up()],
        );
        assert!(at[0][0] > 0.8 && at[0][1] > 0.8, "{at:?}");
        assert!(at[1][0] < 0.05 && at[1][1] < 0.05, "{at:?}");
    }
}
