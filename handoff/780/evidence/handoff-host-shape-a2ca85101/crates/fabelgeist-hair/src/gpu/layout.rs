//! Checked native cardinality and byte layouts for fixed-point-count strands.

use super::StrandGpu;
use crate::lod::{StrandInfo, StrandLevel};
use crate::parameters::{PointsPerStrand, StrandCount};
use crate::strands::{StrandShape, StrandShapeError, Strands};
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
    shape: StrandShape,
}

#[derive(Debug, thiserror::Error)]
pub enum StrandLayoutError {
    #[error("invalid host strand shape")]
    Shape(#[from] StrandShapeError),
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
    #[error("{strands:?} strands of {points:?} points exceed native sample addresses")]
    SampleCount {
        strands: StrandCount,
        points: PointsPerStrand,
    },
    #[error("{points:?} points exceed native ribbon vertex addresses")]
    DrawVertices { points: PointsPerStrand },

}

impl StrandLayout {
    pub fn new(strands: StrandCount, points: PointsPerStrand) -> StrandLayoutResult<Self> {
        Self::try_from(StrandShape::new(strands, points)?)
    }

    /// Admit a host level's complete shape, then choose its resident prefix.
    /// Full host collections need not fit the native sample address space.
    pub fn for_host_level(
        level: &StrandLevel,
        maximum: BufferByteLength,
    ) -> StrandLayoutResult<Self> {
        let shape = level.layout();
        let strands = shape.strands();
        let points = shape.points();
        if strands.count() == 0 {
            return Self::try_from(shape);
        }
        // Divide before multiplying: the full host stride need not fit u64.
        // Supported GPU hosts have 32-bit or 64-bit collection addresses.
        let count = (strands.count() as u64)
            .min(u64::from(maximum) / size_of::<[f32; 4]>() as u64 / points.count() as u64)
            .min(u64::from(maximum) / size_of::<StrandInfo>() as u64);
        Self::try_from(shape.limited_to(StrandCount::from_count(count as usize)))
    }

    pub fn strands(self) -> StrandCount {
        self.shape.strands()
    }

    pub fn points(self) -> PointsPerStrand {
        self.shape.points()
    }

    /// Native uniform or numerical-kernel population, admitted by `new`.
    pub fn native_strands(self) -> u32 {
        self.shape.strands().count() as u32
    }

    /// Native uniform or numerical-kernel point count, admitted by `new`.
    pub fn native_points(self) -> u32 {
        self.shape.points().count() as u32
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

    /// One native u32 value per strand, for maps and indices.
    pub fn strand_word_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_strands()) * size_of::<u32>() as u64)
    }

    /// Native point storage is one vec4 per sample.
    pub fn positions_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_samples()) * size_of::<[f32; 4]>() as u64)
    }

    pub fn info_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.native_strands()) * size_of::<StrandInfo>() as u64)
    }

    /// Storage binding extent, including one native record for an empty level.
    /// The receiver checks the actual buffer and device binding limit.
    pub fn positions_binding_bytes(self) -> BufferByteLength {
        self.positions_bytes()
            .max(BufferByteLength::from(size_of::<[f32; 4]>()))
    }

    /// Storage binding extent, separate from the empty level's zero data bytes.
    pub fn info_binding_bytes(self) -> BufferByteLength {
        self.info_bytes()
            .max(BufferByteLength::from(size_of::<StrandInfo>()))
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
        Ok(Self { shape: self.shape.prefix(strands)? })
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
        Self { shape: self.shape.limited_to(StrandCount::from_count(count as usize)) }
    }
}

impl TryFrom<StrandShape> for StrandLayout {
    type Error = StrandLayoutError;

    fn try_from(shape: StrandShape) -> StrandLayoutResult<Self> {
        let strands = shape.strands();
        let points = shape.points();
        let count = u32::try_from(strands.count()).map_err(|source| StrandLayoutError::StrandCount {
            requested: strands, source,
        })?;
        let samples = u32::try_from(points.count()).map_err(|source| StrandLayoutError::PointCount {
            requested: points, source,
        })?;
        count.checked_mul(samples).ok_or(StrandLayoutError::SampleCount { strands, points })?;
        if count != 0 {
            samples.checked_mul(2).ok_or(StrandLayoutError::DrawVertices { points })?;
        }
        Ok(Self { shape })
    }
}

impl TryFrom<&Strands> for StrandLayout {
    type Error = StrandLayoutError;

    fn try_from(strands: &Strands) -> StrandLayoutResult<Self> {
        Self::try_from(StrandShape::try_from(strands)?)
    }
}

impl TryFrom<&StrandLevel> for StrandLayout {
    type Error = StrandLayoutError;

    fn try_from(level: &StrandLevel) -> StrandLayoutResult<Self> {
        Self::try_from(level.layout())
    }
}
