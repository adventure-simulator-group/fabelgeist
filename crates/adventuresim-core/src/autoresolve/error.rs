use crate::battle_time::BattleTimeError;
use crate::elapsed::ElapsedError;
use crate::unit_interval::FractionError;

/// Checked physical inputs and interval arithmetic used during pure resolution.
#[derive(Debug, thiserror::Error)]
pub enum BattleResolutionError {
    #[error(transparent)]
    Speed(#[from] crate::linear_speed::SpeedError),
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error(transparent)]
    BattleTime(#[from] BattleTimeError),
    #[error(transparent)]
    Performance(#[from] FractionError),
}

pub type BattleResolutionResult<T> = Result<T, BattleResolutionError>;
