//! Checked corrective-array relationships and the native dense tensor adapter.

use burn::tensor::{Device, Tensor, TensorData};
use fabelgeist_numpy_storage::{
    NpyArray, NpyDimension, NpyElementCount, NpyFloat32Values, NpyIntegerValues, NpyLayoutError,
    NpyShape, NpyTensorLayout,
};

use fabelgeist_rig::RigJointCount;

use super::SKIPPED_JOINTS;
use crate::character::MeshVertexCount;
pub use error::{CorrectiveExtent, CorrectiveImportError, Result};

mod error;

const HIDDEN_PER_JOINT: NpyDimension = NpyDimension::new(24);
const FEATURES_PER_JOINT: NpyDimension = NpyDimension::new(6);
const VERTEX_COORDINATES: NpyDimension = NpyDimension::new(3);
const SPARSE_COLUMNS: NpyElementCount = NpyElementCount::new(2);

/// A corrective array and its admitted basis/network shape.
/// Mesh and rig counts are checked; this does not bind a particular mesh identity.
pub(super) struct CorrectiveBasis {
    array: NpyArray,
    layout: CorrectiveLayout,
}

/// The admitted `[components, vertices, 3]` basis shape and network extents.
/// `hidden` is the component axis; `vertices` equals the supplied mesh count;
/// the coordinate axis is fixed by VERTEX_COORDINATES. Consumers use these named
/// extents and never reinterpret the generic source dimensions.
pub(super) struct CorrectiveLayout {
    joints: RigJointCount,
    posed_joints: NpyDimension,
    vertices: MeshVertexCount,
    features: NpyDimension,
    hidden: NpyDimension,
    basis: NpyTensorLayout<2>,
    activation: NpyTensorLayout<2>,
}

/// Sparse contributions admitted against the layout before dense allocation.
/// Offsets cannot be uploaded using a different layout or unchecked coordinates.
pub(super) struct SparseActivation<'layout> {
    layout: &'layout CorrectiveLayout,
    contributions: Vec<SparseContribution>,
}

struct SparseContribution {
    offset: CorrectiveDenseOffset,
    weight: f32,
}

/// A checked native dense-buffer offset, meaningful only with the layout
/// retained by SparseActivation. It never leaves this import adapter.
struct CorrectiveDenseOffset {
    native_index: usize,
}

/// A hidden-unit coordinate after the storage reader's native-width cast.
/// Its role is known; membership in a particular layout is not yet established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrectiveHiddenIndex {
    native_index: usize,
}

/// A feature-channel coordinate after the storage reader's native-width cast.
/// Its role is known; membership in a particular layout is not yet established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorrectiveFeatureIndex {
    native_index: usize,
}

#[derive(Clone, Copy)]
struct SparseCoordinate {
    hidden: CorrectiveHiddenIndex,
    feature: CorrectiveFeatureIndex,
}

impl CorrectiveBasis {
    pub(super) fn new(
        array: NpyArray,
        joints: RigJointCount,
        vertices: MeshVertexCount,
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
        vertices: MeshVertexCount,
    ) -> Result<Self> {
        // Rig membership count becomes an array extent here; this is not a rig ordinal.
        let joint_extent = NpyDimension::from(joints.len());
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
        // Mesh quantity becomes a generic axis only at this array-layout boundary.
        let vertex_extent = NpyDimension::from(vertices.len());
        if basis_vertices != vertex_extent {
            return Err(CorrectiveImportError::VertexCount {
                basis: basis_vertices,
                mesh: vertices,
            });
        }
        let posed_joints = joint_extent
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
        let basis_columns = Self::scaled(
            vertex_extent,
            VERTEX_COORDINATES,
            CorrectiveExtent::BasisColumns,
        )?;
        let basis = NpyTensorLayout::try_from([hidden, basis_columns]).map_err(|source| {
            CorrectiveImportError::TensorLayout {
                extent: CorrectiveExtent::BasisElements,
                source,
            }
        })?;
        let activation = NpyTensorLayout::try_from([features, hidden]).map_err(|source| {
            CorrectiveImportError::TensorLayout {
                extent: CorrectiveExtent::DenseElements,
                source,
            }
        })?;
        Ok(Self {
            joints,
            posed_joints,
            vertices,
            features,
            hidden,
            basis,
            activation,
        })
    }

