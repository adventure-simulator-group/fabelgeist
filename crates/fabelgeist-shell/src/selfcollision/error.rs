//! Self-collision failures retain resource, particle capacity and operation roles.
use super::{SelfCollisionBuffer, SelfCollisionKernel};
use fabelgeist_compute::{
    KernelCacheError, KernelDispatchError, SortBuildError, SortError, SortScratchError,
};
use fabelgeist_gpu::prelude::BufferCreationError;
use fabelgeist_xpbd::{ParticleCapacity, ParticleCount, ParticleInputCount};

#[derive(Debug)]
pub enum SelfCollisionBuildError {
    Adjacency {
        particles: ParticleCount,
        lists: ParticleInputCount,
    },
    Kernel {
        kernel: SelfCollisionKernel,
        source: Box<KernelCacheError>,
    },
    Sort(SortBuildError),
    Allocation {
        buffer: SelfCollisionBuffer,
        source: BufferCreationError,
    },
    Scratch(SortScratchError),
}
impl std::fmt::Display for SelfCollisionBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Adjacency { particles, lists } => write!(
                formatter,
                "SelfCollision: {particles} particles but {lists} adjacency lists"
            ),
            Self::Kernel { source, .. } => source.fmt(formatter),
            Self::Sort(source) => source.fmt(formatter),
            Self::Allocation { source, .. } => source.fmt(formatter),
            Self::Scratch(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for SelfCollisionBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Adjacency { .. } => None,
            Self::Kernel { source, .. } => Some(source.as_ref()),
            Self::Sort(source) => Some(source),
            Self::Allocation { source, .. } => Some(source),
            Self::Scratch(source) => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum SelfCollisionRecordError {
    Capacity {
        capacity: ParticleCapacity,
        count: ParticleCount,
    },
    Dispatch {
        kernel: SelfCollisionKernel,
        source: Box<KernelDispatchError>,
    },
    Sort(SortError),
}
impl std::fmt::Display for SelfCollisionRecordError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capacity { capacity, count } => write!(
                formatter,
                "SelfCollision: built for {capacity} particles, given {count}"
            ),
            Self::Dispatch { source, .. } => source.fmt(formatter),
            Self::Sort(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for SelfCollisionRecordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Capacity { .. } => None,
            Self::Dispatch { source, .. } => Some(source.as_ref()),
            Self::Sort(source) => Some(source),
        }
    }
}
