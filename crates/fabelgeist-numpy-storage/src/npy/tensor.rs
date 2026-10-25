//! Fixed-rank dimensions and decoded values at the Burn boundary.

use burn::tensor::{Device, ReshapeArgs, Shape, Tensor, TensorData};
use derive_more::From;

use super::metadata::Result;
use super::{NpyDimension, NpyElementCount, NpyFloat32Values, NpyLayoutError, NpyShape};

/// A fixed number of logical dimensions. Element-product and value-cardinality
/// admission occurs before upload; reshape retains Burn's own shape checks.
#[derive(Clone, Copy, Debug, From)]
pub struct NpyTensorLayout<const D: usize> {
    dimensions: [NpyDimension; D],
}

impl<const D: usize> NpyTensorLayout<D> {
    pub fn element_count(self) -> Result<NpyElementCount> {
        Ok(NpyShape::try_from(self.dimensions.to_vec())?.element_count())
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
        let expected = layout.element_count()?;
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
