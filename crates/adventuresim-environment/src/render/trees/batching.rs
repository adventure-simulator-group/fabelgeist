//! Spatial reuse policy and native diagnostic batch occupancy.

use std::collections::BTreeMap;

use bevy::prelude::Resource;
use fabelgeist_math::units::Metres;
use serde::Serialize;

use super::super::chunks::CHUNK;

/// Capacity and size partitions are independent of this spatial cell policy.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
pub enum TreeBatching {
    #[default]
    LocalCells,
    RegionalCells,
}

/// Diagnostic wire counts: instance count maps to nonempty source batch count.
/// Counts describe source streams, before per-instance GPU visibility culling.
#[derive(Clone, Debug, Default, Serialize)]
pub struct TreeBatchOccupancy {
    pub policy: TreeBatching,
    pub histogram: BTreeMap<usize, usize>,
}

impl TreeBatching {
    pub(super) fn cell_width(self) -> Metres {
        match self {
            Self::LocalCells => Metres::from_metres(CHUNK),
            Self::RegionalCells => Metres::from_metres(128.0),
        }
    }
}
