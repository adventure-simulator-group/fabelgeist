use super::mapping::{MappingExtent, MappingRequest};
use super::readback::Readback;
use super::{Buffer, BufferByteLength, BufferReadError, ReadResult};
use crate::globals::WgpuContext;
use anyhow::Result;
use std::sync::Arc;

impl Buffer {
    /// Read the logical byte prefix as complete, nonzero-sized host elements.
    ///
    /// The context must contain the exact creating device and its paired
    /// queue. External aliases must leave the source unmapped and grant this
    /// operation exclusive mapping access. Handle the producing submission's
    /// result before reading; successful readback does not certify later GPU
    /// writes or global device health.
    ///
    /// Native progress waits explicitly; browser callbacks use the event loop.
    /// Sources without MAP_READ use a staging copy, subject to native copy
    /// capabilities and alignment. Logical zero returns an empty vector after
    /// host-layout admission. Physical allocation padding is never returned.
    pub async fn read<T: bytemuck::AnyBitPattern>(&self, context: &WgpuContext) -> Result<Vec<T>> {
        let size = u64::from(self.size);
        let allocated = self.buffer.size();
        if size > allocated {
            return Err(BufferReadError::ExtentExceedsAllocation {
                requested: self.size,
                available: BufferByteLength::from(allocated),
            }
            .into());
        }
        let extent = MappingExtent::for_prefix(self.size, BufferByteLength::from(allocated))?;
        let readback = Readback::<T>::new(self.size)?;
        let Some(extent) = extent else {
            return Ok(readback.copy_from(&[])?);
        };
        let target = if self.buffer.usage().contains(wgpu::BufferUsages::MAP_READ) {
            self.buffer.clone()
        } else {
            self.readback_target(context, extent)?
        };
        let mut mapping = MappingRequest::request(context, target, extent)?;
        mapping.complete(context).await?;

        // The typed destination was allocated before obtaining this view:
        // WASM memory growth must not detach it during the copy.
        let data = mapping.mapped_range();
        let values = readback.copy_from(&data);
        drop(data);
        let values = values?;
        mapping.release(context)?;
        Ok(values)
    }

    fn readback_target(
        &self,
        context: &WgpuContext,
        extent: MappingExtent,
    ) -> ReadResult<Arc<wgpu::Buffer>> {
        let target = context
            .observe_validation(|| {
                context.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Staging Buffer"),
                    size: u64::from(extent.bytes()),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .map_err(BufferReadError::Native)?;
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &target, 0, u64::from(extent.bytes()));
        context.submit(encoder).map_err(BufferReadError::Native)?;
        Ok(Arc::new(target))
    }
}
