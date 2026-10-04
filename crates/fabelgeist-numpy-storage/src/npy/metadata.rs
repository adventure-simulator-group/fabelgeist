//! Array axes, element counts, fixed-rank SDK admission, and storage layout.
use super::{Dtype, NpyDecodeError, NpyRank};
use fabelgeist_storage::StorageByteLength;

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
    pub fn repeated(self, extent: NpyDimension) -> Option<Self> {
        self.0.checked_mul(extent.0).map(Self)
    }
    pub fn occupancy(self) -> NpyOccupancy {
        if self.0 == 0 {
            NpyOccupancy::Empty
        } else {
            NpyOccupancy::Nonempty
        }
    }
    pub fn payload_length(self, dtype: Dtype) -> Result<StorageByteLength, NpyDecodeError> {
        let width = match dtype.storage_width() {
            NpyElementWidth::Byte => 1,
            NpyElementWidth::Word32 => 4,
            NpyElementWidth::Word64 => 8,
        };
        Ok(StorageByteLength::from(
            self.0
                .checked_mul(width)
                .ok_or(NpyDecodeError::PayloadLengthOverflow)?,
        ))
    }
}
impl std::fmt::Display for NpyElementCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpyElementOrdinal(usize);
impl From<usize> for NpyElementOrdinal {
    fn from(ordinal: usize) -> Self {
        Self(ordinal)
    }
}
impl From<NpyElementOrdinal> for usize {
    fn from(ordinal: NpyElementOrdinal) -> Self {
        ordinal.0
    }
}
impl NpyElementOrdinal {
    pub fn advance(self, elements: NpyElementCount) -> Option<Self> {
        self.0.checked_add(elements.0).map(Self)
    }
}
impl std::fmt::Display for NpyElementOrdinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpyElementWidth {
    Byte,
    Word32,
    Word64,
}
#[derive(Clone, PartialEq, Eq)]
pub struct NpyShape {
    dimensions: Vec<NpyDimension>,
    elements: NpyElementCount,
}
impl TryFrom<Vec<NpyDimension>> for NpyShape {
    type Error = NpyDecodeError;
    fn try_from(dimensions: Vec<NpyDimension>) -> Result<Self, NpyDecodeError> {
        // A zero axis makes the whole array empty before any product can overflow.
        let mut elements = 1usize;
        if dimensions.contains(&NpyDimension(0)) {
            elements = 0;
        } else {
            for dimension in &dimensions {
                elements = elements
                    .checked_mul(dimension.0)
                    .ok_or(NpyDecodeError::ElementCountOverflow)?;
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
    ) -> Result<NpyTensorLayout<D>, NpyDecodeError> {
        let dimensions: [NpyDimension; D] = self.dimensions.as_slice().try_into().map_err(
            |source: std::array::TryFromSliceError| -> NpyDecodeError {
                NpyDecodeError::Rank {
                    expected: NpyRank::from(D),
                    actual: self.rank(),
                    source,
                }
            },
        )?;
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
#[derive(Debug)]
pub(super) struct NpyTensorLayout<const D: usize> {
    pub(super) dimensions: [usize; D],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_and_zero_axes_have_distinct_rank_and_element_counts() {
        let scalar = NpyShape::try_from(Vec::new()).unwrap();
        assert_eq!(scalar.rank(), NpyRank::from(0));
        assert_eq!(scalar.element_count(), NpyElementCount::from(1));
        assert_eq!(scalar.element_count().occupancy(), NpyOccupancy::Nonempty);
        assert_eq!(scalar.tensor_layout::<0>().unwrap().dimensions, [0usize; 0]);

        let dimensions = vec![
            NpyDimension::from(usize::MAX),
            NpyDimension::from(usize::MAX),
            NpyDimension::from(0),
        ];
        let empty = NpyShape::try_from(dimensions.clone()).unwrap();
        assert_eq!(empty.dimensions(), dimensions);
        assert_eq!(empty.rank(), NpyRank::from(3));
        assert_eq!(empty.element_count(), NpyElementCount::from(0));
        assert_eq!(empty.element_count().occupancy(), NpyOccupancy::Empty);
        assert_eq!(
            empty.element_count().payload_length(Dtype::F64).unwrap(),
            StorageByteLength::from(0usize)
        );
    }

    #[test]
    fn shape_and_payload_products_fail_at_their_distinct_boundaries() {
        assert!(matches!(
            NpyShape::try_from(vec![NpyDimension::from(usize::MAX), NpyDimension::from(2)]),
            Err(NpyDecodeError::ElementCountOverflow)
        ));
        let wide = NpyShape::try_from(vec![NpyDimension::from(usize::MAX)]).unwrap();
        assert!(matches!(
            wide.element_count().payload_length(Dtype::F64),
            Err(NpyDecodeError::PayloadLengthOverflow)
        ));
        assert_eq!(
            wide.element_count().payload_length(Dtype::U8).unwrap(),
            StorageByteLength::from(usize::MAX)
        );
        assert_eq!(format!("{wide:?}"), format!("[{}]", usize::MAX));
    }
}
