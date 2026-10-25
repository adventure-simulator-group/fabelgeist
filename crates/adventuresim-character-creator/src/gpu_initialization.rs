//! Reusable causes for a GPU initialization outcome retained by a Creator cache.

use std::{fmt, sync::Arc};

/// A shared initialization failure, retaining its original error and cause chain.
///
/// Cloning this error shares the recorded outcome; it does not retry initialization.
#[derive(Clone, Debug, thiserror::Error)]
pub struct GpuInitializationError {
    #[source]
    cause: Arc<anyhow::Error>,
}

impl GpuInitializationError {
    /// Retain the failure returned by the device or kernel initializer.
    pub fn new(cause: impl Into<anyhow::Error>) -> Self {
        Self {
            cause: Arc::new(cause.into()),
        }
    }
}

impl fmt::Display for GpuInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.cause.as_ref(), formatter)
    }
}
