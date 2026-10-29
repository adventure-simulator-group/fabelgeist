//! MHR's corrective archive schema and its fixed array roles.

use std::path::Path;

use anyhow::{Context, Result};
use fabelgeist_numpy_storage::{NpyArray, NpyFloat32Values, NpyIntegerValues, Npz, NpzArrayName};

use super::import::{CorrectiveLayout, SparseActivation};

/// Encoded corrective archives with explicit roles before NPZ admission.
/// Payload contents are checked when the archives and their members are read.
pub struct CorrectiveArchiveBytes {
    pub activation: Vec<u8>,
    pub basis: Vec<u8>,
}

pub(super) struct CorrectiveArchives {
    activation: Npz,
    basis: Npz,
}

pub(super) enum CorrectiveArray {
    Basis,
    SparseIndices,
    SparseWeights,
}

impl CorrectiveArray {
    pub(super) fn name(self) -> NpzArrayName {
        NpzArrayName::new(match self {
            Self::Basis => "corrective_blendshapes",
            Self::SparseIndices => "0.sparse_indices",
            Self::SparseWeights => "0.sparse_weight",
        })
    }
}

impl CorrectiveArchives {
    pub(super) fn open(activation: &Path, basis: &Path) -> Result<Self> {
        let basis = Npz::open(basis)?;
        let activation = Npz::open(activation)?;
        Ok(Self { activation, basis })
    }

    pub(super) fn from_bytes(archives: CorrectiveArchiveBytes) -> Result<Self> {
        let activation = Npz::from_bytes(archives.activation)?;
        let basis = Npz::from_bytes(archives.basis)?;
        Ok(Self { activation, basis })
    }

    pub(super) fn basis(&self) -> Result<Option<NpyArray>> {
        let name = CorrectiveArray::Basis.name();
        let Some(member) = self.basis.get(&name) else {
            return Ok(None);
        };
        member
            .array()
            .context("reading the corrective basis")
            .map(Some)
    }

    pub(super) fn activation<'layout>(
        &self,
        layout: &'layout CorrectiveLayout,
    ) -> Result<SparseActivation<'layout>> {
        let indices = NpyIntegerValues::from(
            &self
                .activation
                .array(&CorrectiveArray::SparseIndices.name())
                .context("reading the sparse activation indices")?,
        );
        let weights = NpyFloat32Values::from(
            &self
                .activation
                .array(&CorrectiveArray::SparseWeights.name())
                .context("reading the sparse activation weights")?,
        );
        Ok(SparseActivation::new(indices, weights, layout)?)
    }
}
