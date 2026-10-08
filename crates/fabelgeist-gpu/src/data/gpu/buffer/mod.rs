use crate::globals::WgpuContext;
use anyhow::{Result, anyhow};
use std::sync::Arc;

mod definition;
mod extent;
mod label;
mod readback;
mod upload;
mod write_error;
pub use definition::{BufferDefinition, BufferUse};
pub use extent::{BufferByteLength, BufferByteOffset};
pub use label::BufferLabel;
pub use upload::{BufferUpload, BufferUploadOccupancy};
pub use write_error::{BufferWriteError, BufferWriteResult};

#[derive(Clone, Debug)]
pub struct Buffer {
    pub buffer: Arc<wgpu::Buffer>,
    pub size: BufferByteLength,
    pub usage: wgpu::BufferUsages,
}

impl Buffer {
    pub fn new(
        context: &WgpuContext,
        bytes: BufferByteLength,
        definition: BufferDefinition,
    ) -> Result<Buffer> {
        if u64::from(bytes) == 0 {
            return Err(anyhow!("Buffer size must be greater than 0"));
        }
        let buffer = definition.allocate_native(context, bytes);
        let usage = buffer.usage();
        Ok(Buffer {
            buffer: Arc::new(buffer),
            size: bytes,
            usage,
        })
    }

    pub async fn read<T: bytemuck::AnyBitPattern>(&self, context: &WgpuContext) -> Result<Vec<T>> {
        let size = u64::from(self.size);
        let readback = readback::Readback::<T>::new(size)?;
        let is_mappable = self.usage.contains(wgpu::BufferUsages::MAP_READ);

        let (target_buffer, needs_unmap) = if is_mappable {
            (self.buffer.clone(), false)
        } else {
            // 1. Create staging buffer
            let staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // 2. Copy data to staging buffer
            let mut encoder = context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging_buffer, 0, size);
            context.queue.submit(Some(encoder.finish()));
            (Arc::new(staging_buffer), true)
        };

        #[allow(unused_mut)]
        let (tx, mut rx) = futures_channel::oneshot::channel();

        // 1. Start mapping
        {
            let slice = target_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx.send(res);
            });
        }

        // 2. Poll if on native
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Waiting on the queue is what a readback actually costs. Asking
            // whether it is done and then sleeping a millisecond between asks
            // adds the sleep's granularity to every one -- and on Windows that
            // is not a millisecond, it is whatever the system timer is set to,
            // which turned a four-megabyte read into four milliseconds and
            // sometimes eight. `Wait` returns when the work is done.
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            loop {
                match rx.try_recv() {
                    Ok(Some(res)) => {
                        res.map_err(|e| anyhow!("GPU Mapping error: {:?}", e))?;
                        break;
                    }
                    // `Wait` covers submitted work, so this is reached only for
                    // a buffer that was already mappable and had nothing
                    // submitted for it. Yielding keeps that case from spinning
                    // a core without putting a sleep back in the common one.
                    Ok(None) => {
                        let _ = context.device.poll(wgpu::PollType::Poll);
                        std::thread::yield_now();
                    }
                    Err(_) => return Err(anyhow!("Mapping channel closed")),
                }
            }
        }

        // 3. Await result (only on wasm, since native loop already consumed rx)
        #[cfg(target_arch = "wasm32")]
        rx.await
            .map_err(|_| anyhow!("Mapping channel closed"))?
            .map_err(|_| anyhow!("GPU Mapping error"))?;

        // Allocate before obtaining the mapped view so WASM memory growth
        // cannot detach it; initialize typed values only after copying bytes.
        let slice = target_buffer.slice(..);
        let data = slice.get_mapped_range();
        let result = readback.copy_from(&data);
        drop(data);

        if needs_unmap {
            target_buffer.unmap();
        } else {
            // If it's the original buffer, we still need to unmap it so it can be used by the GPU again
            target_buffer.unmap();
        }

        result
    }
    pub fn write(&self, context: &WgpuContext, data: BufferUpload<'_>) -> Result<()> {
        data.write_to(context, &self.buffer, BufferByteOffset::START);
        Ok(())
    }

    /// Write one host upload at a checked byte address within this buffer.
    pub fn write_at(
        &self,
        context: &WgpuContext,
        at: BufferByteOffset,
        data: BufferUpload<'_>,
    ) -> BufferWriteResult<()> {
        let bytes = data.length();
        let end = at
            .checked_after(bytes)
            .ok_or(BufferWriteError::OffsetOverflow { at, bytes })?;
        if !end.is_end_within(self.size) {
            return Err(BufferWriteError::OutOfBounds {
                at,
                bytes,
                length: self.size,
            });
        }
        data.write_to(context, &self.buffer, at);
        Ok(())
    }

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
