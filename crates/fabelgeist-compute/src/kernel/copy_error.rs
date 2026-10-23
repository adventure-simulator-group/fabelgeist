//! Logical buffer-extent rejection while recording a batch copy.

use fabelgeist_gpu::prelude::BufferByteLength;

/// A requested copy does not fit its source or destination logical extent.
///
/// The extents are the buffers' public metadata at admission. Native allocation,
/// alignment, usage and overlap validation are separate SDK responsibilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferCopyError {
    pub bytes: BufferByteLength,
    pub source_length: BufferByteLength,
    pub destination_length: BufferByteLength,
}

/// The result of logical extent admission for a batch copy.
pub type BufferCopyResult<T> = std::result::Result<T, BufferCopyError>;

impl BufferCopyError {
    pub(super) fn new(
        bytes: BufferByteLength,
        source_length: BufferByteLength,
        destination_length: BufferByteLength,
    ) -> Self {
        Self {
            bytes,
            source_length,
            destination_length,
        }
    }
}

impl std::fmt::Display for BufferCopyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "KernelBatch::copy_buffer: {} bytes does not fit {} -> {}",
            self.bytes, self.source_length, self.destination_length
        )
    }
}

impl std::error::Error for BufferCopyError {}
