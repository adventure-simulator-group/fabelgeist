//! Checked corrective-array relationships and the native dense tensor adapter.

use burn::tensor::{Device, Tensor, TensorData};
use fabelgeist_numpy_storage::{
    NpyArray, NpyDimension, NpyElementCount, NpyFloat32Values, NpyIntegerValues, NpyLayoutError,
    NpyShape, NpyTensorLayout,
};

use fabelgeist_rig::RigJointCount;

use super::SKIPPED_JOINTS;
pub use error::{CorrectiveExtent, CorrectiveImportError, Result};

mod error;

const HIDDEN_PER_JOINT: NpyDimension = NpyDimension::new(24);
const FEATURES_PER_JOINT: NpyDimension = NpyDimension::new(6);
const VERTEX_COORDINATES: NpyDimension = NpyDimension::new(3);
const SPARSE_COLUMNS: NpyElementCount = NpyElementCount::new(2);

/// A corrective array and the layout admitted against its owning mesh and rig.
pub(super) struct CorrectiveBasis {
    array: NpyArray,
    layout: CorrectiveLayout,
}

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
    weights: NpyFloat32Values,
}

impl CorrectiveBasis {
    pub(super) fn new(
        array: NpyArray,
        joints: RigJointCount,
        vertices: NpyDimension,
    ) -> Result<Self> {
        let layout = CorrectiveLayout::new(array.shape(), joints, vertices)?;
        Ok(Self { array, layout })
    }

    pub(super) fn layout(&self) -> &CorrectiveLayout {
        &self.layout
    }

    pub(super) fn into_layout(self) -> CorrectiveLayout {
        self.layout
    }

    pub(super) fn to_tensor(
        &self,
        device: &Device,
    ) -> std::result::Result<Tensor<2>, NpyLayoutError> {
        NpyFloat32Values::from(&self.array).into_tensor(self.layout.basis_dimensions(), device)
    }
}

impl CorrectiveLayout {
    pub(super) fn new(
        basis: &NpyShape,
        joints: RigJointCount,
        vertices: NpyDimension,
    ) -> Result<Self> {
        // Rig membership count becomes an array extent here; this is not a rig ordinal.
        let joints = NpyDimension::from(joints.len());
        let &[components, basis_vertices, coordinates] = basis.dimensions() else {
            return Err(CorrectiveImportError::BasisShape {
                shape: basis.clone(),
            });
        };
        if coordinates != VERTEX_COORDINATES {
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
        let posed_joints = joints
            .checked_sub(SKIPPED_JOINTS)
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
        let dense_elements =
            features
                .element_product(hidden)
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

    pub(super) fn output_dimensions(&self, batch: NpyDimension) -> NpyTensorLayout<3> {
        NpyTensorLayout::from([batch, self.vertices, VERTEX_COORDINATES])
    }

    pub(super) fn feature_dimensions(&self, batch: NpyDimension) -> NpyTensorLayout<2> {
        NpyTensorLayout::from([batch, self.features])
    }

    pub(super) fn basis_dimensions(&self) -> NpyTensorLayout<2> {
        NpyTensorLayout::from([self.hidden, self.basis_columns])
    }

    fn activation_dimensions(&self) -> NpyTensorLayout<2> {
        NpyTensorLayout::from([self.features, self.hidden])
    }

    fn scaled(
        dimension: NpyDimension,
        factor: NpyDimension,
        extent: CorrectiveExtent,
    ) -> Result<NpyDimension> {
        dimension
            .checked_mul(factor)
            .ok_or(CorrectiveImportError::ExtentOverflow { extent })
    }
}

impl SparseActivation {
    pub(super) fn new(indices: NpyIntegerValues, weights: NpyFloat32Values) -> Result<Self> {
        if weights.element_count().checked_mul(SPARSE_COLUMNS) != Some(indices.element_count()) {
            return Err(CorrectiveImportError::SparseCardinality {
                indices: indices.element_count(),
                weights: weights.element_count(),
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
        let (rows, columns) = self.indices.values().split_at(self.weights.values().len());
        for ((row, column), value) in rows.iter().zip(columns).zip(self.weights.values()) {
            // Preserve the storage reader's native cast before its bounds check,
            // including the rejected coordinate's platform-width diagnostic.
            let row = i64::from(*row) as usize;
            let column = i64::from(*column) as usize;
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
            TensorData::new(dense, layout.activation_dimensions()),
            device,
        ))
    }
}
