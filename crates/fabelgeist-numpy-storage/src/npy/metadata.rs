//! Array axes, checked element counts, payload length, and fixed-rank SDK layout.
use derive_more::{Display, Error, From, Into};

use super::Dtype;

pub type Result<T> = std::result::Result<T, NpyLayoutError>;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, From, Into)]
pub struct NpyDimension(usize);

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, From)]
pub struct NpyRank(usize);

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, From, Into)]
pub struct NpyElementCount(usize);

/// Byte length of the serialized array's element payload, separate from its
/// logical element count. Native slices are admitted only at format decoding.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub struct NpyPayloadLength(usize);

/// Immutable dimensions and their admitted product. A rank-zero scalar contains
/// one element; any zero axis makes an array empty before other products overflow.
#[derive(Clone, PartialEq, Eq)]
pub struct NpyShape {
    dimensions: Vec<NpyDimension>,
    elements: NpyElementCount,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyElementWidth {
    Byte,
    Word32,
    Word64,
}

#[derive(Debug, Display, Error)]
pub enum NpyLayoutError {
    #[display("NumPy element count overflows the storage layout")]
    ElementCountOverflow { dimensions: Vec<NpyDimension> },
    #[display("NumPy payload length overflows the storage layout")]
    PayloadLengthOverflow {
        elements: NpyElementCount,
        width: NpyElementWidth,
    },
    #[display("truncated .npy payload")]
    TruncatedPayload {
        expected: NpyPayloadLength,
        available: NpyPayloadLength,
    },
    #[display("array has {actual} dimensions, expected {expected}")]
    Rank {
        expected: NpyRank,
        actual: NpyRank,
        source: std::array::TryFromSliceError,
    },
    #[display("tensor layout requires {expected} values, received {actual}")]
    ValueCount {
        expected: NpyElementCount,
        actual: NpyElementCount,
    },
}
impl NpyDimension {
    /// Admits a native array extent, including zero, at a decoder or literal.
    pub const fn new(dimension: usize) -> Self {
        Self(dimension)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }
}

impl NpyElementCount {
    /// Admits the literal element product once, with zero axes taking priority.
    pub(super) fn from_dimensions(dimensions: &[NpyDimension]) -> Result<Self> {
        let mut elements = 1usize;
        if dimensions.contains(&NpyDimension(0)) {
            elements = 0;
        } else {
            for dimension in dimensions {
                elements = elements.checked_mul(dimension.0).ok_or_else(|| {
                    NpyLayoutError::ElementCountOverflow {
                        dimensions: dimensions.to_vec(),
                    }
                })?;
            }
        }
        Ok(Self(elements))
    }

    /// Admits a native collection cardinality at its source boundary.
    pub const fn new(elements: usize) -> Self {
        Self(elements)
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    pub fn payload_length(self, dtype: Dtype) -> Result<NpyPayloadLength> {
        let width = dtype.storage_width();
        self.0
            .checked_mul(width.native_width())
            .map(NpyPayloadLength)
            .ok_or(NpyLayoutError::PayloadLengthOverflow {
                elements: self,
                width,
            })
    }
}
impl NpyElementWidth {
    /// Native byte width used by checked serialized-payload arithmetic.
    pub(super) fn native_width(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Word32 => 4,
            Self::Word64 => 8,
        }
    }
}
impl NpyPayloadLength {
    /// The narrow native serialization adapter, preserving trailing-file policy.
    pub(super) fn admit(self, payload: &[u8]) -> Result<&[u8]> {
        payload
            .get(..self.0)
            .ok_or(NpyLayoutError::TruncatedPayload {
                expected: self,
                available: Self(payload.len()),
            })
    }
}
impl TryFrom<Vec<NpyDimension>> for NpyShape {
    type Error = NpyLayoutError;
    fn try_from(dimensions: Vec<NpyDimension>) -> std::result::Result<Self, NpyLayoutError> {
        let elements = NpyElementCount::from_dimensions(&dimensions)?;
        Ok(Self {
            dimensions,
            elements,
        })
    }
}
impl NpyShape {
    pub fn dimensions(&self) -> &[NpyDimension] {
        &self.dimensions
    }
    pub fn rank(&self) -> NpyRank {
        NpyRank::from(self.dimensions.len())
    }
    pub fn element_count(&self) -> NpyElementCount {
        self.elements
    }
}
impl std::fmt::Debug for NpyShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.dimensions.iter().copied().map(usize::from))
            .finish()
    }
}
#[cfg(test)]
mod tests;
