//! Particle state admission and device failures retain their stage and cause.
use super::{InverseMassCount, ParticleCapacity, ParticleCount, ParticleInputCount};
use fabelgeist_gpu::prelude::{BufferByteLength, BufferCreationError, ReadbackError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleBufferRole {
    Positions,
    Previous,
    Velocities,
}

#[derive(Debug)]
pub enum ParticleError {
    Allocation {
        role: ParticleBufferRole,
        capacity: ParticleCapacity,
        bytes: BufferByteLength,
        source: BufferCreationError,
    },
    InputLength {
        positions: ParticleInputCount,
        masses: InverseMassCount,
    },
    Capacity {
        capacity: ParticleCapacity,
        actual: ParticleInputCount,
    },
    PositionCount {
        expected: ParticleCount,
        actual: ParticleInputCount,
    },
    Readback {
        role: ParticleBufferRole,
        source: Box<ReadbackError>,
    },
}
impl std::fmt::Display for ParticleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocation { source, .. } => source.fmt(formatter),
            Self::InputLength { positions, masses } => write!(
                formatter,
                "Particles::write: {positions} positions but {masses} inverse masses"
            ),
            Self::Capacity { capacity, actual } => write!(
                formatter,
                "Particles::write: capacity is {capacity}, given {actual}"
            ),
            Self::PositionCount { expected, actual } => write!(
                formatter,
                "Particles::write_positions: holds {expected} particles, given {actual}"
            ),
            Self::Readback { source, .. } => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ParticleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            Self::Readback { source, .. } => Some(source.as_ref()),
            Self::InputLength { .. } | Self::Capacity { .. } | Self::PositionCount { .. } => None,
        }
    }
}
