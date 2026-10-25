//! Admission of a narrower logical view of an existing native buffer.

use super::{Buffer, BufferByteLength};
use std::{error::Error, fmt};

pub type PrefixResult<T> = Result<T, BufferPrefixError>;

/// A requested prefix exceeds the logical parent or its physical allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferPrefixError {
    pub requested: BufferByteLength,
    pub logical: BufferByteLength,
    pub physical: BufferByteLength,
}

impl fmt::Display for BufferPrefixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU buffer prefix requests {} bytes from {} logical bytes in a {}-byte allocation",
            self.requested, self.logical, self.physical,
        )
    }
}

impl Error for BufferPrefixError {}

impl Buffer {
    /// Retain this native allocation with a checked, narrower logical extent.
    ///
    /// The prefix must fit both this buffer's current logical extent and its
    /// actual allocation. Zero is valid. This operation shares the native
    /// handle without allocating, copying, mapping, submitting or waiting.
    /// Logical bytes need not be aligned; readback owns mapping alignment,
    /// and a native storage binding still requires nonempty logical bytes.
    pub fn prefix(&self, bytes: BufferByteLength) -> PrefixResult<Self> {
        let physical = BufferByteLength::from(self.buffer.size());
        if bytes > self.size || bytes > physical {
            return Err(BufferPrefixError {
                requested: bytes,
                logical: self.size,
                physical,
            });
        }
        Ok(Self {
            buffer: self.buffer.clone(),
            size: bytes,
        })
    }
}
