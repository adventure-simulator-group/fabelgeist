//! MHR's corrective archive schema and its fixed array roles.

use std::path::Path;

use anyhow::{Context, Result};
use fabelgeist_numpy_storage::{
    ArchiveMemberPresence, NpyArray, NpyFloat32Values, NpyIntegerValues, Npz, NpzArrayName,
};

use super::import::SparseActivation;

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

    pub(super) fn from_bytes(activation: Vec<u8>, basis: Vec<u8>) -> Result<Self> {
        let activation = Npz::from_bytes(activation)?;
        let basis = Npz::from_bytes(basis)?;
        Ok(Self { activation, basis })
    }

    pub(super) fn basis(&self) -> Result<Option<NpyArray>> {
        let name = CorrectiveArray::Basis.name();
        if self.basis.contains(&name) == ArchiveMemberPresence::Absent {
            return Ok(None);
        }
        self.basis
            .array(&name)
            .context("reading the corrective basis")
            .map(Some)
    }

    pub(super) fn activation(&self) -> Result<SparseActivation> {
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
        Ok(SparseActivation::new(indices, weights)?)
    }
}
