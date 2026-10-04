//! A batch copy request that exceeds either logical buffer extent.
use fabelgeist_gpu::prelude::BufferByteLength;

#[derive(Debug)]
pub struct BufferCopyError {
    pub bytes: BufferByteLength,
    pub source_length: BufferByteLength,
    pub destination_length: BufferByteLength,
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
