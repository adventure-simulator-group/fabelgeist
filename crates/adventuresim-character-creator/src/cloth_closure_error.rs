//! Admission errors shared by cloth edges, fasteners and lacing construction.
use fabelgeist_math::units::Metres;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ClothClosureError>;

#[derive(Debug, Error)]
pub(crate) enum ClothClosureError {
    #[error("closure lies outside its physical opening edges")]
    OutsideOpening,
    #[error("closure edge stations must advance")]
    NonAdvancingEdge,
    #[error("closure edge source is absent from fitted cloth")]
    MissingSource,
    #[error("cloth closure edges coincide")]
    CoincidentEdges,
    #[error("degenerate cloth closure frame")]
    DegenerateFrame,
    #[error("eye footprint requires a hook and eye")]
    EyeRequiresHook,
    #[error("return attachment requires a hook")]
    ReturnRequiresHook,
    #[error("hook cannot bridge the opening: gap={gap:?}, reach={reach:?}")]
    InsufficientHookReach { gap: Metres, reach: Metres },
    #[error("lacing backing support is invalid")]
    InvalidBackingSupport,
}
