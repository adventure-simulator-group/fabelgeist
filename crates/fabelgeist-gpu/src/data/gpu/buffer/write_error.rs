//! Logical byte-range admission before a queue write.

use super::{BufferByteLength, BufferByteOffset};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BufferWriteError {
    #[error("Buffer write overflows an offset")]
    OffsetOverflow {
        at: BufferByteOffset,
        bytes: BufferByteLength,
    },
    #[error("Writing {bytes} bytes at {at} runs past the end of a {length}-byte buffer")]
    OutOfBounds {
        at: BufferByteOffset,
        bytes: BufferByteLength,
        length: BufferByteLength,
    },
}

/// An admitted logical write; native alignment and usage remain device policy.
pub(super) struct BufferWriteRange {
    pub at: BufferByteOffset,
}

impl BufferWriteRange {
    pub fn new(
        at: BufferByteOffset,
        bytes: BufferByteLength,
        length: BufferByteLength,
    ) -> Result<Self, BufferWriteError> {
        let Some(end) = at.checked_after(bytes) else {
            return Err(BufferWriteError::OffsetOverflow { at, bytes });
        };
        if u64::from(end) > u64::from(length) {
            return Err(BufferWriteError::OutOfBounds { at, bytes, length });
        }
        Ok(Self { at })
    }
}
