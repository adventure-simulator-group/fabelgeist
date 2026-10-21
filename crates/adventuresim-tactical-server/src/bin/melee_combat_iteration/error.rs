//! Failures from a melee iteration run, retaining the pure resolution cause.

use adventuresim_core::autoresolve::BattleResolutionError;

pub(crate) type IterationResult<T> = Result<T, IterationError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum IterationError {
    #[error(transparent)]
    Resolution(#[from] BattleResolutionError),
    #[error("{0}")]
    Operation(String),
}

impl From<String> for IterationError {
    fn from(message: String) -> Self {
        Self::Operation(message)
    }
}

impl From<&str> for IterationError {
    fn from(message: &str) -> Self {
        Self::Operation(message.to_owned())
    }
}
