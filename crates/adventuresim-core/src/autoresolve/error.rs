use crate::battle_time::BattleTimeError;
use crate::elapsed::ElapsedError;
use crate::linear_speed::SpeedError;
use crate::unit_interval::FractionError;

/// Checked physical inputs and interval arithmetic used during pure resolution.
#[derive(Debug, thiserror::Error)]
pub enum BattleResolutionError {
    #[error(transparent)]
    Geometry(#[from] fabelgeist_geometry::GeometryError),
    #[error(transparent)]
    Speed(#[from] SpeedError),
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error(transparent)]
    BattleTime(#[from] BattleTimeError),
    #[error(transparent)]
    Performance(#[from] FractionError),
}

pub type BattleResolutionResult<T> = Result<T, BattleResolutionError>;
