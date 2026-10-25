//! Native spacing of consecutive storage-array elements.

use super::BufferByteLength;
use std::num::NonZeroU32;

/// Byte spacing of records in a native storage array, including padding.
///
/// This is independent of an element's payload size and the array's byte
/// extent. Admit the native array stride from the shader's checked layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferElementStride(NonZeroU32);

/// A storage-array stride cannot be zero.
#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("native storage-array stride must be nonzero")]
pub struct BufferElementStrideError;

impl TryFrom<u32> for BufferElementStride {
    type Error = BufferElementStrideError;

    fn try_from(bytes: u32) -> Result<Self, Self::Error> {
        NonZeroU32::new(bytes).map(Self).ok_or(BufferElementStrideError)
    }
}

impl BufferElementStride {
    pub fn byte_length(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0.get()))
    }
}
