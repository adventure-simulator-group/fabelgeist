use crate::globals::WgpuContext;
use anyhow::Result;
use std::sync::Arc;

pub use admission_error::BufferAdmissionError;
pub use binding::BufferBindingError;

pub use definition::{BufferDefinition, BufferUse};
pub use extent::{BufferByteLength, BufferByteOffset};
pub use label::BufferLabel;
pub(in crate::data::gpu) use mapping::{MappingExtent, MappingRequest};
pub use prefix::{BufferPrefixError, PrefixResult};
pub use read_error::BufferReadError;
use read_error::ReadResult;
pub(in crate::data::gpu) use readback::Readback;
pub use upload::{BufferUpload, BufferUploadOccupancy};

mod admission_error;
mod binding;
mod definition;
mod extent;
mod label;
mod mapping;
mod prefix;
mod read;
mod read_error;
mod readback;
mod upload;

#[derive(Clone, Debug)]
pub struct Buffer {
    pub buffer: Arc<wgpu::Buffer>,
    pub size: BufferByteLength,
}

impl Buffer {
    /// Allocate on `context.device`; the returned buffer belongs to that device.
    pub fn new(
        context: &WgpuContext,
        bytes: BufferByteLength,
        definition: BufferDefinition,
    ) -> Result<Buffer> {
        if u64::from(bytes) == 0 {
            return Err(BufferAdmissionError::EmptyAllocation.into());
        }
        let buffer = definition.allocate_native(context, bytes)?;
        Ok(Buffer {
            buffer: Arc::new(buffer),
            size: bytes,
        })
    }

    /// Write within this logical extent through the creating device's paired queue.
    ///
    /// This buffer must belong to `context.device`, and `context.queue` must
    /// be paired with that same logical device.
    pub fn write(&self, context: &WgpuContext, data: BufferUpload<'_>) -> Result<()> {
        self.write_at(context, BufferByteOffset::START, data)
    }

    /// Write one host upload at a checked byte address within this buffer.
    ///
    /// This buffer must belong to `context.device`, and `context.queue` must
    /// be paired with that same logical device.
    pub fn write_at(
        &self,
        context: &WgpuContext,
        at: BufferByteOffset,
        data: BufferUpload<'_>,
    ) -> Result<()> {
        let end =
            at.checked_after(data.length())
                .ok_or(BufferAdmissionError::WriteOffsetOverflow {
                    at,
                    bytes: data.length(),
                })?;
        if u64::from(end) > u64::from(self.size) {
            return Err(BufferAdmissionError::WriteOutOfBounds {
                at,
                bytes: data.length(),
                capacity: self.size,
            }
            .into());
        }
        data.write_to(context, &self.buffer, at)?;
        Ok(())
    }

    /// Allocate and upload using a device and its paired queue.
    ///
    /// `context.queue` must be paired with `context.device`.
    pub fn from_upload(
        context: &WgpuContext,
        data: BufferUpload<'_>,
        definition: BufferDefinition,
    ) -> Result<Buffer> {
        let buffer = Self::new(
            context,
            data.length(),
            definition.with_usage(BufferUse::CopyDestination),
        )?;
        buffer.write(context, data)?;
        Ok(buffer)
    }
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.buffer, &other.buffer) && self.size == other.size
    }
}

#[cfg(test)]
mod tests;
