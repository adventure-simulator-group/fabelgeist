//! Shell admission retains input roles before allocating simulation state.
use crate::{
    ParticleArealDensity, ParticleInputCount, ParticleInputIndex, ParticleMass, ParticleMassCount,
};
use fabelgeist_math::Vec3;
use fabelgeist_xpbd::{ConstraintCount, ParticleIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellConstraintFamily {
    Stretch,
    Seams,
    Bending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellParticleReference {
    Triangle,
    Stretch,
    Seam,
}

#[derive(Debug)]
pub enum ShellMeshError {
    TriangleSource {
        source: fabelgeist_mesh::MeshTriangleError,
    },
    ArealDensity {
        density: ParticleArealDensity,
    },
    ParticleCount {
        actual: ParticleInputCount,
    },
    NonFinitePosition {
        particle: ParticleInputIndex,
        position: Vec3,
    },
    MassCount {
        particles: ParticleInputCount,
        masses: ParticleMassCount,
    },
    InvalidMass {
        particle: ParticleInputIndex,
        mass: ParticleMass,
    },
    ConstraintDataLength {
        family: ShellConstraintFamily,
        constraints: ConstraintCount,
        records: ConstraintCount,
    },
    TriangleIndex {
        index: ParticleIndex,
        particles: ParticleInputCount,
    },
    IndexOutOfBounds {
        family: ShellParticleReference,
        index: ParticleIndex,
        particles: ParticleInputCount,
    },
    BendIndex {
        index: ParticleIndex,
        particles: ParticleInputCount,
    },
    RestData {
        family: ShellConstraintFamily,
    },
}

impl From<fabelgeist_mesh::MeshTriangleError> for ShellMeshError {
    fn from(source: fabelgeist_mesh::MeshTriangleError) -> Self {
        Self::TriangleSource { source }
    }
}

impl std::fmt::Display for ShellMeshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TriangleSource { source } => source.fmt(formatter),
            Self::ArealDensity { .. } => formatter.write_str("areal density must be positive"),
            Self::ParticleCount { .. } => formatter.write_str("invalid shell particle count"),
            Self::NonFinitePosition { .. } => formatter.write_str("non-finite shell position"),
            Self::MassCount { .. } | Self::InvalidMass { .. } => {
                formatter.write_str("invalid shell masses")
            }
            Self::ConstraintDataLength { .. } => {
                formatter.write_str("constraint data length mismatch")
            }
            Self::TriangleIndex { .. } => formatter.write_str("shell triangle index out of bounds"),
            Self::IndexOutOfBounds { .. } => formatter.write_str("shell index out of bounds"),
            Self::BendIndex { .. } => formatter.write_str("bend index out of bounds"),
            Self::RestData { .. } => formatter.write_str("invalid shell rest data"),
        }
    }
}

impl std::error::Error for ShellMeshError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TriangleSource { source } => Some(source),
            _ => None,
        }
    }
}
