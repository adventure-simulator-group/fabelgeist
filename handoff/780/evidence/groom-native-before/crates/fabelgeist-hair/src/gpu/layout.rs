//! Checked native cardinality and byte layouts for fixed-point-count strands.

use super::StrandGpu;
use crate::lod::{StrandInfo, StrandLevel};
use crate::parameters::{PointsPerStrand, StrandCount};
use crate::strands::Strands;
use fabelgeist_compute::SortItemCount;
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount};
use fabelgeist_spatial_hash::HashPointCount;
use std::num::TryFromIntError;

pub type StrandLayoutResult<T> = Result<T, StrandLayoutError>;

/// Native addresses for one fixed-point-count strand population.
///
/// This checks cardinality, not buffer capacity, device/source identity,
/// geometric values, or a simulation kernel's additional work arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrandLayout {
    strands: StrandCount,
    points: PointsPerStrand,
}

/// A per-strand collection paired with the root UV population.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrandAttribute {
    RootNormal,
    Id,
    Group,
    Radius,
}

#[derive(Debug, thiserror::Error)]
pub enum StrandLayoutError {
    #[error("strand population {requested:?} exceeds native addresses")]
    StrandCount {
        requested: StrandCount,
        #[source]
        source: TryFromIntError,
    },
    #[error("points per strand {requested:?} exceed native addresses")]
    PointCount {
        requested: PointsPerStrand,
        #[source]
        source: TryFromIntError,
    },
    #[error("nonempty strands need a root and tip, received {points:?} points")]
    MissingRootOrTip { points: PointsPerStrand },
    #[error("{strands:?} strands of {points:?} points exceed native sample addresses")]
    SampleCount {
        strands: StrandCount,
        points: PointsPerStrand,
    },
    #[error("{points:?} points exceed native ribbon vertex addresses")]
    DrawVertices { points: PointsPerStrand },
    #[error("{strands:?} strands of {points:?} points exceed host collection addresses")]
    HostSampleCount {
        strands: StrandCount,
        points: PointsPerStrand,
    },
    #[error("{strands:?} strands of {points:?} points have {actual} position records")]
    PositionCount {
        strands: StrandCount,
        points: PointsPerStrand,
        actual: usize,
    },
    #[error("{attribute:?} has {actual} records for {strands:?} strands")]
    AttributeCount {
        attribute: StrandAttribute,
        strands: StrandCount,
        actual: usize,
    },
    #[error("strand prefix {requested:?} exceeds source population {available:?}")]
    Prefix {
        requested: StrandCount,
        available: StrandCount,
    },
}

impl StrandLayout {
    pub fn new(strands: StrandCount, points: PointsPerStrand) -> StrandLayoutResult<Self> {
        let count = u32::try_from(strands.count()).map_err(|source| StrandLayoutError::StrandCount {
            requested: strands,
            source,
        })?;
        let samples = u32::try_from(points.count()).map_err(|source| StrandLayoutError::PointCount {
            requested: points,
            source,
        })?;
        if count != 0 && samples < 2 {
            return Err(StrandLayoutError::MissingRootOrTip { points });
        }
        count
            .checked_mul(samples)
            .ok_or(StrandLayoutError::SampleCount { strands, points })?;
        if count != 0 {
            samples
                .checked_mul(2)
                .ok_or(StrandLayoutError::DrawVertices { points })?;
        }
        Ok(Self { strands, points })
    }

    /// Admit a host level's complete shape, then choose its resident prefix.
    /// Full host collections need not fit the native sample address space.
    pub fn for_host_level(
        level: &StrandLevel,
        maximum: BufferByteLength,
    ) -> StrandLayoutResult<Self> {
        let strands = StrandCount::from_count(level.len());
        let points = PointsPerStrand::from_count(level.points);
        let expected = strands
            .count()
            .checked_mul(points.count())
            .ok_or(StrandLayoutError::HostSampleCount { strands, points })?;
        if level.positions.len() != expected {
            return Err(StrandLayoutError::PositionCount {
                strands,
                points,
                actual: level.positions.len(),
            });
        }
        if strands.count() == 0 {
            return Self::new(strands, points);
        }
        if points.count() < 2 {
            return Err(StrandLayoutError::MissingRootOrTip { points });
        }
        // Divide before multiplying: the full host stride need not fit u64.
        // Supported GPU hosts have 32-bit or 64-bit collection addresses.
        let count = (strands.count() as u64)
            .min(u64::from(maximum) / size_of::<[f32; 4]>() as u64 / points.count() as u64)
            .min(u64::from(maximum) / size_of::<StrandInfo>() as u64);
        Self::new(StrandCount::from_count(count as usize), points)
    }

