use fabelgeist_gpu::data::gpu::buffer::BufferByteLength;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum KernelBatchError {
    CopyOutOfBounds {
        requested: BufferByteLength,
        source: BufferByteLength,
        destination: BufferByteLength,
    },
}

impl fmt::Display for KernelBatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CopyOutOfBounds {
                requested,
                source,
                destination,
            } => write!(
                formatter,
                "KernelBatch::copy_buffer: {requested} bytes does not fit {source} -> {destination}"
            ),
        }
    }
}
impl Error for KernelBatchError {}
