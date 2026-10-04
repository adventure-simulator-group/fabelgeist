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
    #[error("{0} is not built on the device yet")]
    NotOnDevice(&'static str),
    #[error("armor GPU: {0}")]
    Gpu(std::sync::Arc<str>),
    #[error("armor GPU: {0}")]
    Kernel(#[from] fabelgeist_compute::KernelCacheError),
    #[error("armor GPU: {0}")]
    BufferCreation(#[from] fabelgeist_gpu::prelude::BufferCreationError),
    #[error("armor GPU: {0}")]
    Dispatch(#[from] fabelgeist_compute::KernelDispatchError),
    #[error("armor GPU readback: {0}")]
    Readback(#[from] fabelgeist_gpu::prelude::ReadbackError),
}
