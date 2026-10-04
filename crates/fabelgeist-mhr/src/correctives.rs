//! The non-linear pose-corrective network.
//!
//! Joint rotations are turned into a 6D rotation feature per joint, pushed
//! through a sparse 750 -> 3000 layer, a ReLU, and a dense 3000 -> `3 * V`
//! layer, giving a per-vertex offset applied before skinning.

mod error;
mod layout;
mod sparse;
#[cfg(test)]
mod tests;
use crate::MeshVertexCount;
use burn::tensor::{Device, Tensor, TensorData, activation};
pub use error::CorrectiveDecodeError;
use fabelgeist_fs::FileContents;
use fabelgeist_numpy_storage::{
    ArchiveMemberPresence, NpyArray, Npz, NpzArrayError, NpzArrayName, ZipReadError,
};
pub use layout::{
    CorrectiveBasisDimensions, CorrectiveDimension, CorrectiveNetworkDimensions,
    CorrectiveRigTopology, RejectedCorrectiveShape,
};
use sparse::AdmittedSparseActivation;
pub use sparse::{SparseActivationCoordinate, SparseActivationLayout, SparseCoordinateAxis};

/// The first two joints do not define a local pose, so they carry no feature.
pub const SKIPPED_JOINTS: usize = 2;
/// Hidden units per posed joint.
const HIDDEN_PER_JOINT: usize = 24;
/// Feature channels per posed joint (a 6D rotation).
const FEATURES_PER_JOINT: usize = 6;

/// The archive's role remains distinct from each array's role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectiveArchiveRole {
    Activation,
    Basis,
}
impl CorrectiveArchiveRole {
    fn admit(self, contents: FileContents) -> Result<Npz, CorrectiveDecodeError> {
        Npz::from_bytes(contents).map_err(|source: ZipReadError| -> CorrectiveDecodeError {
            CorrectiveDecodeError::Archive {
                role: self,
                source: Box::new(source),
            }
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectiveArrayRole {
    Basis,
    SparseIndices,
    SparseWeights,
}
impl CorrectiveArrayRole {
    fn name(self) -> NpzArrayName {
        NpzArrayName::from(match self {
            Self::Basis => "corrective_blendshapes",
            Self::SparseIndices => "0.sparse_indices",
            Self::SparseWeights => "0.sparse_weight",
        })
    }
    fn read(self, archive: &Npz) -> Result<NpyArray, CorrectiveDecodeError> {
        archive
            .array(&self.name())
            .map_err(|source: NpzArrayError| -> CorrectiveDecodeError {
                CorrectiveDecodeError::Array {
                    role: self,
                    source: Box::new(source),
                }
            })
    }
}

pub struct PoseCorrectives {
    /// Sparse activation layer, densified and transposed: `[posed * 6, hidden]`.
    activation: Tensor<2>,
    /// Corrective basis: `[hidden, vertices * 3]`.
    basis: Tensor<2>,
    dimensions: CorrectiveNetworkDimensions,
}

impl PoseCorrectives {
    /// Loads corrective archives already held in memory (for web/streamed assets).
    pub fn from_bytes(
        activation: FileContents,
        basis: FileContents,
        rig: CorrectiveRigTopology,
        device: &Device,
    ) -> Result<Option<Self>, CorrectiveDecodeError> {
        Self::from_archives(
            CorrectiveArchiveRole::Activation.admit(activation)?,
            CorrectiveArchiveRole::Basis.admit(basis)?,
            rig,
            device,
        )
    }

    pub(crate) fn from_archives(
        activation: Npz,
        archive: Npz,
        rig: CorrectiveRigTopology,
        device: &Device,
    ) -> Result<Option<Self>, CorrectiveDecodeError> {
        if archive.contains(&CorrectiveArrayRole::Basis.name()) == ArchiveMemberPresence::Absent {
            return Ok(None);
        }
        let basis = CorrectiveArrayRole::Basis.read(&archive)?;
        let basis_dimensions = CorrectiveBasisDimensions::try_from(&basis)?;
        basis_dimensions.admit_vertices(rig)?;
        let dimensions = CorrectiveNetworkDimensions::try_from(rig)?;
        basis_dimensions.admit_components(dimensions)?;
        let indices = CorrectiveArrayRole::SparseIndices.read(&activation)?;
        let weights = CorrectiveArrayRole::SparseWeights.read(&activation)?;
        let dense = AdmittedSparseActivation::from_arrays(&indices, &weights, dimensions)?;
        Ok(Some(Self {
            activation: Tensor::from_data(TensorData::from(dense), device),
            basis: Tensor::from_data(
                TensorData::new(
                    Vec::<f32>::from(basis.floating_values()),
                    [basis_dimensions.components, dimensions.output_coordinates],
                ),
                device,
            ),
            dimensions,
        }))
    }

    pub fn num_vertices(&self) -> MeshVertexCount {
        MeshVertexCount::from(self.dimensions.vertices)
    }

    /// Per-vertex corrective offsets `[batch, vertices, 3]`, from joint
    /// parameters shaped `[batch, joints, 7]`.
    pub fn forward(&self, joint_parameters: Tensor<3>) -> Tensor<3> {
        let batch = joint_parameters.dims()[0];
        let features = self.pose_features(joint_parameters);
        let hidden = activation::relu(features.matmul(self.activation.clone()));
        hidden
            .matmul(self.basis.clone())
            .reshape([batch, self.dimensions.vertices, 3])
    }

    /// The 6D rotation feature per posed joint, flattened to `[batch, posed * 6]`.
    ///
    /// A 6D feature is the first two columns of the joint's rotation matrix,
    /// with 1 subtracted from the two diagonal entries so a rest pose is zero.
    fn pose_features(&self, joint_parameters: Tensor<3>) -> Tensor<2> {
        let batch = joint_parameters.dims()[0];
        let euler = joint_parameters
            .narrow(1, SKIPPED_JOINTS, self.dimensions.posed_joints)
            .narrow(2, 3, 3);

        let (sx, cx) = axis_sin_cos(&euler, RotationFeatureAxis::X);
        let (sy, cy) = axis_sin_cos(&euler, RotationFeatureAxis::Y);
        let (sz, cz) = axis_sin_cos(&euler, RotationFeatureAxis::Z);

        let feature = Tensor::cat(
            vec![
                (cy.clone() * cz.clone()).sub_scalar(1.0),
                cy.clone() * sz.clone(),
                sy.clone().neg(),
                cx.clone().neg() * sz.clone() + sx.clone() * sy.clone() * cz.clone(),
                (cx * cz + sx.clone() * sy * sz).sub_scalar(1.0),
                sx * cy,
            ],
            2,
        );
        feature.reshape([batch, self.dimensions.posed_joints * FEATURES_PER_JOINT])
    }
}

enum RotationFeatureAxis {
    X,
    Y,
    Z,
}
fn axis_sin_cos(euler: &Tensor<3>, axis: RotationFeatureAxis) -> (Tensor<3>, Tensor<3>) {
    let index = match axis {
        RotationFeatureAxis::X => 0,
        RotationFeatureAxis::Y => 1,
        RotationFeatureAxis::Z => 2,
    };
    let angle = euler.clone().narrow(2, index, 1);
    (angle.clone().sin(), angle.cos())
}
