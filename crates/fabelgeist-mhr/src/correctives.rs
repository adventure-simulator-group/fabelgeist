//! The non-linear pose-corrective network.
//!
//! Joint rotations are turned into a 6D rotation feature per joint, pushed
//! through a sparse 750 -> 3000 layer, a ReLU, and a dense 3000 -> `3 * V`
//! layer, giving a per-vertex offset applied before skinning.

use std::path::Path;

use anyhow::Result;
use burn::tensor::{Device, Tensor, activation};
use fabelgeist_numpy_storage::{NpyDimension, NpyLayoutError};
use fabelgeist_rig::RigJointCount;

use crate::character::MeshVertexCount;

pub use archive::CorrectiveArchiveBytes;
use archive::CorrectiveArchives;
use import::{CorrectiveBasis, CorrectiveLayout};
pub use import::{
    CorrectiveExtent, CorrectiveFeatureIndex, CorrectiveHiddenIndex, CorrectiveImportError,
};

mod archive;
mod import;

/// The first two joints do not define a local pose, so they carry no feature.
pub const SKIPPED_JOINTS: NpyDimension = NpyDimension::new(2);

pub struct PoseCorrectives {
    /// Sparse activation layer, densified and transposed: `[posed * 6, hidden]`.
    activation: Tensor<2>,
    /// Corrective basis: `[hidden, vertices * 3]`.
    basis: Tensor<2>,
    layout: CorrectiveLayout,
}

struct AxisSinCos {
    sine: Tensor<3>,
    cosine: Tensor<3>,
}

enum EulerAxis {
    X,
    Y,
    Z,
}

impl PoseCorrectives {
    /// Loads the corrective network for one level of detail, or `None` when the
    /// LOD ships no corrective basis.
    ///
    /// `activation_path` holds the sparse mask and weights shared by every LOD;
    /// `basis_path` holds this LOD's corrective basis.
    pub fn load(
        activation_path: &Path,
        basis_path: &Path,
        num_joints: RigJointCount,
        num_vertices: MeshVertexCount,
        device: &Device,
    ) -> Result<Option<Self>> {
        Self::from_archives(
            CorrectiveArchives::open(activation_path, basis_path)?,
            num_joints,
            num_vertices,
            device,
        )
    }

    /// Loads corrective archives already held in memory (for web/streamed assets).
    pub fn from_bytes(
        archives: CorrectiveArchiveBytes,
        num_joints: RigJointCount,
        num_vertices: MeshVertexCount,
        device: &Device,
    ) -> Result<Option<Self>> {
        Self::from_archives(
            CorrectiveArchives::from_bytes(archives)?,
            num_joints,
            num_vertices,
            device,
        )
    }

    fn from_archives(
        archives: CorrectiveArchives,
        num_joints: RigJointCount,
        num_vertices: MeshVertexCount,
        device: &Device,
    ) -> Result<Option<Self>> {
        let Some(array) = archives.basis()? else {
            return Ok(None);
        };
        let basis = CorrectiveBasis::new(array, num_joints, num_vertices)?;
        let activation = archives.activation(basis.layout())?.into_tensor(device);
        let tensor = basis.to_tensor(device)?;
        Ok(Some(Self {
            activation,
            basis: tensor,
            layout: basis.into_layout(),
        }))
    }

    pub fn num_vertices(&self) -> MeshVertexCount {
        self.layout.vertices()
    }

    /// Per-vertex corrective offsets `[batch, vertices, 3]`, from joint
    /// parameters shaped `[batch, joints, 7]`. Batch-dependent layout products
    /// are admitted before the corresponding tensor operations.
    pub fn forward(
        &self,
        joint_parameters: Tensor<3>,
    ) -> std::result::Result<Tensor<3>, NpyLayoutError> {
        let batch = NpyDimension::from(joint_parameters.dims()[0]);
        let output_dimensions = self.layout.output_dimensions(batch)?;
        let features = self.pose_features(joint_parameters)?;
        let hidden = activation::relu(features.matmul(self.activation.clone()));
        Ok(hidden.matmul(self.basis.clone()).reshape(output_dimensions))
    }

    /// The 6D rotation feature per posed joint, flattened to `[batch, posed * 6]`.
    ///
    /// A 6D feature is the first two columns of the joint's rotation matrix,
    /// with 1 subtracted from the two diagonal entries so a rest pose is zero.
    fn pose_features(
        &self,
        joint_parameters: Tensor<3>,
    ) -> std::result::Result<Tensor<2>, NpyLayoutError> {
        let batch = NpyDimension::from(joint_parameters.dims()[0]);
        let feature_dimensions = self.layout.feature_dimensions(batch)?;
        let euler = joint_parameters
            .narrow(
                1,
                usize::from(SKIPPED_JOINTS),
                usize::from(self.layout.posed_joints()),
            )
            .narrow(2, 3, 3);

        let AxisSinCos {
            sine: sx,
            cosine: cx,
        } = EulerAxis::X.sin_cos(&euler);
        let AxisSinCos {
            sine: sy,
            cosine: cy,
        } = EulerAxis::Y.sin_cos(&euler);
        let AxisSinCos {
            sine: sz,
            cosine: cz,
        } = EulerAxis::Z.sin_cos(&euler);

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
        Ok(feature.reshape(feature_dimensions))
    }
}

impl EulerAxis {
    fn sin_cos(self, euler: &Tensor<3>) -> AxisSinCos {
        let channel = match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        };
        let angle = euler.clone().narrow(2, channel, 1);
        AxisSinCos {
            sine: angle.clone().sin(),
            cosine: angle.cos(),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod value_tests;
