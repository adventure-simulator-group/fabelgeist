//! Results of many stages brought back in one readback.
//!
//! Every read of a device buffer waits for the queue, and the queue is shared
//! by every thread fitting armor: a piece that read its positions, normals,
//! indices and statuses one by one waited for everyone else's work a dozen
//! times over. Instead each result stages the buffers it needs, one
//! [`Readback`] brings them all back, and each result parses its own slice.

use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_compute::Readback;
use fabelgeist_gpu::prelude::Buffer;

use super::{ArmorGpu, device_error};
use crate::GenerateError;

/// Buffers waiting to be read back together.
#[derive(Default)]
pub struct Staging<'a> {
    buffers: Vec<&'a Buffer>,
}

/// Where one staged buffer's bytes will be.
#[derive(Clone, Copy, Debug)]
pub struct Staged(usize);

/// The bytes of every staged buffer.
pub struct StagedResults {
    bytes: Vec<Vec<u8>>,
}

impl<'a> Staging<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stage(&mut self, buffer: &'a Buffer) -> Staged {
        self.buffers.push(buffer);
        Staged(self.buffers.len() - 1)
    }
}

impl StagedResults {
    /// A staged buffer's contents as plain values.
    pub fn get<T: bytemuck::Pod>(&self, staged: Staged) -> Vec<T> {
        bytemuck::pod_collect_to_vec(&self.bytes[staged.0])
    }

    /// Typed prefix, excluding storage allocation padding.
    pub fn prefix<T: bytemuck::Pod>(&self, staged: Staged, count: usize) -> Vec<T> {
        let mut values = self.get(staged);
        values.truncate(count);
        values
    }

    /// The first word of a staged status buffer.
    pub fn status(&self, staged: Staged) -> u32 {
        self.get::<u32>(staged)[0]
    }
}

impl ArmorGpu {
    /// Read every staged buffer back with one mapping, after everything
    /// submitted so far.
    pub fn read_staged(&self, staging: Staging) -> Result<StagedResults, GenerateError> {
        pollster::block_on(self.read_staged_async(staging))
    }

    pub async fn read_staged_async(
        &self,
        staging: Staging<'_>,
    ) -> Result<StagedResults, GenerateError> {
        let mut batch = self.batch(KernelBatchLabel::from("armor readback"));
        let readback = Readback::record(self.context(), &mut batch, &staging.buffers);
        batch.submit();
        let bytes = readback.read(self.context()).await.map_err(device_error)?;
        Ok(StagedResults { bytes })
    }
}