    pub(super) fn joints(&self) -> RigJointCount {
        self.joints
    }

    pub(super) fn posed_joints(&self) -> NpyDimension {
        self.posed_joints
    }

    pub(super) fn vertices(&self) -> MeshVertexCount {
        self.vertices
    }

    pub(super) fn output_dimensions(
        &self,
        batch: NpyDimension,
    ) -> std::result::Result<NpyTensorLayout<3>, NpyLayoutError> {
        NpyTensorLayout::try_from([
            batch,
            NpyDimension::from(self.vertices.len()),
            VERTEX_COORDINATES,
        ])
    }

    pub(super) fn feature_dimensions(
        &self,
        batch: NpyDimension,
    ) -> std::result::Result<NpyTensorLayout<2>, NpyLayoutError> {
        NpyTensorLayout::try_from([batch, self.features])
    }

    pub(super) fn basis_dimensions(&self) -> NpyTensorLayout<2> {
        self.basis
    }

    fn activation_dimensions(&self) -> NpyTensorLayout<2> {
        self.activation
    }

    /// Converts role coordinates to a native dense offset only after membership
    /// in this layout. The admitted dense product proves the arithmetic fits.
    fn dense_offset(&self, coordinate: SparseCoordinate) -> Result<CorrectiveDenseOffset> {
        let SparseCoordinate { hidden, feature } = coordinate;
        let hidden_width = usize::from(self.hidden);
        if hidden.native_index >= hidden_width || feature.native_index >= usize::from(self.features)
        {
            return Err(CorrectiveImportError::SparseCoordinate { hidden, feature });
        }
        Ok(CorrectiveDenseOffset {
            native_index: feature.native_index * hidden_width + hidden.native_index,
        })
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

impl<'layout> SparseActivation<'layout> {
    pub(super) fn new(
        indices: NpyIntegerValues,
        weights: NpyFloat32Values,
        layout: &'layout CorrectiveLayout,
    ) -> Result<Self> {
        if weights.element_count().checked_mul(SPARSE_COLUMNS) != Some(indices.element_count()) {
            return Err(CorrectiveImportError::SparseCardinality {
                indices: indices.element_count(),
                weights: weights.element_count(),
            });
        }
        let (rows, columns) = indices.values().split_at(weights.values().len());
        let contributions = rows
            .iter()
            .zip(columns)
            .zip(weights.values())
            .map(|((hidden, feature), weight)| {
                let coordinate = SparseCoordinate {
                    hidden: CorrectiveHiddenIndex::from(*hidden),
                    feature: CorrectiveFeatureIndex::from(*feature),
                };
                Ok(SparseContribution {
                    offset: layout.dense_offset(coordinate)?,
                    weight: *weight,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            layout,
            contributions,
        })
    }

    /// Densifies in Burn's transposed `[features, hidden]` order. Only the
    /// checked dense offset, allocation length and coefficient reach native storage.
    pub(super) fn into_tensor(self, device: &Device) -> Tensor<2> {
        // Every offset and the allocation extent were admitted together.
        let mut dense = vec![0.0f32; usize::from(self.layout.activation.element_count())];
        for contribution in self.contributions {
            // Ordered writes intentionally let the last duplicate replace earlier ones.
            dense[contribution.offset.native_index] = contribution.weight;
        }
        Tensor::from_data(
            TensorData::new(dense, self.layout.activation_dimensions()),
            device,
        )
    }
}

impl CorrectiveHiddenIndex {
    /// Native-width coordinate for structured diagnostics, not layout membership.
    pub const fn native_index(self) -> usize {
        self.native_index
    }
}

impl CorrectiveFeatureIndex {
    /// Native-width coordinate for structured diagnostics, not layout membership.
    pub const fn native_index(self) -> usize {
        self.native_index
    }
}

impl From<i64> for CorrectiveHiddenIndex {
    fn from(value: i64) -> Self {
        // Preserve cast-before-bounds, including rejected negative diagnostics.
        Self {
            native_index: value as usize,
        }
    }
}

impl From<i64> for CorrectiveFeatureIndex {
    fn from(value: i64) -> Self {
        Self {
            native_index: value as usize,
        }
    }
}
