//! Native command failures retain simulation, report and external I/O causes.

use std::path::PathBuf;

use adventuresim_core::{
    autoresolve::{BattleResolutionError, MeleeIterationError},
    elapsed::ElapsedError,
};
use adventuresim_tactical_core::animation::AnimationValueError;

use super::measurement::MeasurementError;

pub(crate) type IterationResult<T> = Result<T, IterationError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum IterationError {
    #[error(transparent)]
    Resolution(#[from] BattleResolutionError),
    #[error(transparent)]
    Preparation(#[from] MeleeIterationError),
    #[error(transparent)]
    Geometry(#[from] fabelgeist_geometry::GeometryError),
    #[error(transparent)]
    Performance(#[from] adventuresim_core::unit_interval::FractionError),
    #[error(transparent)]
    Measurement(#[from] AnimationValueError),
    #[error(transparent)]
    Statistics(#[from] MeasurementError),
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error(transparent)]
    ReportContract(#[from] ReportContractError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("{}: {source}", .path.display())]
    WriteReport {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("seed counts must both be positive (tactical: {tactical}, autoresolve: {autoresolve})")]
    InvalidSeedCounts { tactical: u64, autoresolve: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MovementObservation {
    VelocityBefore,
    VelocityAfter,
    SpeedLimit,
    Elapsed,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ReportContractError {
    #[error("autoresolve contact has no recoverable interval at this performance snapshot")]
    UnavailableRecoveryInterval,
    #[error("autoresolve movement record {sequence} is missing its {observation:?} observation")]
    MissingMovementObservation {
        sequence: u64,
        observation: MovementObservation,
    },
    #[error("unknown tactical victor {victor}")]
    UnknownTacticalVictor { victor: String },
}

impl From<adventuresim_tactical_core::combat::MeleeGeometryError> for IterationError {
    fn from(error: adventuresim_tactical_core::combat::MeleeGeometryError) -> Self {
        use adventuresim_tactical_core::combat::MeleeGeometryError;
        match error {
            MeleeGeometryError::Geometry(error) => Self::Geometry(error),
            MeleeGeometryError::Animation(error) => Self::Measurement(error),
            MeleeGeometryError::Timing(error) => Self::Timing(error),
            MeleeGeometryError::Performance(error) => Self::Performance(error),
        }
    }
}
