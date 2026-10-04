use crate::globals::WgpuContext;
use std::sync::Arc;

mod creation_error;
mod definition;
mod extent;
mod label;
mod upload;
pub use creation_error::BufferCreationError;
mod length_error;
mod mapping;
mod read_error;
mod readback;
mod results;
#[cfg(test)]
mod tests;
mod write_error;

pub use definition::{BufferDefinition, BufferUse};
pub use extent::{BufferByteLength, BufferByteOffset, ReadbackRange};
pub use label::BufferLabel;
pub use length_error::BufferLengthError;
pub use mapping::{ReadbackMapping, ReadbackView};
pub use read_error::ReadbackError;
pub use readback::{BufferReadback, MappedBufferBytes};
pub use results::{ReadbackBlocks, ReadbackSlot, ReadbackSources, ReadbackStatusWord};
pub use upload::{BufferUpload, BufferUploadOccupancy};
pub use write_error::BufferWriteError;

/// Native storage with a separately retained logical byte result.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::*;
/// fn allocate_at_address(context: &WgpuContext, address: BufferByteOffset) {
///     let _ = Buffer::new(context, address, BufferDefinition::storage());
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Buffer {
    pub buffer: Arc<wgpu::Buffer>,
    length: BufferByteLength,
}

impl Buffer {
    pub fn new(
        context: &WgpuContext,
        bytes: BufferByteLength,
        definition: BufferDefinition,
    ) -> std::result::Result<Buffer, BufferCreationError> {
        if bytes == BufferByteLength::default() {
            return Err(BufferCreationError::Empty);
        }

        let buffer = definition.allocate_native(context, bytes);

        Ok(Buffer {
            buffer: Arc::new(buffer),
            length: bytes,
        })
    }

    /// Logical bytes, excluding any retained allocation padding.
    pub fn length(&self) -> BufferByteLength {
        self.length
    }

    /// Select a logical result length while retaining the native allocation.
    pub fn with_logical_length(
        mut self,
        requested: BufferByteLength,
    ) -> Result<Self, BufferLengthError> {
        let available = BufferByteLength::from(self.buffer.size());
        if requested > available {
            return Err(BufferLengthError {
                requested,
                available,
            });
        }
        self.length = requested;
        Ok(self)
    }

    /// Convert a GPU element representation into initialized host values.
    pub async fn read<T: bytemuck::AnyBitPattern>(
        &self,
        context: &WgpuContext,
    ) -> std::result::Result<Vec<T>, ReadbackError> {
        let length = self.length;
        let readback = BufferReadback::<T>::new(length)?;
        if length == BufferByteLength::from(0u64) {
            return readback.copy_from(MappedBufferBytes::from(&[][..]));
        }
        let target_buffer = if self.buffer.usage().contains(wgpu::BufferUsages::MAP_READ) {
            self.buffer.clone()
        } else {
            let extent = length.copy_aligned()?;
            let available = BufferByteLength::from(self.buffer.size());
            if u64::from(extent) > u64::from(available) {
                return Err(ReadbackError::CopyStorage {
                    required: extent,
                    available,
                });
            }
            let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: u64::from(extent),
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder = context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging, 0, u64::from(extent));
            context.queue.submit(Some(encoder.finish()));
            Arc::new(staging)
        };
        let mapping = ReadbackMapping::new(context, &target_buffer).await?;
        let view = mapping.view()?;
        readback.copy_from(view.bytes())
    }
    pub fn write(&self, context: &WgpuContext, data: BufferUpload<'_>) {
        data.write_to(context, &self.buffer, BufferByteOffset::START);
    }

    /// Write `data` into this buffer starting `at` bytes in.
    ///
    /// For a buffer that holds several things and has only one of them
    /// changing: a ring of video frames, a slice of a vertex stream. Writing
    /// the whole buffer to replace part of it is the alternative, and at these
    /// sizes that is the cost.
    pub fn write_at(
        &self,
        context: &WgpuContext,
        at: BufferByteOffset,
        data: BufferUpload<'_>,
    ) -> Result<(), BufferWriteError> {
        let range = write_error::BufferWriteRange::new(at, data.length(), self.length)?;
        data.write_to(context, &self.buffer, range.at);
        Ok(())
    }
    pub fn from_upload(
        context: &WgpuContext,
        data: BufferUpload<'_>,
        definition: BufferDefinition,
    ) -> Result<Buffer, BufferCreationError> {
        let buffer = Self::new(
            context,
            data.length(),
            definition.with_usage(BufferUse::CopyDestination),
        )?;
        data.write_to(context, &buffer.buffer, BufferByteOffset::START);
        Ok(buffer)
    }

    /// Allocate backing storage for an empty logical result.
    pub fn empty_result(
        context: &WgpuContext,
        allocation: BufferByteLength,
        definition: BufferDefinition,
    ) -> Result<Buffer, BufferCreationError> {
        let mut buffer = Self::new(context, allocation, definition)?;
        buffer.length = BufferByteLength::default();
        Ok(buffer)
    }
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.buffer, &other.buffer) && self.length == other.length
    }
}
