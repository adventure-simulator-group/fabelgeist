//! Checked corrective-array relationships and the native dense tensor adapter.

use burn::tensor::{Device, Tensor, TensorData};
use fabelgeist_numpy_storage::{
    NpyDimension, NpyElementCount, NpyFloatValues, NpyIntegerValues, NpyShape,
};

use super::SKIPPED_JOINTS;
pub use error::{CorrectiveExtent, CorrectiveImportError, Result};

mod error;

const HIDDEN_PER_JOINT: usize = 24;
const FEATURES_PER_JOINT: usize = 6;
const VERTEX_COORDINATES: usize = 3;

/// Array extents admitted together before allocating either corrective tensor.
pub(super) struct CorrectiveLayout {
    posed_joints: NpyDimension,
    vertices: NpyDimension,
    features: NpyDimension,
    hidden: NpyDimension,
    basis_columns: NpyDimension,
    dense_elements: NpyElementCount,
}

/// The two contiguous index columns and their corresponding ordered weights.
pub(super) struct SparseActivation {
    indices: NpyIntegerValues,
    weights: NpyFloatValues,
}

impl CorrectiveLayout {
    pub(super) fn new(
        basis: &NpyShape,
        joints: NpyDimension,
        vertices: NpyDimension,
    ) -> Result<Self> {
        let &[components, basis_vertices, coordinates] = basis.dimensions() else {
            return Err(CorrectiveImportError::BasisShape {
                shape: basis.clone(),
            });
        };
        if coordinates != NpyDimension::from(VERTEX_COORDINATES) {
            return Err(CorrectiveImportError::BasisShape {
                shape: basis.clone(),
            });
        }
        if basis_vertices != vertices {
            return Err(CorrectiveImportError::VertexCount {
                basis: basis_vertices,
                mesh: vertices,
            });
        }
        // These are exact tensor-extent calculations. Re-admit each result
        // after checked native arithmetic; no rig ordinal or geometry is made.
        let posed_joints = usize::from(joints)
            .checked_sub(SKIPPED_JOINTS)
            .map(NpyDimension::from)
            .ok_or(CorrectiveImportError::JointCount { joints })?;
        let hidden = Self::scaled(
            posed_joints,
            HIDDEN_PER_JOINT,
            CorrectiveExtent::HiddenUnits,
        )?;
        if components != hidden {
            return Err(CorrectiveImportError::Components {
                actual: components,
                expected: hidden,
            });
        }
        let features = Self::scaled(
            posed_joints,
            FEATURES_PER_JOINT,
            CorrectiveExtent::FeatureChannels,
        )?;
        let basis_columns =
            Self::scaled(vertices, VERTEX_COORDINATES, CorrectiveExtent::BasisColumns)?;
        let dense_elements = usize::from(features)
            .checked_mul(usize::from(hidden))
            .map(NpyElementCount::from)
            .ok_or(CorrectiveImportError::ExtentOverflow {
                extent: CorrectiveExtent::DenseElements,
            })?;
        Ok(Self {
            posed_joints,
            vertices,
            features,
            hidden,
            basis_columns,
            dense_elements,
        })
    }

    pub(super) fn posed_joints(&self) -> NpyDimension {
        self.posed_joints
    }

    pub(super) fn vertices(&self) -> NpyDimension {
        self.vertices
    }

    pub(super) fn features(&self) -> NpyDimension {
        self.features
    }

    /// Burn requires native dimensions for this flattened basis tensor.
    pub(super) fn basis_dimensions(&self) -> [usize; 2] {
        [usize::from(self.hidden), usize::from(self.basis_columns)]
    }

    fn scaled(
        dimension: NpyDimension,
        factor: usize,
        extent: CorrectiveExtent,
    ) -> Result<NpyDimension> {
        usize::from(dimension)
            .checked_mul(factor)
            .map(NpyDimension::from)
            .ok_or(CorrectiveImportError::ExtentOverflow { extent })
    }
}

impl SparseActivation {
    pub(super) fn new(indices: NpyIntegerValues, weights: NpyFloatValues) -> Result<Self> {
        if weights.values().len().checked_mul(2) != Some(indices.values().len()) {
            return Err(CorrectiveImportError::SparseCardinality {
                indices: NpyElementCount::from(indices.values().len()),
                weights: NpyElementCount::from(weights.values().len()),
            });
        }
        Ok(Self { indices, weights })
    }

    /// Densifies in Burn's transposed `[features, hidden]` order. Native indices,
    /// lengths and coefficients live in this allocation/indexing adapter only.
    pub(super) fn into_tensor(
        self,
        layout: &CorrectiveLayout,
        device: &Device,
    ) -> Result<Tensor<2>> {
        let inputs = usize::from(layout.features);
        let hidden = usize::from(layout.hidden);
        // Cardinality comes from the admitted shape, not a second multiplication.
        let mut dense = vec![0.0f32; usize::from(layout.dense_elements)];
        for (slot, value) in self.weights.values().iter().enumerate() {
            // Preserve the storage reader's native cast before its bounds check,
            // including the rejected coordinate's platform-width diagnostic.
            let row = i64::from(self.indices.values()[slot]) as usize;
            let column =
                i64::from(self.indices.values()[self.weights.values().len() + slot]) as usize;
            if row >= hidden || column >= inputs {
                return Err(CorrectiveImportError::SparseCoordinate {
                    native_row: row,
                    native_column: column,
                });
            }
            // Admission proves column * hidden + row is inside dense. Ordered
            // writes intentionally let the last duplicate replace earlier ones.
            dense[column * hidden + row] = f32::from(*value);
        }
        Ok(Tensor::from_data(
            TensorData::new(dense, [inputs, hidden]),
            device,
        ))
    }
}
