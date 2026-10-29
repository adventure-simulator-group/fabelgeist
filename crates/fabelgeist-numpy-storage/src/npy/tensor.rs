//! Fixed-rank dimensions and decoded values at the Burn boundary.

use burn::tensor::{Device, ReshapeArgs, Shape, Tensor, TensorData};

use super::metadata::Result;
use super::{NpyDimension, NpyElementCount, NpyFloat32Values, NpyLayoutError, NpyRank, NpyShape};

/// Fixed-rank logical dimensions and their admitted literal element product.
///
/// This does not certify a match with a value collection or source tensor.
/// Burn still interprets zero dimensions and validates source cardinality when
/// this layout is used as a reshape argument.
#[derive(Clone, Copy, Debug)]
pub struct NpyTensorLayout<const D: usize> {
    dimensions: [NpyDimension; D],
    elements: NpyElementCount,
}

impl<const D: usize> NpyTensorLayout<D> {
    pub fn element_count(self) -> NpyElementCount {
        self.elements
    }
}

impl<const D: usize> TryFrom<[NpyDimension; D]> for NpyTensorLayout<D> {
    type Error = NpyLayoutError;

    fn try_from(dimensions: [NpyDimension; D]) -> Result<Self> {
        let elements = NpyElementCount::from_dimensions(&dimensions)?;
        Ok(Self {
            dimensions,
            elements,
        })
    }
}

impl<const D: usize> TryFrom<&NpyShape> for NpyTensorLayout<D> {
    type Error = NpyLayoutError;

    fn try_from(shape: &NpyShape) -> Result<Self> {
        let dimensions = shape
            .dimensions()
            .try_into()
            .map_err(|source| NpyLayoutError::Rank {
                expected: NpyRank::from(D),
                actual: shape.rank(),
                source,
            })?;
        Ok(Self {
            dimensions,
            elements: shape.element_count(),
        })
    }
}

impl<const D: usize> From<NpyTensorLayout<D>> for Shape {
    fn from(layout: NpyTensorLayout<D>) -> Self {
        Self::new(layout.dimensions.map(usize::from))
    }
}

impl<const D2: usize> ReshapeArgs<D2> for NpyTensorLayout<D2> {
    fn into_shape<const D: usize>(self, source: Shape) -> Shape {
        // Delegate to Burn so inferred/zero dimensions and mismatch failures
        // retain the framework's numerical reshape semantics.
        <Shape as ReshapeArgs<D2>>::into_shape::<D>(Shape::from(self), source)
    }
}

impl NpyFloat32Values {
    /// Uploads decoded float32 projections only after the complete fixed-rank
    /// layout agrees with their cardinality. Burn owns the final native vector.
    pub fn into_tensor<const D: usize>(
        self,
        layout: NpyTensorLayout<D>,
        device: &Device,
    ) -> Result<Tensor<D>> {
        let expected = layout.element_count();
        let actual = self.element_count();
        if expected != actual {
            return Err(NpyLayoutError::ValueCount { expected, actual });
        }
        Ok(Tensor::from_data(
            TensorData::new(Vec::<f32>::from(self), layout),
            device,
        ))
    }
}
