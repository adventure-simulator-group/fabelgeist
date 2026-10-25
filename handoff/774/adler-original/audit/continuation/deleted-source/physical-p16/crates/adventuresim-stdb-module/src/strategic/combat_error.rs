//! Strategic combat failures stay structured until the reducer runtime renders them.

use adventuresim_core::autoresolve::{AuthoredThreatError, BattleResolutionError};

pub type StrategicCombatResult<T> = Result<T, StrategicCombatError>;

#[derive(Debug, thiserror::Error)]
pub enum StrategicCombatError {
    #[error(transparent)]
    AuthoredThreat(#[from] AuthoredThreatError),
    #[error(transparent)]
    Resolution(#[from] BattleResolutionError),
    #[error("Autopsy demo autoresolve could not produce its required death")]
    RequiredDeathUnavailable,
    #[error("{0}")]
    Operation(String),
}

impl From<String> for StrategicCombatError {
    fn from(message: String) -> Self {
        Self::Operation(message)
    }
}

impl From<&str> for StrategicCombatError {
    fn from(message: &str) -> Self {
        Self::Operation(message.to_owned())
    }
}
