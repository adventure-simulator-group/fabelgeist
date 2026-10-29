//! Corrective-array admission failures, distinct from archive/parser causes.

use std::{error::Error, fmt};

use fabelgeist_numpy_storage::{NpyDimension, NpyElementCount, NpyLayoutError, NpyShape};

use fabelgeist_rig::RigJointCount;

use super::{CorrectiveFeatureIndex, CorrectiveHiddenIndex};
use crate::character::MeshVertexCount;

pub type Result<T> = std::result::Result<T, CorrectiveImportError>;

/// The native tensor extent whose checked calculation overflowed.
#[derive(Debug, Clone, Copy)]
pub enum CorrectiveExtent {
    HiddenUnits,
    FeatureChannels,
    BasisColumns,
    BasisElements,
    DenseElements,
}

/// A rejected relationship in the corrective network's imported tensor data.
#[derive(Debug)]
pub enum CorrectiveImportError {
    BasisShape {
        shape: NpyShape,
    },
    VertexCount {
        basis: NpyDimension,
        mesh: MeshVertexCount,
    },
    JointCount {
        joints: RigJointCount,
    },
    Components {
        actual: NpyDimension,
        expected: NpyDimension,
    },
    ExtentOverflow {
        extent: CorrectiveExtent,
    },
    TensorLayout {
        extent: CorrectiveExtent,
        source: NpyLayoutError,
    },
    SparseCardinality {
        indices: NpyElementCount,
        weights: NpyElementCount,
    },
    SparseCoordinate {
        hidden: CorrectiveHiddenIndex,
        feature: CorrectiveFeatureIndex,
    },
}

impl fmt::Display for CorrectiveImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasisShape { shape } => write!(
                f,
                "{} has shape {shape:?}, expected [components, vertices, 3]",
                super::super::archive::CorrectiveArray::Basis.name()
            ),
            Self::VertexCount { basis, mesh } => write!(
                f,
                "corrective basis covers {basis} vertices but the mesh has {mesh}"
            ),
            Self::JointCount { joints } => write!(
                f,
                "corrective network has {joints} joints, fewer than the {} unposed joints",
                super::super::SKIPPED_JOINTS
            ),
            Self::Components { actual, expected } => write!(
                f,
                "corrective basis has {actual} components, expected {expected}"
            ),
            Self::ExtentOverflow { extent } | Self::TensorLayout { extent, .. } => {
                write!(f, "corrective tensor extent overflows for {extent:?}")
            }
            Self::SparseCardinality { indices, weights } => write!(
                f,
                "sparse activation has {indices} indices for {weights} weights"
            ),
            Self::SparseCoordinate { hidden, feature } => write!(
                f,
                "sparse activation index ({}, {}) is out of bounds",
                hidden.native_index(),
                feature.native_index()
            ),
        }
    }
}

impl Error for CorrectiveImportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TensorLayout { source, .. } => Some(source),
            _ => None,
        }
    }
}
