//! Surface projection retains the failing host interval or state-read stage.
use fabelgeist_gpu::prelude::ReadbackError;
use fabelgeist_xpbd::{ParticleCount, ParticleError, ParticleInputCount};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceProjectionStage {
    PositionsRead,
    VelocitiesRead,
    PositionEncoding,
}

#[derive(Debug)]
pub enum SurfaceProjectionError {
    IntervalCount {
        expected: ParticleCount,
        actual: ParticleInputCount,
    },
    PreviousReadback {
        source: Box<ReadbackError>,
    },
    ParticleState {
        stage: SurfaceProjectionStage,
        source: ParticleError,
    },
}
impl std::fmt::Display for SurfaceProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntervalCount { .. } => {
                formatter.write_str("interval start does not match the particle count")
            }
            Self::PreviousReadback { source } => source.fmt(formatter),
            Self::ParticleState { source, .. } => source.fmt(formatter),
        }
    }
}
impl std::error::Error for SurfaceProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IntervalCount { .. } => None,
            Self::PreviousReadback { source } => Some(source.as_ref()),
            Self::ParticleState { source, .. } => Some(source),
        }
    }
}
