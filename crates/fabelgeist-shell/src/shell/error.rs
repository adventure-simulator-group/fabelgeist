//! Shell construction and host projection retain concrete lower-layer causes.
use crate::surface_contact::SurfaceProjectionError;
use crate::{SelfCollisionBuildError, ShellConstraintFamily, ShellMaterialError, ShellMeshError};
use fabelgeist_compute::KernelCacheError;
use fabelgeist_xpbd::{
    ConstraintAttachmentError, ConstraintBuildError, ConstraintLayoutError, ParticleError,
    SolverStepError,
};

#[derive(Debug)]
pub enum ShellBuildError {
    Mesh(ShellMeshError),
    Material(ShellMaterialError),
    Particles(ParticleError),
    Constraint {
        family: ShellConstraintFamily,
        source: Box<ConstraintBuildError>,
    },
    BendLayout(ConstraintLayoutError),
    BendKernel(Box<KernelCacheError>),
    BendAttachment(ConstraintAttachmentError),
    SelfCollision(Box<SelfCollisionBuildError>),
}
impl ShellBuildError {
    pub(super) fn constraint(family: ShellConstraintFamily, source: ConstraintBuildError) -> Self {
        Self::Constraint {
            family,
            source: Box::new(source),
        }
    }
}
impl std::fmt::Display for ShellBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mesh(source) => source.fmt(formatter),
            Self::Material(source) => source.fmt(formatter),
            Self::Particles(source) => source.fmt(formatter),
            Self::Constraint { source, .. } => source.fmt(formatter),
            Self::BendLayout(source) => source.fmt(formatter),
            Self::BendKernel(source) => source.fmt(formatter),
            Self::BendAttachment(source) => source.fmt(formatter),
            Self::SelfCollision(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ShellBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Mesh(source) => Some(source),
            Self::Material(source) => Some(source),
            Self::Particles(source) => Some(source),
            Self::Constraint { source, .. } => Some(source.as_ref()),
            Self::BendLayout(source) => Some(source),
            Self::BendKernel(source) => Some(source.as_ref()),
            Self::BendAttachment(source) => Some(source),
            Self::SelfCollision(source) => Some(source.as_ref()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellProjectionStage {
    OuterLayer,
    ResidualPositions,
}
#[derive(Debug)]
pub enum ShellProjectionError {
    ParticleState {
        stage: ShellProjectionStage,
        source: ParticleError,
    },
    SurfaceContact(SurfaceProjectionError),
}
impl std::fmt::Display for ShellProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParticleState { source, .. } => source.fmt(formatter),
            Self::SurfaceContact(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ShellProjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParticleState { source, .. } => Some(source),
            Self::SurfaceContact(source) => Some(source),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellPositionValidity {
    Finite,
    NonFinite,
}

#[derive(Debug)]
pub enum ShellHookError {
    Body(Box<fabelgeist_physics::CollisionRecordError>),
    SelfCollision(Box<crate::SelfCollisionRecordError>),
}
impl ShellHookError {
    pub(super) fn body(source: fabelgeist_physics::CollisionRecordError) -> Self {
        Self::Body(Box::new(source))
    }
    pub(super) fn self_collision(source: crate::SelfCollisionRecordError) -> Self {
        Self::SelfCollision(Box::new(source))
    }
}
impl std::fmt::Display for ShellHookError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Body(source) => source.fmt(formatter),
            Self::SelfCollision(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ShellHookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Body(source) => Some(source.as_ref()),
            Self::SelfCollision(source) => Some(source.as_ref()),
        }
    }
}

#[derive(Debug)]
pub enum ShellStepError {
    Solver(SolverStepError<ShellHookError>),
    IntervalRead(ParticleError),
    Projection(ShellProjectionError),
}
impl std::fmt::Display for ShellStepError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solver(source) => source.fmt(formatter),
            Self::IntervalRead(source) => source.fmt(formatter),
            Self::Projection(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ShellStepError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Solver(source) => Some(source),
            Self::IntervalRead(source) => Some(source),
            Self::Projection(source) => Some(source),
        }
    }
}
