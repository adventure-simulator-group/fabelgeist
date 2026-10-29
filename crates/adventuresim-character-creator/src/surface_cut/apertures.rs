//! Authored through-cloth apertures remain separate from sewing boundaries.
use super::{CUT_VERTEX_SNAP_METRES, ConvexRegion, SurfaceCut, dot};
use anyhow::{Result, ensure};
use std::collections::BTreeSet;
/// Original canonical-body metre frame; topology indices are resolved after sampling.
#[derive(Clone, Debug)]
pub(crate) struct ClothAperture {
    pub region: ConvexRegion,
    pub center: [f32; 3],
    pub across: [f32; 3],
    pub up: [f32; 3],
}
impl ClothAperture {
    pub fn contains(&self, point: [f32; 3]) -> bool {
        self.region
            .iter()
            .all(|p| dot(point, p.normal) - p.offset <= CUT_VERTEX_SNAP_METRES)
    }
    pub fn project(&self, point: [f32; 3]) -> [f32; 2] {
        let delta = std::array::from_fn(|i| point[i] - self.center[i]);
        [dot(delta, self.across), dot(delta, self.up)]
    }
}
impl SurfaceCut {
    pub fn aperture_loops(&self, positions: &[[f32; 3]]) -> Result<Vec<Vec<u32>>> {
        let boundaries = self.boundary_loops(positions)?;
        let mut used = BTreeSet::new();
        self.apertures
            .iter()
            .map(|aperture| {
                let found = boundaries
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.iter().all(|&v| aperture.contains(positions[v as usize])))
                    .collect::<Vec<_>>();
                ensure!(
                    found.len() == 1,
                    "each cloth aperture must retain exactly one complete boundary"
                );
                ensure!(used.insert(found[0].0), "cloth apertures must not merge");
                Ok(found[0].1.clone())
            })
            .collect()
    }
    pub fn sewing_boundary_loops(&self, positions: &[[f32; 3]]) -> Result<Vec<Vec<u32>>> {
        let apertures = self.aperture_loops(positions)?;
        Ok(self
            .boundary_loops(positions)?
            .into_iter()
            .filter(|b| !apertures.iter().any(|a| a == b))
            .collect())
    }
}
