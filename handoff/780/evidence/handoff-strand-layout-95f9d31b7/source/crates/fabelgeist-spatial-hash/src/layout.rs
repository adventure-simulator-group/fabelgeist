//! Shared spatial-hash populations, bucket layout and cell scale.

use std::fmt;
use std::num::TryFromIntError;

#[cfg(feature = "gpu")]
use fabelgeist_compute::{SortItemCount, SortKeyWidth};
#[cfg(feature = "gpu")]
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameter};

/// Minimum bucket population of the shared host/native hash layout.
const MIN_BUCKETS: u32 = 64;
/// Two buckets per point keeps typical runs short before exact-cell filtering.
const BUCKETS_PER_POINT: u32 = 2;

/// Entries in a spatial hash, independently of sorting or dispatch shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct HashPointCount(u32);

/// A nonempty power-of-two bucket population admitted by [`HashLayout`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashBucketCount(u32);

/// A point population whose shared bucket table fits native u32 addressing.
#[derive(Clone, Copy, Debug)]
pub struct HashLayout {
    points: HashPointCount,
    buckets: HashBucketCount,
}

/// Reciprocal cell size in the caller's coordinate units.
///
/// This owns the original normalization: nonpositive and NaN sizes become
/// `f32::MIN_POSITIVE`; positive infinity gives a zero reciprocal. It does not
/// prove that any particular point projects to a representable integer cell.
#[derive(Clone, Copy, Debug, Default)]
pub struct CellScale(f32);

/// The requested point population cannot produce the shared bucket layout.
#[derive(Clone, Copy, Debug)]
pub struct HashLayoutError {
    points: HashPointCount,
}

impl From<u32> for HashPointCount {
    fn from(points: u32) -> Self {
        Self(points)
    }
}

impl TryFrom<usize> for HashPointCount {
    type Error = TryFromIntError;

    fn try_from(points: usize) -> Result<Self, Self::Error> {
        u32::try_from(points).map(Self)
    }
}

impl HashPointCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[cfg(feature = "gpu")]
    pub fn word_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * size_of::<u32>() as u64)
    }

    /// The native point buffer is `array<vec4<f32>>`, in caller coordinates.
    #[cfg(feature = "gpu")]
    pub fn point_bytes(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * size_of::<[f32; 4]>() as u64)
    }
}

impl fmt::Display for HashPointCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Default for HashBucketCount {
    fn default() -> Self {
        Self(MIN_BUCKETS)
    }
}

impl HashBucketCount {
    pub(crate) fn mask(self) -> u32 {
        self.0 - 1
    }

    #[cfg(feature = "gpu")]
    pub fn range_bytes(self) -> BufferByteLength {
        // The final range word may be u32::MAX; its byte extent is wider.
        BufferByteLength::from(u64::from(self.0) * size_of::<[u32; 2]>() as u64)
    }

    #[cfg(feature = "gpu")]
    pub fn sort_key_width(self) -> SortKeyWidth {
        SortKeyWidth::from(self.0.trailing_zeros())
    }
}

impl From<HashBucketCount> for usize {
    fn from(buckets: HashBucketCount) -> Self {
        buckets.0 as usize
    }
}

impl TryFrom<HashPointCount> for HashLayout {
    type Error = HashLayoutError;

    fn try_from(points: HashPointCount) -> Result<Self, Self::Error> {
        let buckets = points.0.max(1)
            .checked_mul(BUCKETS_PER_POINT)
            .and_then(u32::checked_next_power_of_two)
            .ok_or(HashLayoutError { points })?;
        Ok(Self { points, buckets: HashBucketCount(buckets.max(MIN_BUCKETS)) })
    }
}

impl HashLayout {
    pub fn point_count(self) -> HashPointCount {
        self.points
    }

    pub fn bucket_count(self) -> HashBucketCount {
        self.buckets
    }

    /// Empty allocations retain one point slot, as in the original GPU grid.
    #[cfg(feature = "gpu")]
    pub(crate) fn with_sentinel(self) -> Self {
        Self { points: HashPointCount(self.points.0.max(1)), ..self }
    }

    /// Prefer half-again growth, without rejecting a representable request
    /// solely because speculative spare capacity would overflow the layout.
    #[cfg(feature = "gpu")]
    pub(crate) fn grown_for(self, requested: Self) -> Self {
        let Some(grown) = self.points.0.checked_add(self.points.0 / 2) else {
            return requested;
        };
        match Self::try_from(HashPointCount(grown.max(requested.points.0))) {
            Ok(layout) => layout,
            Err(_) => requested,
        }
    }
}

impl CellScale {
    /// Admit a cell size in the same coordinate units as the hashed points.
    pub fn from_cell_size(size: f32) -> Self {
        Self(1.0 / size.max(f32::MIN_POSITIVE))
    }

    /// Native CPU/WGSL cell projection coefficient, not a physical length.
    pub fn inverse_cell(self) -> f32 {
        self.0
    }

    pub fn cell_size(self) -> f32 {
        1.0 / self.0
    }
}

impl fmt::Display for HashLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "spatial hash bucket layout cannot address {} points", self.points)
    }
}

impl std::error::Error for HashLayoutError {}

#[cfg(feature = "gpu")]
impl From<HashPointCount> for SortItemCount {
    fn from(points: HashPointCount) -> Self {
        Self::from(points.0)
    }
}

#[cfg(feature = "gpu")]
impl From<HashPointCount> for InvocationCount {
    fn from(points: HashPointCount) -> Self {
        Self::from(points.0)
    }
}

#[cfg(feature = "gpu")]
impl From<HashBucketCount> for InvocationCount {
    fn from(buckets: HashBucketCount) -> Self {
        Self::from(buckets.0)
    }
}

#[cfg(feature = "gpu")]
impl From<HashPointCount> for PassParameter {
    fn from(points: HashPointCount) -> Self {
        Self::Unsigned(points.0)
    }
}

#[cfg(feature = "gpu")]
impl From<HashBucketCount> for PassParameter {
    fn from(buckets: HashBucketCount) -> Self {
        Self::Unsigned(buckets.0)
    }
}

#[cfg(feature = "gpu")]
impl From<CellScale> for PassParameter {
    fn from(scale: CellScale) -> Self {
        Self::from(scale.0)
    }
}
