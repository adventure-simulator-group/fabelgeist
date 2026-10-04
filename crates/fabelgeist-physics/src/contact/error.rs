//! Body-collision failures retain counts, resource roles and concrete causes.
use super::{ColliderCapacity, ColliderCount, CollisionKernel};
use fabelgeist_compute::kernel::{BufferCopyError, KernelCacheError, KernelDispatchError};
use fabelgeist_gpu::prelude::BufferCreationError;

#[derive(Debug)]
pub enum CollisionBuildError {
    Kernel {
        kernel: CollisionKernel,
        source: Box<KernelCacheError>,
    },
    Allocation {
        capacity: ColliderCapacity,
        source: BufferCreationError,
    },
}
impl std::fmt::Display for CollisionBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kernel { source, .. } => source.fmt(formatter),
            Self::Allocation { source, .. } => source.fmt(formatter),
        }
    }
}
impl std::error::Error for CollisionBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Kernel { source, .. } => Some(source.as_ref()),
            Self::Allocation { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum ColliderUpdateError {
    Count {
        held: ColliderCount,
        provided: ColliderCount,
    },
    Allocation {
        capacity: ColliderCapacity,
        source: BufferCreationError,
    },
}
impl std::fmt::Display for ColliderUpdateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count { held, provided } => write!(
                formatter,
                "Collisions::update_colliders: holds {held} colliders, given {provided}; use `set_colliders` to change the count"
            ),
            Self::Allocation { source, .. } => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ColliderUpdateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Count { .. } => None,
            Self::Allocation { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum CollisionRecordError {
    Dispatch {
        kernel: CollisionKernel,
        source: Box<KernelDispatchError>,
    },
    PreviousPositionsCopy(BufferCopyError),
}
impl std::fmt::Display for CollisionRecordError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispatch { source, .. } => source.fmt(formatter),
            Self::PreviousPositionsCopy(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for CollisionRecordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch { source, .. } => Some(source.as_ref()),
            Self::PreviousPositionsCopy(source) => Some(source),
        }
    }
}
