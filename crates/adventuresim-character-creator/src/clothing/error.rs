//! Stable clothing admission and refitting failures with retained provenance.

use derive_more::{AsRef, Display, From};
use fabelgeist_rig::RigJointLookupError;

/// Display identity of the garment whose fitting failed, not a rig joint.
#[derive(Clone, Debug, PartialEq, Eq, AsRef, Display, From)]
#[from(String, &str)]
#[display("{}", format_args!("{_0}"))]
pub struct ClothingPieceName(#[as_ref(str)] String);

/// The rejected triangle's original encoded vertex slots.
#[derive(Clone, Debug, PartialEq, Eq, Display, From)]
#[display("{}", format_args!("{_0:?}"))]
pub struct ClothingFaceContext([u32; 3]);

#[derive(Debug, thiserror::Error)]
pub enum ClothingError {
    #[error("clothing inputs have inconsistent vertex or joint counts")]
    InconsistentBodyInputs,
    #[error("clothing topology references a missing vertex")]
    MissingBodyVertex,
    #[error(transparent)]
    MissingLandmark(#[from] RigJointLookupError),
    #[error("{garment} has no anatomical surface spans")]
    NoSurfaceSpans { garment: ClothingPieceName },
    #[error("{garment} selected no weighted vertices")]
    NoWeightedVertices { garment: ClothingPieceName },
    #[error("{garment} selected no waist vertices")]
    NoWaistVertices { garment: ClothingPieceName },
    #[error("{garment} selected no MHR triangles")]
    NoSelectedTriangles { garment: ClothingPieceName },
    #[error("{garment} contains a non-finite fitted vertex")]
    NonFiniteFittedVertex { garment: ClothingPieceName },
    #[error("{garment} contains an out-of-range face: {face}")]
    OutOfRangeFace {
        garment: ClothingPieceName,
        face: ClothingFaceContext,
    },
    #[error("garment morph changed body vertex correspondence")]
    MorphCorrespondenceChanged,
    #[error("garment morph invalidated base triangles")]
    MorphTrianglesInvalidated,
}