    pub fn strands(self) -> StrandCount {
        self.strands
    }

    pub fn points(self) -> PointsPerStrand {
        self.points
    }

    /// Native uniform or numerical-kernel population, admitted by `new`.
    pub fn native_strands(self) -> u32 {
        self.strands.count() as u32
    }

    /// Native uniform or numerical-kernel point count, admitted by `new`.
    pub fn native_points(self) -> u32 {
        self.points.count() as u32
    }

    pub fn native_samples(self) -> u32 {
        self.native_strands() * self.native_points()
    }

    /// Ribbon vertices per instance; empty populations draw no vertices.
    pub fn native_draw_vertices(self) -> u32 {
        if self.native_strands() == 0 {
            0
        } else {
            self.native_points() * 2
        }
    }

    pub fn strand_invocations(self) -> InvocationCount {
        InvocationCount::from(self.native_strands())
    }

    pub fn sample_invocations(self) -> InvocationCount {
        InvocationCount::from(self.native_samples())
    }

    pub fn segment_invocations(self) -> InvocationCount {
        let segments = self.native_points().saturating_sub(1);
        InvocationCount::from(self.native_strands() * segments)
    }

    pub fn sort_items(self) -> SortItemCount {
        SortItemCount::from(self.native_strands())
    }

    pub fn hash_points(self) -> HashPointCount {
        HashPointCount::from(self.native_strands())
    }

    /// Native point storage is one vec4 per sample.
    pub fn positions_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_samples()) * size_of::<[f32; 4]>() as u64)
    }

    pub fn info_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_strands()) * size_of::<StrandInfo>() as u64)
    }

    pub fn attributes_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_strands()) * size_of::<StrandGpu>() as u64)
    }

    /// Native roots are one vec4 per strand.
    pub fn roots_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_strands()) * size_of::<[f32; 4]>() as u64)
    }

    /// Count prefix only; the caller retains the actual source buffers.
    pub fn prefix(self, strands: StrandCount) -> StrandLayoutResult<Self> {
        if strands > self.strands {
            return Err(StrandLayoutError::Prefix {
                requested: strands,
                available: self.strands,
            });
        }
        Ok(Self { strands, ..self })
    }

    /// Resident level population fitting both position and info bindings.
    /// This applies the renderer's storage limit, not a geometry rewrite.
    pub fn fit_level_binding(self, maximum: BufferByteLength) -> Self {
        if self.native_strands() == 0 {
            return self;
        }
        // Nonempty admission proves points >= 2 and all divisors positive.
        let position_stride = u64::from(self.native_points()) * size_of::<[f32; 4]>() as u64;
        let count = u64::from(self.native_strands())
            .min(u64::from(maximum) / position_stride)
            .min(u64::from(maximum) / size_of::<StrandInfo>() as u64);
        Self {
            strands: StrandCount::from_count(count as usize),
            ..self
        }
    }

    fn check_positions(self, actual: usize) -> StrandLayoutResult<Self> {
        if actual != self.native_samples() as usize {
            return Err(StrandLayoutError::PositionCount {
                strands: self.strands,
                points: self.points,
                actual,
            });
        }
        Ok(self)
    }
}

impl TryFrom<&Strands> for StrandLayout {
    type Error = StrandLayoutError;

    fn try_from(strands: &Strands) -> StrandLayoutResult<Self> {
        let layout = Self::new(
            StrandCount::from_count(strands.len()),
            PointsPerStrand::from_count(strands.points),
        )?
        .check_positions(strands.positions.len())?;
        for attribute in [
            StrandAttribute::RootNormal,
            StrandAttribute::Id,
            StrandAttribute::Group,
            StrandAttribute::Radius,
        ] {
            let actual = match attribute {
                StrandAttribute::RootNormal => strands.root_normal.len(),
                StrandAttribute::Id => strands.id.len(),
                StrandAttribute::Group => strands.group.len(),
                StrandAttribute::Radius => strands.radius.len(),
            };
            if actual != layout.strands.count() {
                return Err(StrandLayoutError::AttributeCount {
                    attribute,
                    strands: layout.strands,
                    actual,
                });
            }
        }
        Ok(layout)
    }
}

impl TryFrom<&StrandLevel> for StrandLayout {
    type Error = StrandLayoutError;

    fn try_from(level: &StrandLevel) -> StrandLayoutResult<Self> {
        Self::new(
            StrandCount::from_count(level.len()),
            PointsPerStrand::from_count(level.points),
        )?
        .check_positions(level.positions.len())
    }
}
