//! Failure to obtain a host value from a reduction result.

/// An empty reduction has no component to project to a host number.
#[derive(Debug, thiserror::Error)]
#[error("An empty reduction has no numeric result")]
pub struct EmptyReductionResult;
