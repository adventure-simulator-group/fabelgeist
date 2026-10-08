//! Array axes, checked element counts, payload length, and fixed-rank SDK layout.
use super::Dtype;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyDimension(usize);
impl From<usize> for NpyDimension {
    fn from(extent: usize) -> Self {
        Self(extent)
    }
}
impl From<NpyDimension> for usize {
    fn from(dimension: NpyDimension) -> Self {
        dimension.0
    }
}
impl std::fmt::Display for NpyDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyRank(usize);
impl From<usize> for NpyRank {
    fn from(rank: usize) -> Self {
        Self(rank)
    }
}
impl std::fmt::Display for NpyRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyElementCount(usize);
impl From<usize> for NpyElementCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyOccupancy {
    Empty,
    Nonempty,
}
impl NpyElementCount {
    pub fn occupancy(self) -> NpyOccupancy {
        if self.0 == 0 {
            NpyOccupancy::Empty
        } else {
            NpyOccupancy::Nonempty
        }
    }
    pub fn payload_length(self, dtype: Dtype) -> Result<NpyPayloadLength, NpyLayoutError> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyElementWidth {
    Byte,
    Word32,
    Word64,
}
impl NpyElementWidth {
    /// Native stored-word width at the little-endian decoding adapter.
    pub(super) fn native_width(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Word32 => 4,
            Self::Word64 => 8,
        }
    }
}

/// Byte length of the serialized array's element payload, separate from its
/// logical element count. Native slices are admitted only at format decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyPayloadLength(usize);
impl std::fmt::Display for NpyPayloadLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl NpyPayloadLength {
    /// The narrow native serialization adapter, preserving trailing-file policy.
    pub(super) fn admit(self, payload: &[u8]) -> Result<&[u8], NpyLayoutError> {
        payload
            .get(..self.0)
            .ok_or(NpyLayoutError::TruncatedPayload {
                expected: self,
                available: Self(payload.len()),
            })
    }
}

/// Immutable dimensions and their admitted product. A rank-zero scalar contains
/// one element; any zero axis makes an array empty before other products overflow.
#[derive(Clone, PartialEq, Eq)]
pub struct NpyShape {
    dimensions: Vec<NpyDimension>,
    elements: NpyElementCount,
}
impl TryFrom<Vec<NpyDimension>> for NpyShape {
    type Error = NpyLayoutError;
    fn try_from(dimensions: Vec<NpyDimension>) -> Result<Self, NpyLayoutError> {
        let mut elements = 1usize;
        if dimensions.contains(&NpyDimension(0)) {
            elements = 0;
        } else {
            for dimension in &dimensions {
                elements = elements.checked_mul(dimension.0).ok_or_else(|| {
                    NpyLayoutError::ElementCountOverflow {
                        dimensions: dimensions.clone(),
                    }
                })?;
            }
        }
        Ok(Self {
            dimensions,
            elements: NpyElementCount(elements),
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
    pub(super) fn tensor_layout<const D: usize>(
        &self,
    ) -> Result<NpyTensorLayout<D>, NpyLayoutError> {
        let dimensions: [NpyDimension; D] =
            self.dimensions
                .as_slice()
                .try_into()
                .map_err(|source| NpyLayoutError::Rank {
                    expected: NpyRank::from(D),
                    actual: self.rank(),
                    source,
                })?;
        Ok(NpyTensorLayout {
            dimensions: dimensions.map(usize::from),
        })
    }
}
impl std::fmt::Debug for NpyShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.dimensions.iter().copied().map(usize::from))
            .finish()
    }
}
/// The native Burn SDK shape is produced only after nominal rank admission.
pub(super) struct NpyTensorLayout<const D: usize> {
    pub(super) dimensions: [usize; D],
}

#[derive(Debug)]
pub enum NpyLayoutError {
    ElementCountOverflow {
        dimensions: Vec<NpyDimension>,
    },
    PayloadLengthOverflow {
        elements: NpyElementCount,
        width: NpyElementWidth,
    },
    TruncatedPayload {
        expected: NpyPayloadLength,
        available: NpyPayloadLength,
    },
    Rank {
        expected: NpyRank,
        actual: NpyRank,
        source: std::array::TryFromSliceError,
    },
}
impl std::fmt::Display for NpyLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElementCountOverflow { .. } => {
                f.write_str("NumPy element count overflows the storage layout")
            }
            Self::PayloadLengthOverflow { .. } => {
                f.write_str("NumPy payload length overflows the storage layout")
            }
            Self::TruncatedPayload { .. } => f.write_str("truncated .npy payload"),
            Self::Rank {
                expected, actual, ..
            } => write!(f, "array has {actual} dimensions, expected {expected}"),
        }
    }
}
impl std::error::Error for NpyLayoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rank { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
