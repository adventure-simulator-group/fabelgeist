//! Why armor generation failed.

use fabelgeist_compute::ReadbackError;
use thiserror::Error;

use crate::DesignError;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("armor skin: {0}")]
    Skin(#[from] fabelgeist_rig::RigSkinError),
    #[error("armor GPU readback: {0}")]
    Readback(#[from] ReadbackError),
    #[error("armor staged readback: {0}")]
    Staging(#[from] crate::gpu::staging::StagingError),
    #[error("armor plate shell: {0}")]
    PlateShell(#[from] PlateShellFailure),
    #[error(
        "sabaton ankle cutaway {cutaway_m} m must be smaller than the available instep span {available_span_m} m"
    )]
    SabatonTrimExceedsFoot {
        cutaway_m: f32,
        available_span_m: f32,
    },
    #[error("invalid bracer design: {0}")]
    Design(#[from] DesignError),
    #[error("anatomical surface arrays or topology are inconsistent")]
    InvalidSurface,
    #[error("anatomical surface has no closed forearm contour at the requested placement")]
    EmptySelection,
    #[error("generated armor geometry is degenerate")]
    Degenerate,
    #[error("armor GPU: {0}")]
    Gpu(#[source] anyhow::Error),
}

/// An evaluated metal wall has unusable facets.
#[derive(Clone, Copy, Debug, Error)]
pub enum PlateShellFailure {
    #[error("a wall face is collapsed or has nonfinite coordinates")]
    CollapsedFace,
}
