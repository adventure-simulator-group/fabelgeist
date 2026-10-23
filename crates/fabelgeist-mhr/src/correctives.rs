//! The non-linear pose-corrective network.
//!
//! Joint rotations are turned into a 6D rotation feature per joint, pushed
//! through a sparse 750 -> 3000 layer, a ReLU, and a dense 3000 -> `3 * V`
//! layer, giving a per-vertex offset applied before skinning.

use std::path::Path;

use anyhow::{Context, Result};
use burn::tensor::{Device, Tensor, TensorData, activation};
use fabelgeist_numpy_storage::{
    ArchiveMemberPresence, NpyDimension, NpyFloatValues, NpyIntegerValues, Npz, NpzArrayName,
};

pub use import::{CorrectiveExtent, CorrectiveImportError};
use import::{CorrectiveLayout, SparseActivation};

mod import;

/// The first two joints do not define a local pose, so they carry no feature.
pub const SKIPPED_JOINTS: usize = 2;

/// Names of the arrays MHR stores the network in.
const BASIS_ARRAY: NpzArrayName<'static> = NpzArrayName::new("corrective_blendshapes");
const SPARSE_INDICES_ARRAY: NpzArrayName<'static> = NpzArrayName::new("0.sparse_indices");
const SPARSE_WEIGHT_ARRAY: NpzArrayName<'static> = NpzArrayName::new("0.sparse_weight");

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
        num_joints: usize,
        num_vertices: usize,
        device: &Device,
    ) -> Result<Option<Self>> {
        let archive = Npz::open(basis_path)?;
        let activation = Npz::open(activation_path)?;
        Self::from_archives(
            activation,
            archive,
            NpyDimension::from(num_joints),
            NpyDimension::from(num_vertices),
            device,
        )
    }

    /// Loads corrective archives already held in memory (for web/streamed assets).
    pub fn from_bytes(
        activation: Vec<u8>,
        basis: Vec<u8>,
        num_joints: usize,
        num_vertices: usize,
        device: &Device,
    ) -> Result<Option<Self>> {
        Self::from_archives(
            Npz::from_bytes(activation)?,
            Npz::from_bytes(basis)?,
            NpyDimension::from(num_joints),
            NpyDimension::from(num_vertices),
            device,
        )
    }

    fn from_archives(
        activation: Npz,
        archive: Npz,
        num_joints: NpyDimension,
        num_vertices: NpyDimension,
        device: &Device,
    ) -> Result<Option<Self>> {
        if archive.contains(&BASIS_ARRAY) == ArchiveMemberPresence::Absent {
            return Ok(None);
        }
        let basis = archive
            .array(&BASIS_ARRAY)
            .context("reading the corrective basis")?;
        let layout = CorrectiveLayout::new(basis.shape(), num_joints, num_vertices)?;
        let basis_values =
            NpyFloatValues::try_from(&basis).context("decoding the corrective basis")?;

        let indices = NpyIntegerValues::try_from(
            &activation
                .array(&SPARSE_INDICES_ARRAY)
                .context("reading the sparse activation indices")?,
        )
        .context("decoding the sparse activation indices")?;
        let values = NpyFloatValues::try_from(
            &activation
                .array(&SPARSE_WEIGHT_ARRAY)
                .context("reading the sparse activation weights")?,
        )
        .context("decoding the sparse activation weights")?;
        let activation = SparseActivation::new(indices, values)?.into_tensor(&layout, device)?;

        Ok(Some(Self {
            activation,
            basis: Tensor::from_data(
                TensorData::new(Vec::<f32>::from(basis_values), layout.basis_dimensions()),
                device,
            ),
            layout,
        }))
    }

    pub fn num_vertices(&self) -> NpyDimension {
        self.layout.vertices()
    }

    /// Per-vertex corrective offsets `[batch, vertices, 3]`, from joint
    /// parameters shaped `[batch, joints, 7]`.
    pub fn forward(&self, joint_parameters: Tensor<3>) -> Tensor<3> {
        let batch = joint_parameters.dims()[0];
        let features = self.pose_features(joint_parameters);
        let hidden = activation::relu(features.matmul(self.activation.clone()));
        hidden
            .matmul(self.basis.clone())
            .reshape([batch, usize::from(self.layout.vertices()), 3])
    }

    /// The 6D rotation feature per posed joint, flattened to `[batch, posed * 6]`.
    ///
    /// A 6D feature is the first two columns of the joint's rotation matrix,
    /// with 1 subtracted from the two diagonal entries so a rest pose is zero.
    fn pose_features(&self, joint_parameters: Tensor<3>) -> Tensor<2> {
        let batch = joint_parameters.dims()[0];
        let euler = joint_parameters
            .narrow(1, SKIPPED_JOINTS, usize::from(self.layout.posed_joints()))
            .narrow(2, 3, 3);

        let (sx, cx) = axis_sin_cos(&euler, 0);
        let (sy, cy) = axis_sin_cos(&euler, 1);
        let (sz, cz) = axis_sin_cos(&euler, 2);

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
        feature.reshape([batch, usize::from(self.layout.features())])
    }
}

fn axis_sin_cos(euler: &Tensor<3>, axis: usize) -> (Tensor<3>, Tensor<3>) {
    let angle = euler.clone().narrow(2, axis, 1);
    (angle.clone().sin(), angle.cos())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod value_tests;
