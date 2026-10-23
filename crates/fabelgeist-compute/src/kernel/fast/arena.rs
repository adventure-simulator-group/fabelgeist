use crate::prelude::*;
use fabelgeist_gpu::data::gpu::buffer::{BufferByteLength, BufferByteOffset};
use std::{error::Error, fmt};

/// Native buffer capacity for per-batch uniform slots, in bytes.
const CHUNK_BYTES: u64 = 64 * 1024;

/// A batch owns these slots until its commands have been submitted.
pub(in crate::kernel) struct UniformArena {
    chunks: Vec<wgpu::Buffer>,
    used: BufferByteOffset,
}

pub(super) struct UniformSlot {
    pub buffer: wgpu::Buffer,
    pub offset: BufferByteOffset,
}

#[derive(Debug)]
pub(super) enum UniformArenaError {
    StrideExceedsChunk,
    AddressOverflow,
    MissingChunk,
    DynamicOffsetOverflow,
}

impl fmt::Display for UniformArenaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StrideExceedsChunk => "Kernel uniform slot exceeds arena chunk capacity",
            Self::AddressOverflow => "Kernel uniform arena byte address overflow",
            Self::MissingChunk => "Kernel uniform arena has no allocated chunk",
            Self::DynamicOffsetOverflow => {
                "Kernel uniform offset exceeds native dynamic-offset range"
            }
        })
    }
}
impl Error for UniformArenaError {}

impl Default for UniformArena {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            used: BufferByteOffset::START,
        }
    }
}

impl UniformArena {
    pub(super) fn allocate(
        &mut self,
        context: &WgpuContext,
        stride: BufferByteLength,
    ) -> Result<UniformSlot> {
        if u64::from(stride) > CHUNK_BYTES {
            return Err(UniformArenaError::StrideExceedsChunk.into());
        }
        let end = self
            .used
            .checked_after(stride)
            .ok_or(UniformArenaError::AddressOverflow)?;
        if self.chunks.is_empty() || u64::from(end) > CHUNK_BYTES {
            #[cfg(not(target_arch = "wasm32"))]
            let scope = context.blocking_validation.then(|| {
                context
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation)
            });
            let chunk = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Kernel Uniform Arena"),
                size: CHUNK_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            chunk.unmap();
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(scope) = scope {
                let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
                if let Some(cause) = pollster::block_on(scope.pop()) {
                    return Err(ComputePassError::Native(std::sync::Arc::new(cause)).into());
                }
            }
            self.chunks.push(chunk);
            self.used = BufferByteOffset::START;
        }
        let slot = UniformSlot {
            buffer: self
                .chunks
                .last()
                .ok_or(UniformArenaError::MissingChunk)?
                .clone(),
            offset: self.used,
        };
        self.used = self
            .used
            .checked_after(stride)
            .ok_or(UniformArenaError::AddressOverflow)?;
        Ok(slot)
    }
}
