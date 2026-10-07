//! Baked occlusion: how much of the sky a point inside the hair sees.
//!
//! Real-time shadowing of hair by hair is deep opacity maps
//! ([`crate::render`]), for one light. Everything else -- the sky, fill
//! lights, and on mobile the key light too -- is darkened by a volume baked
//! once per groom: the hair's optical density on a grid (each strand
//! segment's projected area in each cell, the body's cells opaque), and in
//! each cell the transmittance averaged over directions around it. A point
//! deep in a bun sees little of the sky; a flyaway sees all of it. The value
//! rides along with every point to the GPU, so it costs nothing to draw.

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::collider::Collider;
use crate::strands::Strands;

pub struct Occlusion {
    lo: Vec3,
    cell: f32,
    dims: [usize; 3],
    /// Sky visibility per cell, 0 to 1.
    visibility: Vec<f32>,
}

/// How dark hair is to itself.
#[derive(Clone, Copy, Debug)]
pub struct OcclusionSettings {
    /// Cells along the longest side of the hair's box.
    pub resolution: usize,
    /// Scales the optical density: hair scatters most light onwards rather
    /// than absorbing it, so a physical extinction darkens far too much.
    pub strength: f32,
    /// Directions averaged per cell.
    pub directions: usize,
}

impl Default for OcclusionSettings {
    fn default() -> Self {
        Self {
            resolution: 64,
            strength: 0.2,
            directions: 16,
        }
    }
}

impl Occlusion {
    pub fn bake(
        strands: &Strands,
        body: Option<&dyn Collider>,
        settings: &OcclusionSettings,
    ) -> Self {
        let [lo, hi] = strands.bounds();
        let pad = Vec3::splat(0.02);
        let (lo, hi) = (lo - pad, hi + pad);
        let extent = hi - lo;
        let cell = extent.max_component() / settings.resolution.max(4) as f32;
        let dims = [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as usize).max(1));
        let count = dims[0] * dims[1] * dims[2];
        let index = |c: [usize; 3]| (c[2] * dims[1] + c[1]) * dims[0] + c[0];
        let cell_of = |p: Vec3| -> Option<[usize; 3]> {
            let q = (p - lo) * (1.0 / cell);
            let c = [q.x, q.y, q.z];
            if c.iter().zip(dims).any(|(&v, d)| v < 0.0 || v >= d as f32) {
                return None;
            }
            Some(c.map(|v| v as usize))
        };

        // Optical density: projected area per volume, per metre.
        let mut density = vec![0.0f32; count];
        let volume = cell * cell * cell;
        for s in 0..strands.len() {
            let diameter = 2.0 * strands.radius[s].max(1e-6);
            for pair in strands.strand(s).windows(2) {
                let mid = (pair[0] + pair[1]) * 0.5;
                if let Some(c) = cell_of(mid) {
                    density[index(c)] += (pair[1] - pair[0]).length() * diameter / volume;
                }
            }
        }
        for d in density.iter_mut() {
            *d *= settings.strength;
        }
        if let Some(body) = body {
            density.par_iter_mut().enumerate().for_each(|(i, d)| {
                let c = [
                    i % dims[0],
                    (i / dims[0]) % dims[1],
                    i / (dims[0] * dims[1]),
                ];
                let p =
                    lo + Vec3::new(c[0] as f32 + 0.5, c[1] as f32 + 0.5, c[2] as f32 + 0.5) * cell;
                if body.distance(p) < 0.0 {
                    *d = 1e4;
                }
            });
        }

        // Directions spread over the sphere (a Fibonacci lattice).
        let n = settings.directions.max(4);
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        let directions: Vec<Vec3> = (0..n)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
                let r = (1.0 - y * y).sqrt();
                let a = golden * i as f32;
                Vec3::new(r * a.cos(), y, r * a.sin())
            })
            .collect();
        let steps = settings.resolution.max(4);
        // Only cells next to hair are ever looked up.
        let mut needed = vec![false; count];
        for (i, &d) in density.iter().enumerate() {
            if d <= 0.0 || d >= 1e4 {
                continue;
            }
            let c = [
                i % dims[0],
                (i / dims[0]) % dims[1],
                i / (dims[0] * dims[1]),
            ];
            for dz in 0..3 {
                for dy in 0..3 {
                    for dx in 0..3 {
                        let n = [c[0] + dx, c[1] + dy, c[2] + dz];
                        if n.iter().zip(dims).all(|(&v, d)| v >= 1 && v <= d) {
                            needed[index([n[0] - 1, n[1] - 1, n[2] - 1])] = true;
                        }
                    }
                }
            }
        }
        let visibility = (0..count)
            .into_par_iter()
            .map(|i| {
                if !needed[i] {
                    return 1.0;
                }
                let c = [
                    i % dims[0],
                    (i / dims[0]) % dims[1],
                    i / (dims[0] * dims[1]),
                ];
                let start =
                    lo + Vec3::new(c[0] as f32 + 0.5, c[1] as f32 + 0.5, c[2] as f32 + 0.5) * cell;
                let mut sum = 0.0;
                for d in &directions {
                    let mut depth = 0.0;
                    for k in 1..steps {
                        let p = start + *d * (k as f32 * cell);
                        let Some(c) = cell_of(p) else { break };
                        depth += density[index(c)] * cell;
                        if depth > 8.0 {
                            break;
                        }
                    }
                    // Light from below the horizon of the body counts less:
                    // weight directions towards the sky.
                    let weight = 0.5 + 0.5 * d.y.max(0.0);
                    sum += (-depth).exp() * weight;
                }
                let norm: f32 = directions.iter().map(|d| 0.5 + 0.5 * d.y.max(0.0)).sum();
                sum / norm
            })
            .collect();
        Self {
            lo,
            cell,
            dims,
            visibility,
        }
    }

    /// Sky visibility at `p`, trilinear between cell centres.
    pub fn at(&self, p: Vec3) -> f32 {
        let q = (p - self.lo) * (1.0 / self.cell) - Vec3::splat(0.5);
        let base = [q.x.floor(), q.y.floor(), q.z.floor()];
        let f = [q.x - base[0], q.y - base[1], q.z - base[2]];
        let mut sum = 0.0;
        for corner in 0..8 {
            let o = [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1];
            let c: [usize; 3] = std::array::from_fn(|k| {
                (base[k] as i64 + o[k] as i64).clamp(0, self.dims[k] as i64 - 1) as usize
            });
            let w: f32 = (0..3)
                .map(|k| if o[k] == 1 { f[k] } else { 1.0 - f[k] })
                .product();
            sum += w * self.visibility[(c[2] * self.dims[1] + c[1]) * self.dims[0] + c[0]];
        }
        sum
    }
}
