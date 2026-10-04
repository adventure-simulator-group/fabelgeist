//! Recoverable failures of buffer readback admission, mapping, and copying.

use super::{BufferByteLength, BufferByteOffset, ReadbackSlot};

#[derive(Debug, thiserror::Error)]
pub enum ReadbackError {
    #[error("GPU readback byte length {length} exceeds the host address space")]
    HostLength {
        length: BufferByteLength,
        #[source]
        source: std::num::TryFromIntError,
    },
    #[error("GPU readback offset {offset} exceeds the host address space")]
    HostOffset {
        offset: BufferByteOffset,
        #[source]
        source: std::num::TryFromIntError,
    },
    #[error("GPU readback requires nonzero-sized elements")]
    ZeroSizedElement,
    #[error("GPU readback length {length} is not a whole number of {element_bytes}-byte elements")]
    PartialElement {
        length: BufferByteLength,
        element_bytes: BufferByteLength,
    },
    #[error("cannot allocate {length} bytes for GPU readback")]
    Allocation {
        length: BufferByteLength,
        #[source]
        source: std::collections::TryReserveError,
    },
    #[error("GPU readback is truncated: expected {expected} bytes, found {actual}")]
    Truncated {
        expected: BufferByteLength,
        actual: BufferByteLength,
    },
    #[error("GPU copy alignment overflows byte length {0}")]
    CopyAlignmentOverflow(BufferByteLength),
    #[error("GPU readback range at {start} overflows with {length} bytes")]
    RangeOverflow {
        start: BufferByteOffset,
        length: BufferByteLength,
    },
    #[error("GPU readback needs {required} physical bytes, but the source has {available}")]
    CopyStorage {
        required: BufferByteLength,
        available: BufferByteLength,
    },
    #[error("GPU readback slot {0:?} is absent")]
    MissingSlot(ReadbackSlot),
    #[error("GPU status readback contains no word")]
    EmptyStatus,
    #[error("GPU readback device polling failed: {0}")]
    Poll(#[source] wgpu::PollError),
    #[error("GPU readback mapping failed: {0}")]
    Mapping(#[source] wgpu::BufferAsyncError),
    #[error("GPU readback mapping channel closed: {0}")]
    MappingCanceled(#[source] futures_channel::oneshot::Canceled),
    #[error("GPU readback mapped range is unavailable: {0}")]
    MappedRange(#[source] wgpu::MapRangeError),
}
