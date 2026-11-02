//! Global shadowing (Sec. 6.2): a voxel grid of shadow cast downwards.
//!
//! Every module deposits a shadow proportional to its size into the cell its
//! centre falls in. The shadow then travels down one layer at a time, each
//! layer seeing the layer above (its shadow plus its deposits) through a
//! normalised 3x3 blur and a decay `b`:
//!
//! ```text
//! S(x, y, z) = b * sum_{dx,dz} k(dx, dz) (S + D)(x + dx, y + 1, z + dz)
//! ```
//!
//! That is the pyramid of Palubicki et al. (2009) as a recurrence -- the cone
//! widens by one cell per layer -- except that a lone occluder's shadow
//! spreads out as it widens while a closed canopy's does not, which is what
//! distinguishes a gap from a forest floor. The cone runs all the way to the
//! bottom layer, as the paper extends it. A module sees `Q_G = exp(-S)` at its
//! centre.
//!
//! Solids ([`crate::obstacles`]) fill the cells they occupy before any module
//! deposits: [`ShadowGrid::occluders`].

use std::sync::Arc;

use rayon::prelude::*;

/// The 3x3 kernel: centre, edge, corner. Sums to one.
pub const KERNEL: [f32; 3] = [0.5, 1.0 / 12.0, 1.0 / 24.0];

/// Deposits are accumulated as fixed point on the GPU; the host rounds the
/// same way so the two agree.
pub const FIXED_POINT: f32 = 1024.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowGrid {
    /// Lower corner of the grid.
    pub origin: [f32; 3],
    /// Edge of a cell, metres.
    pub cell: f32,
    /// Cells along x, y, z.
    pub dims: [u32; 3],
    /// Shadow a module deposits, per unit of its sphere's volume relative to
    /// a cell (clamped to one cell's worth).
    pub strength: f32,
    /// `b`: attenuation per layer travelled.
    pub decay: f32,
    /// What stands in the grid whatever grows -- buildings -- deposited per
    /// cell in fixed point, or nothing.
    pub occluders: Option<Arc<Vec<u32>>>,
}

impl ShadowGrid {
    pub fn cell_count(&self) -> usize {
        self.dims.iter().map(|&d| d as usize).product()
    }

    pub fn index(&self, x: u32, y: u32, z: u32) -> usize {
        ((y as usize * self.dims[2] as usize) + z as usize) * self.dims[0] as usize + x as usize
    }

    /// The cell a point falls in, if it falls in one.
    pub fn cell_of(&self, p: [f32; 3]) -> Option<[u32; 3]> {
        let inverse = 1.0 / self.cell;
        let mut cell = [0u32; 3];
        for k in 0..3 {
            let c = ((p[k] - self.origin[k]) * inverse).floor();
            if c < 0.0 || c >= self.dims[k] as f32 {
                return None;
            }
            cell[k] = c as u32;
        }
        Some(cell)
    }

    /// What one sphere deposits, in fixed point.
    pub fn deposit(&self, radius: f32) -> u32 {
        let volume = fabelgeist_spatial_hash::sphere_volume(radius);
        let amount = self.strength * (volume / (self.cell * self.cell * self.cell)).min(1.0);
        (amount * FIXED_POINT) as u32
    }

    /// `Q_G` at `p`, given the propagated shadow. Above the grid is open sky;
    /// beside it, too; below it, the bottom layer.
    pub fn light_at(&self, shadow: &[f32], p: [f32; 3]) -> f32 {
        let inverse = 1.0 / self.cell;
        let x = ((p[0] - self.origin[0]) * inverse).floor();
        let y = ((p[1] - self.origin[1]) * inverse).floor();
        let z = ((p[2] - self.origin[2]) * inverse).floor();
        if x < 0.0
            || z < 0.0
            || x >= self.dims[0] as f32
            || z >= self.dims[2] as f32
            || y >= self.dims[1] as f32
        {
            return 1.0;
        }
        let y = y.max(0.0);
        (-shadow[self.index(x as u32, y as u32, z as u32)]).exp()
    }

    /// Deposit and propagate on the host. Returns the shadow per cell.
    pub fn propagate(&self, spheres: &[[f32; 4]]) -> Vec<f32> {
        let mut density = match &self.occluders {
            Some(occluders) if occluders.len() == self.cell_count() => occluders.to_vec(),
            _ => vec![0u32; self.cell_count()],
        };
        for s in spheres {
            if let Some([x, y, z]) = self.cell_of([s[0], s[1], s[2]]) {
                let index = self.index(x, y, z);
                density[index] = density[index].wrapping_add(self.deposit(s[3]));
            }
        }

        let [nx, ny, nz] = self.dims.map(|d| d as usize);
        let layer = nx * nz;
        let mut shadow = vec![0.0f32; self.cell_count()];
        for y in (0..ny.saturating_sub(1)).rev() {
            let (below, above) = shadow.split_at_mut((y + 1) * layer);
            let target = &mut below[y * layer..];
            let above = &above[..layer];
            let deposits = &density[(y + 1) * layer..(y + 2) * layer];
            target.par_chunks_mut(nx).enumerate().for_each(|(z, row)| {
                for (x, out) in row.iter_mut().enumerate() {
                    let mut sum = 0.0;
                    for dz in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let (sx, sz) = (x as i32 + dx, z as i32 + dz);
                            if sx < 0 || sz < 0 || sx >= nx as i32 || sz >= nz as i32 {
                                continue;
                            }
                            let i = sz as usize * nx + sx as usize;
                            let w = KERNEL[(dx.abs() + dz.abs()) as usize];
                            sum += w * (above[i] + deposits[i] as f32 / FIXED_POINT);
                        }
                    }
                    *out = self.decay * sum;
                }
            });
        }
        shadow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> ShadowGrid {
        ShadowGrid {
            origin: [-8.0, 0.0, -8.0],
            cell: 1.0,
            dims: [16, 12, 16],
            strength: 1.0,
            decay: 0.95,
            occluders: None,
        }
    }

    #[test]
    fn an_occluder_shades_what_is_below_it_and_not_above() {
        let grid = grid();
        let shadow = grid.propagate(&[[0.5, 8.5, 0.5, 1.0]]);
        let below = grid.light_at(&shadow, [0.5, 6.5, 0.5]);
        let above = grid.light_at(&shadow, [0.5, 10.5, 0.5]);
        let aside = grid.light_at(&shadow, [6.5, 3.5, 6.5]);
        assert!(below < 0.9, "{below}");
        assert_eq!(above, 1.0);
        assert!(aside > below);
    }

    #[test]
    fn a_closed_canopy_shades_more_than_a_lone_tree() {
        let grid = grid();
        let lone = grid.propagate(&[[0.5, 8.5, 0.5, 1.0]]);
        let mut canopy = Vec::new();
        for x in -6..6 {
            for z in -6..6 {
                canopy.push([x as f32 + 0.5, 8.5, z as f32 + 0.5, 1.0]);
            }
        }
        let closed = grid.propagate(&canopy);
        let p = [0.5, 1.5, 0.5];
        assert!(grid.light_at(&closed, p) < grid.light_at(&lone, p) * 0.7);
    }
}
