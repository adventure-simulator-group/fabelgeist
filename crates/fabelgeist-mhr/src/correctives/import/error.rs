//! Corrective-array admission failures, distinct from archive/parser causes.

use std::{error::Error, fmt};

use fabelgeist_numpy_storage::{NpyDimension, NpyElementCount, NpyShape};

pub type Result<T> = std::result::Result<T, CorrectiveImportError>;

/// The native tensor extent whose checked calculation overflowed.
#[derive(Debug, Clone, Copy)]
pub enum CorrectiveExtent {
    HiddenUnits,
    FeatureChannels,
    BasisColumns,
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
        mesh: NpyDimension,
    },
    JointCount {
        joints: NpyDimension,
    },
    Components {
        actual: NpyDimension,
        expected: NpyDimension,
    },
    ExtentOverflow {
        extent: CorrectiveExtent,
    },
    SparseCardinality {
        indices: NpyElementCount,
        weights: NpyElementCount,
    },
    SparseCoordinate {
        /// Rejected coordinate after the native indexing cast, for diagnostics.
        native_row: usize,
        native_column: usize,
    },
}

impl fmt::Display for CorrectiveImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasisShape { shape } => write!(
                f,
                "{} has shape {shape:?}, expected [components, vertices, 3]",
                super::super::BASIS_ARRAY
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
            Self::ExtentOverflow { extent } => {
                write!(f, "corrective tensor extent overflows for {extent:?}")
            }
            Self::SparseCardinality { indices, weights } => write!(
                f,
                "sparse activation has {indices} indices for {weights} weights"
            ),
            Self::SparseCoordinate {
                native_row,
                native_column,
            } => write!(
                f,
                "sparse activation index ({native_row}, {native_column}) is out of bounds"
            ),
        }
    }
}

impl Error for CorrectiveImportError {}
