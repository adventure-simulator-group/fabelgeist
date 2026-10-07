//! Many results back to the host for the price of one.
//!
//! [`Buffer::read`] copies its buffer to a staging buffer, submits, and
//! waits for the queue: one round trip per buffer. A pipeline that ends with a
//! dozen small results -- positions, normals, indices, a few status words --
//! pays that wait a dozen times. A [`Readback`] records every copy into the
//! batch that produced the results, lays them end to end in one staging
//! buffer, and maps it once after the batch is submitted.
//!
//! Waiting at all is the other cost. [`Readback::record_pending`] asks for the
//! mapping as the batch is submitted and returns a [`PendingReadback`], which
//! hands the results over once the GPU has got there -- a frame later, say --
//! and until then says so rather than waiting: a simulation stepped on the
//! GPU can be read every frame without the CPU ever stalling on it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::prelude::*;
#[cfg(test)]
use fabelgeist_gpu::prelude::BufferUpload;

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
            ranges.push((size, u64::from(buffer.size)));
            size += u64::from(buffer.size).div_ceil(ALIGNMENT) * ALIGNMENT;
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

    /// Record as [`Readback::record`] does, and ask for the mapping as soon
    /// as `batch` is submitted, without waiting for it.
    pub fn record_pending(
        context: &WgpuContext,
        batch: &mut KernelBatch,
        buffers: &[&Buffer],
    ) -> PendingReadback {
        let Self { staging, ranges } = Self::record(context, batch, buffers);
        let state = Arc::new(AtomicU8::new(PENDING));
        let staging = Arc::new(staging);
        let (mapped, flag) = (staging.clone(), state.clone());
        batch.after_submit(move || {
            mapped
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    flag.store(
                        if result.is_ok() { READY } else { FAILED },
                        Ordering::Release,
                    );
                });
        });
        PendingReadback {
            staging,
            ranges,
            state,
        }
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
        let data = slice.get_mapped_range();
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

const PENDING: u8 = 0;
const READY: u8 = 1;
const FAILED: u8 = 2;
const TAKEN: u8 = 3;

/// Results on their way back: see [`Readback::record_pending`].
#[derive(Debug)]
pub struct PendingReadback {
    staging: Arc<wgpu::Buffer>,
    ranges: Vec<(u64, u64)>,
    state: Arc<AtomicU8>,
}

impl PendingReadback {
    /// Each buffer's bytes, in the order they were recorded, if the GPU has
    /// finished with them; `None` if not yet. Never waits: on native targets
    /// it only gives the device a chance to report what has finished.
    pub fn try_read(&self, context: &WgpuContext) -> Result<Option<Vec<Vec<u8>>>> {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = context.device.poll(wgpu::PollType::Poll);
        #[cfg(target_arch = "wasm32")]
        let _ = context;
        match self.state.load(Ordering::Acquire) {
            PENDING => return Ok(None),
            FAILED => return Err(anyhow!("Readback: mapping failed")),
            TAKEN => return Err(anyhow!("Readback: already read")),
            _ => {}
        }
        let slice = self.staging.slice(..);
        let data = slice.get_mapped_range();
        let results = self
            .ranges
            .iter()
            .map(|(offset, length)| data[*offset as usize..(*offset + *length) as usize].to_vec())
            .collect();
        drop(data);
        self.staging.unmap();
        // Read once: the staging buffer is no longer mapped.
        self.state.store(TAKEN, Ordering::Release);
        Ok(Some(results))
    }

    /// Whether the results are back, without taking them.
    pub fn is_ready(&self, context: &WgpuContext) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = context.device.poll(wgpu::PollType::Poll);
        #[cfg(target_arch = "wasm32")]
        let _ = context;
        self.state.load(Ordering::Acquire) == READY
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
        let a = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[1u32, 2, 3]),
            definition.clone(),
        )?;
        let b = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[7u8, 8, 9, 10, 11, 12, 13, 14]),
            definition.clone(),
        )?;
        let c = Buffer::from_upload(&context, BufferUpload::from_elements(&[0.5f32]), definition)?;
        let mut batch = KernelBatch::new(&context);
        let readback = Readback::record(&context, &mut batch, &[&a, &b, &c]);
        batch.submit();
        let results = readback.read(&context).await?;
        assert_eq!(bytemuck::cast_slice::<u8, u32>(&results[0]), &[1, 2, 3]);
        assert_eq!(results[1], [7, 8, 9, 10, 11, 12, 13, 14]);
        assert_eq!(bytemuck::cast_slice::<u8, f32>(&results[2]), &[0.5]);
        Ok(())
    }

    #[tokio::test]
    async fn a_pending_readback_arrives_without_waiting() -> Result<()> {
        let context = WgpuContext::new().await?;
        let definition = BufferDefinition::storage();
        let a = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[4u32, 5, 6]),
            definition.clone(),
        )?;
        let b = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0.25f32, 0.75]),
            definition,
        )?;
        let mut batch = KernelBatch::new(&context);
        let pending = Readback::record_pending(&context, &mut batch, &[&a, &b]);
        // Nothing has been submitted, so nothing can have come back.
        assert!(pending.try_read(&context)?.is_none());
        batch.submit();
        let started = std::time::Instant::now();
        let results = loop {
            if let Some(results) = pending.try_read(&context)? {
                break results;
            }
            assert!(
                started.elapsed().as_secs() < 10,
                "the readback never arrived"
            );
            std::thread::yield_now();
        };
        assert_eq!(bytemuck::cast_slice::<u8, u32>(&results[0]), &[4, 5, 6]);
        assert_eq!(bytemuck::cast_slice::<u8, f32>(&results[1]), &[0.25, 0.75]);
        assert!(
            pending.try_read(&context).is_err(),
            "a readback is read once"
        );
        Ok(())
    }
}
