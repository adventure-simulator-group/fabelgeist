//! Sorting failures retain resource, capacity, and digit-stage context.
use super::{SortDigit, SortItemCount};
use crate::{BufferCopyError, KernelCacheError, KernelDispatchError};
use fabelgeist_gpu::prelude::{BufferByteLength, BufferCreationError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortBufferRole {
    Keys,
    Values,
    Histogram,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKernelRole {
    Histogram,
    Scan,
    Scatter,
}
#[derive(Debug)]
pub struct SortScratchError {
    pub role: SortBufferRole,
    pub capacity: SortItemCount,
    pub bytes: BufferByteLength,
    pub source: BufferCreationError,
}
impl std::fmt::Display for SortScratchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}
impl std::error::Error for SortScratchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
#[derive(Debug)]
pub struct SortBuildError {
    pub role: SortKernelRole,
    pub source: Box<KernelCacheError>,
}
impl std::fmt::Display for SortBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}
impl std::error::Error for SortBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortDispatchStage {
    pub kernel: SortKernelRole,
    pub digit: SortDigit,
}
#[derive(Debug)]
pub enum SortError {
    ScratchCapacity {
        capacity: SortItemCount,
        count: SortItemCount,
    },
    BufferLength {
        count: SortItemCount,
        needed: BufferByteLength,
        keys: BufferByteLength,
        values: BufferByteLength,
    },
    Dispatch {
        stage: SortDispatchStage,
        source: Box<KernelDispatchError>,
    },
    Copy {
        role: SortBufferRole,
        source: BufferCopyError,
    },
}
impl std::fmt::Display for SortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScratchCapacity { capacity, count } => write!(
                f,
                "RadixSort: scratch holds {capacity} elements, asked to sort {count}"
            ),
            Self::BufferLength {
                count,
                needed,
                keys,
                values,
            } => write!(
                f,
                "RadixSort: {count} elements need {needed} bytes; keys hold {keys}, values hold {values}"
            ),
            Self::Dispatch { source, .. } => source.fmt(f),
            Self::Copy { source, .. } => source.fmt(f),
        }
    }
}
impl std::error::Error for SortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch { source, .. } => Some(source.as_ref()),
            Self::Copy { source, .. } => Some(source),
            Self::ScratchCapacity { .. } | Self::BufferLength { .. } => None,
        }
    }
}
