//! Space nothing grows into: buildings, as the solids of a
//! [`Structure`](fabelgeist_buildings::Structure).
//!
//! Solids enter the model where other plants already do, as occupied space:
//!
//! * **Crowding** (Eq. 1): a module's light falls with the share of its
//!   sphere inside others' -- and inside masonry. Near a surface the solid
//!   is taken for a half-space at its nearest point, so the share is the cap
//!   of the sphere beyond the plane ([`cap_fraction`]); a module against a
//!   wall loses the half of the sky the wall hides.
//! * **Orientation** (Eq. 3): the same share is part of `f_collisions`, so a
//!   new module turns away from a wall as from a neighbour, and a bud that
//!   can only grow into one stays dormant -- branches grow round a tower.
//! * **Shadow** (Sec. 6.2): a solid fills the shadow cells it occupies, and
//!   the shadow cast below and beside it spreads like a canopy's.
//! * **Soil**: nothing takes root under a building (the scene blocks its
//!   footprint in the terrain's soil mask).

use std::sync::Arc;

use fabelgeist_buildings::{SolidIndex, Structure};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::environment::shadow::{FIXED_POINT, ShadowGrid};

/// How far distance queries look: past the largest module sphere.
const MARGIN: f32 = 6.0;

/// Shadow a shadow cell wholly inside masonry casts, in the units modules
/// deposit in: stone is opaque, so several times a full cell of foliage.
pub const OPACITY: f32 = 2.0;

/// The solids of a scene, indexed.
#[derive(Debug)]
pub struct Obstacles {
    pub structure: Structure,
    pub index: SolidIndex,
}

/// The share of a sphere of radius `r` beyond a plane `d` from its centre
/// (negative when the centre is past the plane): the cap's volume,
/// `pi h^2 (3r - h) / 3` with `h = r - d`, over the sphere's.
pub fn cap_fraction(d: f32, r: f32) -> f32 {
    let h = (r - d).clamp(0.0, 2.0 * r);
    h * h * (3.0 * r - h) / (4.0 * r * r * r).max(1e-12)
}

impl Obstacles {
    pub fn new(structure: Structure) -> Arc<Self> {
        let index = SolidIndex::new(structure.solids.clone(), MARGIN);
        Arc::new(Self { structure, index })
    }

    /// Signed distance to the nearest solid, at most [`MARGIN`].
    pub fn distance(&self, p: Vec3) -> f32 {
        self.index.distance(p.to_array())
    }

    /// The way out of the nearest solid, if one is within [`MARGIN`].
    pub fn normal(&self, p: Vec3) -> Option<Vec3> {
        self.index.normal(p.to_array()).map(Vec3::from_array)
    }

    /// The share of a sphere inside the solids.
    pub fn crowding(&self, center: Vec3, radius: f32) -> f32 {
        cap_fraction(self.distance(center), radius)
    }

    /// The shadow the solids deposit in each cell of `grid`, in fixed point:
    /// [`OPACITY`] for a cell wholly inside them, fading out to nothing for
    /// one whose centre is only half a cell in.
    ///
    /// Shadow in the grid falls from above, so a cell straddling a wall's
    /// face would darken the whole face below it as if the wall stood over
    /// it; only the wall's core casts, and the face sees the half of the
    /// sky the cone of shadow from the core leaves it.
    pub fn occupancy(&self, grid: &ShadowGrid) -> Vec<u32> {
        let [nx, ny, nz] = grid.dims.map(|d| d as usize);
        let mut cells = vec![0u32; nx * ny * nz];
        cells.par_chunks_mut(nx).enumerate().for_each(|(row, out)| {
            let (y, z) = (row / nz, row % nz);
            for (x, cell) in out.iter_mut().enumerate() {
                let p = [
                    grid.origin[0] + (x as f32 + 0.5) * grid.cell,
                    grid.origin[1] + (y as f32 + 0.5) * grid.cell,
                    grid.origin[2] + (z as f32 + 0.5) * grid.cell,
                ];
                let inside = (-self.index.distance(p) / grid.cell - 0.5).clamp(0.0, 1.0);
                *cell = (inside * OPACITY * FIXED_POINT) as u32;
            }
        });
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cap_is_none_of_the_sphere_to_all_of_it() {
        assert_eq!(cap_fraction(2.0, 1.0), 0.0);
        assert!((cap_fraction(0.0, 1.0) - 0.5).abs() < 1e-6);
        assert!((cap_fraction(-1.0, 1.0) - 1.0).abs() < 1e-6);
        // A quarter radius past the plane: h = 1.25 of 1.
        let h = 1.25f32;
        assert!((cap_fraction(-0.25, 1.0) - h * h * (3.0 - h) / 4.0).abs() < 1e-6);
    }
}
