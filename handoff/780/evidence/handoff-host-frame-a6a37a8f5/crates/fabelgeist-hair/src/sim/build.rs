//! Admit simulation source, guide binding and host work-array extents.

use super::{HairSim, Rigid, SimBinding, SimSettings};
use crate::grid::PointGrid;
use crate::identity::PartingGroup;
use crate::lod::{HairLods, StrandLevel};
use crate::parameters::{PointsPerStrand, StrandCount};
use crate::strands::{StrandShape, StrandShapeError};
use fabelgeist_math::Vec3;
use rayon::prelude::*;
use std::sync::Arc;

pub type SimResult<T> = Result<T, SimError>;

#[derive(Debug, thiserror::Error)]
pub enum SimError {
    #[error("invalid simulated strand shape")]
    Shape(#[from] StrandShapeError),
    #[error("simulated population {requested:?} exceeds guide index addresses")]
    GuideCount { requested: StrandCount, #[source] source: std::num::TryFromIntError },
    #[error("simulated {layout:?} exceeds doubled rod work-array addresses")]
    RodExtent { layout: StrandShape },
    #[error("host simulation allocation failed")]
    Allocation(#[from] std::collections::TryReserveError),
}

impl HairSim {
    /// Simulate a prefix at the nearest available point count, retaining both
    /// the actual first drawn level and the selected sampling level.
    pub fn new(lods: &HairLods, settings: SimSettings) -> SimResult<Self> {
        // HairLods' sole builder always creates its original level first.
        let source = &lods.levels()[0];
        let mut sampled = source;
        let wanted = settings.points.count();
        for level in &lods.levels()[1..] {
            if level.layout().points().count().abs_diff(wanted)
                < sampled.layout().points().count().abs_diff(wanted) {
                sampled = level;
            }
        }
        Self::from_sources(source.clone(), sampled.clone(), settings)
    }

    /// Simulate a checked host level, including one read back from a device.
    pub fn from_level(source: Arc<StrandLevel>, settings: SimSettings) -> SimResult<Self> {
        Self::from_sources(source.clone(), source, settings)
    }

    fn from_sources(
        source: Arc<StrandLevel>,
        sampled: Arc<StrandLevel>,
        settings: SimSettings,
    ) -> SimResult<Self> {
        let available = sampled.layout().strands();
        let minimum = if sampled.is_empty() { 0 } else { 1 };
        let count = settings.guides.min(available).max(StrandCount::from_count(minimum));
        // Even an empty source has a positive numerical chunk stride.
        let points = sampled.layout().points().max(PointsPerStrand::from_count(2));
        let layout = StrandShape::new(count, points)?;
        u32::try_from(count.count()).map_err(|source| SimError::GuideCount {
            requested: count, source,
        })?;
        layout.points().count().checked_mul(2).ok_or(SimError::RodExtent { layout })?;
        let slots = layout.samples().count().checked_mul(2)
            .ok_or(SimError::RodExtent { layout })?;
        let mut rest = reserved(layout.samples().count())?;
        rest.extend(sampled.positions()[..layout.samples().count()].iter()
            .map(|p| Vec3::new(p[0], p[1], p[2])));
        let mut position = reserved(rest.len())?;
        position.extend_from_slice(&rest);
        let mut previous = reserved(rest.len())?;
        previous.extend_from_slice(&rest);
        let mut shape = reserved(slots)?;
        shape.resize(slots, [0.0; 4]);
        let mut frames = reserved(slots)?;
        frames.resize(slots, [0.0; 4]);
        let binding = Arc::new(SimBinding::build(source, sampled, layout, &rest)?);
        Ok(Self {
            settings, binding, position, previous, shape, frames, rest,
            shaped: None, h_last: 1.0 / 240.0, head: Rigid::default(), time: 0.0,
        })
    }
}

impl SimBinding {
    fn build(
        source: Arc<StrandLevel>,
        sampled: Arc<StrandLevel>,
        layout: StrandShape,
        rest: &[Vec3],
    ) -> SimResult<Self> {
        let count = layout.strands().count();
        let points = layout.points().count();
        let total = source.layout().strands().count();
        let source_points = source.layout().points().count();
        let root = |s: usize| {
            let p = source.positions()[s * source_points];
            Vec3::new(p[0], p[1], p[2])
        };
        let mut roots = reserved(count)?;
        roots.extend((0..count).map(|g| (rest[g * points], PartingGroup::Unassigned)));
        let spacing = 0.2 / (count as f32).sqrt().max(1.0);
        let grid = PointGrid::build(&roots, spacing.max(0.003));
        let mut guides = reserved(total)?;
        guides.resize(total, [0u32; 4]);
        let mut weights = reserved(total)?;
        weights.resize(total, [0.0f32; 4]);
        guides.par_iter_mut().zip(weights.par_iter_mut()).enumerate()
            .for_each(|(s, (ids, weights))| {
                if s < count {
                    *ids = [s as u32, 0, 0, 0];
                    *weights = [1.0, 0.0, 0.0, 0.0];
                    return;
                }
                let nearest = grid.nearest_k(root(s), 4, f32::INFINITY, |_| true);
                let mut total = 0.0;
                for (k, &(g, d)) in nearest.iter().enumerate() {
                    ids[k] = g;
                    weights[k] = 1.0 / (d * d + 1e-8);
                    total += weights[k];
                }
                for v in weights.iter_mut() {
                    *v /= total.max(1e-12);
                }
            });
        Ok(Self { source, sampled, layout, guides, weights })
    }
}

/// Reserve at the actual host allocation boundary, retaining its concrete cause.
fn reserved<T>(count: usize) -> SimResult<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count)?;
    Ok(values)
}
