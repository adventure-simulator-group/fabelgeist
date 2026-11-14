//! Failures admitting native body skin or transferring it to cloth.
use fabelgeist_rig::{RigJointLookupError, RigJointName, RigSkinError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SkinTransferError {
    #[error(transparent)]
    JointLookup(#[from] RigJointLookupError),
    #[error(transparent)]
    PackedSkin(#[from] RigSkinError),
    #[error("non-finite skin query at {position:?}")]
    NonFiniteQuery { position: [f32; 3] },
    #[error("no body surface for garment skin transfer at {position:?}")]
    MissingBodySurface { position: [f32; 3] },
    #[error("skirt hip landmarks coincide")]
    CoincidentHips,
    #[error("skirt knees must be below its hips")]
    KneesAboveHips,
    #[error("missing transform for garment landmark {joint}")]
    MissingLandmark { joint: RigJointName },
    #[error("non-finite garment landmark {joint}")]
    NonFiniteLandmark { joint: RigJointName },
    #[error("body skin references a missing rig joint")]
    InvalidRigBinding,
    #[error("body skin arrays do not match its vertices")]
    MismatchedVertexArrays,
    #[error("invalid body surface for garment skin transfer")]
    InvalidBodySurface,
    #[error("invalid cloth skin source table")]
    InvalidSourceTable,
    #[error("invalid cloth source weight")]
    NonFiniteSourceWeight,
    #[error("cloth skin source vertex {vertex} is outside its surface")]
    SourceVertexOutsideSurface { vertex: usize },
    #[error("cloth skin source vertex {vertex} has no weights")]
    MissingSourceWeights { vertex: usize },
    #[error("garment vertex has no skin weights")]
    MissingInfluences,
}
