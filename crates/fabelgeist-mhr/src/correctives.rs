//! The non-linear pose-corrective network.
//!
//! Joint rotations are turned into a 6D rotation feature per joint, pushed
//! through a sparse 750 -> 3000 layer, a ReLU, and a dense 3000 -> `3 * V`
//! layer, giving a per-vertex offset applied before skinning.

use std::path::Path;

use anyhow::Result;
use burn::tensor::{Device, Tensor, activation};
use fabelgeist_numpy_storage::NpyDimension;
use fabelgeist_rig::RigJointCount;

use crate::character::MeshVertexCount;

pub use archive::CorrectiveArchiveBytes;
use archive::CorrectiveArchives;
use import::{CorrectiveBasis, CorrectiveLayout};
pub use import::{
    CorrectiveExtent, CorrectiveFeatureIndex, CorrectiveHiddenIndex, CorrectiveImportError,
};
pub use input::CorrectiveForwardError;
use input::CorrectiveInput;

mod archive;
mod import;
mod input;

/// The first two joints do not define a local pose, so they carry no feature.
pub const SKIPPED_JOINTS: NpyDimension = NpyDimension::new(2);

pub struct PoseCorrectives {
    /// Sparse activation layer, densified and transposed: `[posed * 6, hidden]`.
    activation: Tensor<2>,
    /// Corrective basis: `[hidden, vertices * 3]`.
    basis: Tensor<2>,
    layout: CorrectiveLayout,
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
    /// parameters in the model's seven-channel order. Joint count and channel
    /// width must match the admitted network before any tensor operation.
    /// Batch-dependent products remain fallible layout admissions.
    pub fn forward(
        &self,
        joint_parameters: Tensor<3>,
    ) -> std::result::Result<Tensor<3>, CorrectiveForwardError> {
        let input = CorrectiveInput::new(joint_parameters, &self.layout)?;
        let output_dimensions = input.output_dimensions()?;
        let features = input.into_features()?;
        let hidden = activation::relu(features.matmul(self.activation.clone()));
        Ok(hidden.matmul(self.basis.clone()).reshape(output_dimensions))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod value_tests;
