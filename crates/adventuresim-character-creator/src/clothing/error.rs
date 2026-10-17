//! Stable clothing admission and refitting failures with retained provenance.

use fabelgeist_rig::RigJointLookupError;

/// Display identity of the garment whose fitting failed, not a rig joint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClothingPieceName(String);

/// The rejected triangle's original encoded vertex slots.
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl From<&str> for ClothingPieceName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ClothingPieceName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for ClothingPieceName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for ClothingPieceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<[u32; 3]> for ClothingFaceContext {
    fn from(face: [u32; 3]) -> Self {
        Self(face)
    }
}

impl std::fmt::Display for ClothingFaceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
