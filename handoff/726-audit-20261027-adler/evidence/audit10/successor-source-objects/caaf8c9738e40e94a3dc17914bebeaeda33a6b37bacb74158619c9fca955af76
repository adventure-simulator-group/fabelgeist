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

use std::sync::{Arc, Mutex, MutexGuard};

use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferByteOffset, WgpuContext};

use crate::KernelBatch;

type Result<T> = std::result::Result<T, ReadbackError>;

/// A buffer's admitted location in the shared staging allocation.
#[derive(Clone, Copy, Debug)]
struct StagedRange {
    offset: BufferByteOffset,
    length: BufferByteLength,
}

/// Results staged for one mapping.
#[derive(Debug)]
pub struct Readback {
    staging: wgpu::Buffer,
    ranges: Vec<StagedRange>,
}

/// Results on their way back: see [`Readback::record_pending`].
#[derive(Debug)]
pub struct PendingReadback {
    staging: Arc<wgpu::Buffer>,
    ranges: Vec<StagedRange>,
    state: Arc<Mutex<ReadbackState>>,
}

/// Failures of recording or retrieving a batch of GPU results.
#[derive(Debug, thiserror::Error)]
pub enum ReadbackError {
    #[error("readback staging extent exceeds the addressable buffer size")]
    ExtentOverflow,
    #[error("readback mapping callback was dropped")]
    ChannelClosed,
    #[error("readback mapping failed")]
    Mapping(#[source] wgpu::BufferAsyncError),
    #[error("readback device polling failed")]
    Poll(#[source] wgpu::PollError),
    #[error("readback results have already been taken")]
    AlreadyRead,
}

#[derive(Debug)]
enum ReadbackState {
    Pending,
    Ready,
    Failed(wgpu::BufferAsyncError),
    Taken,
}

impl StagedRange {
    /// Convert byte addresses only when slicing the mapped native bytes.
    fn bytes(self, data: &[u8]) -> Vec<u8> {
        let start = u64::from(self.offset) as usize;
        let length = u64::from(self.length) as usize;
        data[start..start + length].to_vec()
    }
}

impl Readback {
    /// Record each source's typed extent in an aligned staging allocation.
    /// Overflow is rejected before a GPU allocation or copy is recorded.
    pub fn record(
        context: &WgpuContext,
        batch: &mut KernelBatch,
        buffers: &[&Buffer],
    ) -> Result<Self> {
        let mut ranges = Vec::with_capacity(buffers.len());
        let mut end = BufferByteOffset::START;
        for buffer in buffers {
            ranges.push(StagedRange {
                offset: end,
                length: buffer.size,
            });
            // wgpu's copy alignment is a byte-layout boundary. Preserve the
            // original padding and ordering while checking the total extent.
            let padded = u64::from(buffer.size)
                .div_ceil(wgpu::COPY_BUFFER_ALIGNMENT)
                .checked_mul(wgpu::COPY_BUFFER_ALIGNMENT)
                .ok_or(ReadbackError::ExtentOverflow)?;
            end = end
                .checked_after(BufferByteLength::from(padded))
                .ok_or(ReadbackError::ExtentOverflow)?;
        }
        let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: u64::from(end).max(wgpu::COPY_BUFFER_ALIGNMENT),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        for (buffer, range) in buffers.iter().zip(&ranges) {
            batch.encoder().copy_buffer_to_buffer(
                &buffer.buffer,
                0,
                &staging,
                u64::from(range.offset),
                u64::from(range.length),
            );
        }
        Ok(Self { staging, ranges })
    }

    /// Record copies and request mapping when the producing batch is submitted.
    pub fn record_pending(
        context: &WgpuContext,
        batch: &mut KernelBatch,
        buffers: &[&Buffer],
    ) -> Result<PendingReadback> {
        let Self { staging, ranges } = Self::record(context, batch, buffers)?;
        let state = Arc::new(Mutex::new(ReadbackState::Pending));
        let staging = Arc::new(staging);
        let (mapped, shared) = (staging.clone(), state.clone());
        batch.after_submit(move || {
            mapped
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    *lock_state(&shared) = match result {
                        Ok(()) => ReadbackState::Ready,
                        Err(error) => ReadbackState::Failed(error),
                    };
                });
        });
        Ok(PendingReadback {
            staging,
            ranges,
            state,
        })
    }

