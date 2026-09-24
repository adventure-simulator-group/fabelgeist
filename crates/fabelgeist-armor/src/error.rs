//! Why armor generation failed.

use thiserror::Error;

use crate::DesignError;

#[derive(Debug, Error)]
pub enum GenerateError {
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
    #[error("generated bracer geometry is degenerate")]
    Degenerate,
    #[error("armor GPU: {0}")]
    Gpu(std::sync::Arc<str>),
}
