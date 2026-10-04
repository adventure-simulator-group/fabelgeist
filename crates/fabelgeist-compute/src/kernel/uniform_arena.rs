//! Batch-owned uniform slots, aligned for the device's dynamic binding ABI.
use fabelgeist_gpu::prelude::{BufferByteLength, BufferByteOffset, WgpuContext};

const CHUNK_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy)]
pub(super) struct UniformStride(BufferByteLength);
impl UniformStride {
    pub fn new(context: &WgpuContext, logical: BufferByteLength) -> Self {
        let alignment =
            u64::from(context.device.limits().min_uniform_buffer_offset_alignment).max(1);
        Self(BufferByteLength::from(
            u64::from(logical).div_ceil(alignment) * alignment,
        ))
    }
}

#[derive(Clone, Copy)]
pub(super) struct UniformDynamicOffset(u32);
impl From<UniformDynamicOffset> for u32 {
    fn from(offset: UniformDynamicOffset) -> Self {
        offset.0
    }
}

pub(super) struct UniformSlot {
    chunk: wgpu::Buffer,
    offset: BufferByteOffset,
}
impl UniformSlot {
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.chunk
    }
    pub fn offset(&self) -> BufferByteOffset {
        self.offset
    }
    pub fn dynamic_offset(&self) -> UniformDynamicOffset {
        // Allocation starts a new chunk before an address exceeds CHUNK_BYTES.
        UniformDynamicOffset(
            u32::try_from(u64::from(self.offset)).expect("uniform slot is within a 64 KiB chunk"),
        )
    }
}

/// The uniform values of one batch's dispatches.
///
/// A kernel used to own a ring of uniform slots that every batch wrote into
/// in turn. A ring is only safe to wrap once the batch holding a slot has been
/// submitted, and with several threads recording at once nothing guarantees
/// that: a long batch still being recorded on one thread saw its slots
/// overwritten by other threads' dispatches, and ran with their counts. So
/// each batch now writes its uniforms into buffers of its own, which live
/// exactly as long as its commands need them.
pub(super) struct UniformArena {
    chunks: Vec<wgpu::Buffer>,
    used: BufferByteOffset,
}
impl Default for UniformArena {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            used: BufferByteOffset::START,
        }
    }
}
impl UniformArena {
    pub fn allocate(&mut self, context: &WgpuContext, stride: UniformStride) -> UniformSlot {
        let end = self
            .used
            .after(stride.0)
            .expect("bounded uniform slot and device alignment fit a byte address");
        if self.chunks.is_empty() || u64::from(end) > CHUNK_BYTES {
            let chunk = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Kernel Uniform Arena"),
                size: CHUNK_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            chunk.unmap();
            self.chunks.push(chunk);
            self.used = BufferByteOffset::START;
        }
        let offset = self.used;
        self.used = self
            .used
            .after(stride.0)
            .expect("bounded uniform slot and device alignment fit a byte address");
        UniformSlot {
            chunk: self.chunks.last().expect("allocated above").clone(),
            offset,
        }
    }
}
