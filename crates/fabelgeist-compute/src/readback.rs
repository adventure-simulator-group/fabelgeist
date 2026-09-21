//! Many results back to the host for the price of one.
//!
//! [`Buffer::read`] copies its buffer to a staging buffer, submits, and
//! waits for the queue: one round trip per buffer. A pipeline that ends with a
//! dozen small results -- positions, normals, indices, a few status words --
//! pays that wait a dozen times. A [`Readback`] records every copy into the
//! batch that produced the results, lays them end to end in one staging
//! buffer, and maps it once after the batch is submitted.

use crate::prelude::*;

/// Copy offsets must be multiples of this many bytes.
const ALIGNMENT: u64 = wgpu::COPY_BUFFER_ALIGNMENT;

/// Results staged for one mapping.
#[derive(Debug)]
pub struct Readback {
    staging: wgpu::Buffer,
    /// Byte offset and length of each staged buffer.
    ranges: Vec<(u64, u64)>,
}

impl Readback {
    /// Record copies of `buffers` into a new staging buffer, in `batch` after
    /// everything already recorded there.
    pub fn record(context: &WgpuContext, batch: &mut KernelBatch, buffers: &[&Buffer]) -> Self {
        let mut ranges = Vec::with_capacity(buffers.len());
        let mut size = 0;
        for buffer in buffers {
            ranges.push((size, buffer.size));
            size += buffer.size.div_ceil(ALIGNMENT) * ALIGNMENT;
        }
        let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: size.max(ALIGNMENT),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        for (buffer, (offset, length)) in buffers.iter().zip(&ranges) {
            batch
                .encoder()
                .copy_buffer_to_buffer(&buffer.buffer, 0, &staging, *offset, *length);
        }
        Self { staging, ranges }
    }

    /// Map the staging buffer once the batch has been submitted, and return
    /// each buffer's bytes in the order they were recorded. On native targets
    /// this waits on the device.
    pub async fn read(self, context: &WgpuContext) -> Result<Vec<Vec<u8>>> {
        let slice = self.staging.slice(..);
        let (sender, receiver) = futures_channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        #[cfg(not(target_arch = "wasm32"))]
        let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
        #[cfg(target_arch = "wasm32")]
        let _ = context;
        receiver
            .await
            .map_err(|_| anyhow!("Readback: mapping channel closed"))?
            .map_err(|error| anyhow!("Readback: mapping failed: {error:?}"))?;
        let data = slice
            .get_mapped_range()
            .map_err(|error| anyhow!("Readback: mapped range unavailable: {error:?}"))?;
        let results = self
            .ranges
            .iter()
            .map(|(offset, length)| data[*offset as usize..(*offset + *length) as usize].to_vec())
            .collect();
        drop(data);
        self.staging.unmap();
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn every_buffer_comes_back_in_order() -> Result<()> {
        let context = WgpuContext::new().await?;
        let definition = BufferDefinition::storage();
        // Odd lengths, so that the staged copies need padding between them.
        let a = Buffer::from_slice(&context, &[1u32, 2, 3], definition.clone())?;
        let b = Buffer::from_slice(
            &context,
            &[7u8, 8, 9, 10, 11, 12, 13, 14],
            definition.clone(),
        )?;
        let c = Buffer::from_slice(&context, &[0.5f32], definition)?;
        let mut batch = KernelBatch::new(&context);
        let readback = Readback::record(&context, &mut batch, &[&a, &b, &c]);
        batch.submit();
        let results = readback.read(&context).await?;
        assert_eq!(bytemuck::cast_slice::<u8, u32>(&results[0]), &[1, 2, 3]);
        assert_eq!(results[1], [7, 8, 9, 10, 11, 12, 13, 14]);
        assert_eq!(bytemuck::cast_slice::<u8, f32>(&results[2]), &[0.5]);
        Ok(())
    }
}
