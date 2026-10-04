//! Construction and dispatch failures retain constraint identity and causes.
use fabelgeist_compute::prelude::{KernelCacheError, KernelDispatchError};
use fabelgeist_gpu::prelude::BufferCreationError;

use crate::{ColorRange, ConstraintAttachmentError, ConstraintCount, ConstraintName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintBuffer {
    Particles,
    Lambdas,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintKernel {
    Projection,
    Clear,
}
#[derive(Debug)]
pub enum ConstraintBuildError {
    DistanceRecordCount {
        set: ConstraintName,
        edges: ConstraintCount,
        rest_lengths: ConstraintCount,
    },
    SpringRecordCount {
        set: ConstraintName,
        edges: ConstraintCount,
        rest_lengths: ConstraintCount,
        spring_params: ConstraintCount,
    },
    Allocation {
        set: ConstraintName,
        buffer: ConstraintBuffer,
        source: BufferCreationError,
    },
    Kernel {
        set: ConstraintName,
        kernel: ConstraintKernel,
        source: KernelCacheError,
    },
    Attachment(ConstraintAttachmentError),
}
impl std::fmt::Display for ConstraintBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DistanceRecordCount {
                edges,
                rest_lengths,
                ..
            } => write!(
                formatter,
                "ConstraintSet::distance: {edges} edges but {rest_lengths} rest lengths"
            ),
            Self::SpringRecordCount {
                edges,
                rest_lengths,
                spring_params,
                ..
            } => write!(
                formatter,
                "ConstraintSet::spring: {edges} edges, {rest_lengths} rest lengths, {spring_params} spring params"
            ),
            Self::Allocation { source, .. } => source.fmt(formatter),
            Self::Kernel { source, .. } => source.fmt(formatter),
            Self::Attachment(source) => source.fmt(formatter),
        }
    }
}
impl std::error::Error for ConstraintBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation { source, .. } => Some(source),
            Self::Kernel { source, .. } => Some(source),
            Self::Attachment(source) => Some(source),
            Self::DistanceRecordCount { .. } | Self::SpringRecordCount { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintDispatchStage {
    Clear,
    Solve(ColorRange),
}
#[derive(Debug)]
pub struct ConstraintDispatchError {
    pub set: ConstraintName,
    pub stage: ConstraintDispatchStage,
    pub source: Box<KernelDispatchError>,
}
impl std::fmt::Display for ConstraintDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}
impl std::error::Error for ConstraintDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}