    /// Map once and return each source's bytes in the recorded order.
    pub async fn read(self, context: &WgpuContext) -> Result<Vec<Vec<u8>>> {
        let slice = self.staging.slice(..);
        let (sender, receiver) = futures_channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        #[cfg(not(target_arch = "wasm32"))]
        context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(ReadbackError::Poll)?;
        #[cfg(target_arch = "wasm32")]
        let _ = context;
        receiver
            .await
            .map_err(|_| ReadbackError::ChannelClosed)?
            .map_err(ReadbackError::Mapping)?;
        let data = slice.get_mapped_range();
        let results = self.ranges.iter().map(|range| range.bytes(&data)).collect();
        drop(data);
        self.staging.unmap();
        Ok(results)
    }
}

impl PendingReadback {
    /// Take completed results once, returning `None` while mapping is pending.
    pub fn try_read(&self, context: &WgpuContext) -> Result<Option<Vec<Vec<u8>>>> {
        poll(context)?;
        let mut state = lock_state(&self.state);
        match &*state {
            ReadbackState::Pending => return Ok(None),
            ReadbackState::Failed(error) => return Err(ReadbackError::Mapping(error.clone())),
            ReadbackState::Taken => return Err(ReadbackError::AlreadyRead),
            ReadbackState::Ready => {}
        }
        let slice = self.staging.slice(..);
        let data = slice.get_mapped_range();
        let results = self.ranges.iter().map(|range| range.bytes(&data)).collect();
        drop(data);
        self.staging.unmap();
        *state = ReadbackState::Taken;
        Ok(Some(results))
    }

    /// Observe readiness without taking the mapped results.
    pub fn is_ready(&self, context: &WgpuContext) -> Result<bool> {
        poll(context)?;
        match &*lock_state(&self.state) {
            ReadbackState::Ready => Ok(true),
            ReadbackState::Failed(error) => Err(ReadbackError::Mapping(error.clone())),
            ReadbackState::Pending | ReadbackState::Taken => Ok(false),
        }
    }
}

fn poll(context: &WgpuContext) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    context
        .device
        .poll(wgpu::PollType::Poll)
        .map_err(ReadbackError::Poll)?;
    #[cfg(target_arch = "wasm32")]
    let _ = context;
    Ok(())
}

fn lock_state(state: &Mutex<ReadbackState>) -> MutexGuard<'_, ReadbackState> {
    // The callback replaces one complete enum value. A poisoned lock cannot
    // expose a partially updated record, so retain the recorded GPU outcome.
    match state.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_gpu::prelude::{BufferDefinition, BufferUpload};

    #[tokio::test]
    async fn every_buffer_comes_back_in_order() -> anyhow::Result<()> {
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
        let readback = Readback::record(&context, &mut batch, &[&a, &b, &c])?;
        batch.submit();
        let results = readback.read(&context).await?;
        assert_eq!(bytemuck::cast_slice::<u8, u32>(&results[0]), &[1, 2, 3]);
        assert_eq!(results[1], [7, 8, 9, 10, 11, 12, 13, 14]);
        assert_eq!(bytemuck::cast_slice::<u8, f32>(&results[2]), &[0.5]);
        Ok(())
    }

    #[tokio::test]
    async fn a_pending_readback_arrives_without_waiting() -> anyhow::Result<()> {
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
        let pending = Readback::record_pending(&context, &mut batch, &[&a, &b])?;
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
