//! Sparse coordinate admission and an owned transposed activation matrix.
use super::{CorrectiveDecodeError, CorrectiveNetworkDimensions};
use burn::tensor::TensorData;
use fabelgeist_numpy_storage::{
    NpyArray, NpyDimension, NpyElementCount, NpyElementOrdinal, NpyFloatValue, NpyIntegerValue,
    NpyIntegerValues,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SparseActivationLayout {
    indices: NpyElementCount,
    weights: NpyElementCount,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SparseActivationCoordinate {
    slot: NpyElementOrdinal,
    row: NpyIntegerValue,
    column: NpyIntegerValue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseCoordinateAxis {
    HiddenRow,
    FeatureColumn,
}
struct AdmittedActivationCoordinate {
    row: usize,
    column: usize,
}
impl SparseActivationCoordinate {
    fn from_columns(
        indices: &NpyIntegerValues,
        slot: NpyElementOrdinal,
        layout: SparseActivationLayout,
    ) -> Self {
        Self {
            slot,
            row: indices.value(slot).expect("checked sparse row block"),
            column: indices
                .value(
                    slot.advance(layout.weights)
                        .expect("checked sparse block offset"),
                )
                .expect("checked sparse column block"),
        }
    }
    fn admit(
        self,
        network: CorrectiveNetworkDimensions,
    ) -> Result<AdmittedActivationCoordinate, CorrectiveDecodeError> {
        let row = usize::try_from(self.row).map_err(
            |source: std::num::TryFromIntError| -> CorrectiveDecodeError {
                CorrectiveDecodeError::SparseCoordinateEncoding {
                    coordinate: self,
                    axis: SparseCoordinateAxis::HiddenRow,
                    source,
                }
            },
        )?;
        let column = usize::try_from(self.column).map_err(
            |source: std::num::TryFromIntError| -> CorrectiveDecodeError {
                CorrectiveDecodeError::SparseCoordinateEncoding {
                    coordinate: self,
                    axis: SparseCoordinateAxis::FeatureColumn,
                    source,
                }
            },
        )?;
        if row >= network.hidden || column >= network.inputs {
            return Err(CorrectiveDecodeError::SparseCoordinate {
                coordinate: self,
                network,
            });
        }
        Ok(AdmittedActivationCoordinate { row, column })
    }
}
#[derive(Clone, Copy, Default)]
struct ActivationCoefficient(f32);
impl From<NpyFloatValue> for ActivationCoefficient {
    fn from(value: NpyFloatValue) -> Self {
        Self(f32::from(value))
    }
}
impl From<ActivationCoefficient> for f32 {
    fn from(value: ActivationCoefficient) -> Self {
        value.0
    }
}
pub(super) struct AdmittedSparseActivation {
    values: Vec<ActivationCoefficient>,
    network: CorrectiveNetworkDimensions,
}
impl AdmittedSparseActivation {
    pub(super) fn from_arrays(
        indices: &NpyArray,
        weights: &NpyArray,
        network: CorrectiveNetworkDimensions,
    ) -> Result<Self, CorrectiveDecodeError> {
        let indices = indices.integer_values();
        let weights = weights.floating_values();
        let layout = SparseActivationLayout {
            indices: indices.element_count(),
            weights: weights.element_count(),
        };
        if layout.weights.repeated(NpyDimension::from(2)) != Some(layout.indices) {
            return Err(CorrectiveDecodeError::SparseLayout(layout));
        }
        let mut activation = Self {
            values: vec![ActivationCoefficient::default(); network.dense_entries],
            network,
        };
        for (slot, weight) in weights.values().iter().copied().enumerate() {
            let coordinate = SparseActivationCoordinate::from_columns(
                &indices,
                NpyElementOrdinal::from(slot),
                layout,
            )
            .admit(network)?;
            activation.assign(coordinate, ActivationCoefficient::from(weight));
        }
        Ok(activation)
    }
    fn assign(&mut self, coordinate: AdmittedActivationCoordinate, weight: ActivationCoefficient) {
        // Keep archive order and assignment: the last duplicate wins.
        self.values[coordinate.column * self.network.hidden + coordinate.row] = weight;
    }
}
impl From<AdmittedSparseActivation> for TensorData {
    fn from(activation: AdmittedSparseActivation) -> Self {
        Self::new(
            activation
                .values
                .into_iter()
                .map(f32::from)
                .collect::<Vec<_>>(),
            [activation.network.inputs, activation.network.hidden],
        )
    }
}
impl std::fmt::Display for SparseActivationLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} indices for {} weights", self.indices, self.weights)
    }
}
impl std::fmt::Display for SparseActivationCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "slot {} at ({}, {})", self.slot, self.row, self.column)
    }
}
