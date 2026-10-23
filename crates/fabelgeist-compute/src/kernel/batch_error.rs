use super::BufferCopy;
use fabelgeist_gpu::data::gpu::buffer::BufferByteLength;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum KernelBatchError {
    CopyOutOfBounds {
        copy: BufferCopy,
        source: BufferByteLength,
        destination: BufferByteLength,
    },
}

impl fmt::Display for KernelBatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CopyOutOfBounds {
                copy,
                source,
                destination,
            } => write!(
                formatter,
                "KernelBatch::copy_buffer: {} bytes at {} -> {} exceeds {source} -> {destination} bytes",
                copy.bytes, copy.source_offset, copy.destination_offset
            ),
        }
    }
}
impl Error for KernelBatchError {}
