//! Results of many stages brought back in one readback.
//!
//! Every read of a device buffer waits for the queue, and the queue is shared
//! by every thread fitting armor: a piece that read its positions, normals,
//! indices and statuses one by one waited for everyone else's work a dozen
//! times over. Instead each result stages the buffers it needs, one
//! [`Readback`] brings them all back, and each result parses its own slice.

use fabelgeist_compute::Readback;
use fabelgeist_gpu::prelude::{
    Buffer, ReadbackBlocks, ReadbackError, ReadbackSlot, ReadbackSources, ReadbackStatusWord,
};

use super::ArmorGpu;
use crate::GenerateError;

/// Buffers waiting to be read back together.
#[derive(Default)]
pub struct Staging<'a> {
    buffers: Vec<&'a Buffer>,
}

/// The bytes of every staged buffer.
pub struct StagedResults {
    bytes: ReadbackBlocks,
}

impl<'a> Staging<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stage(&mut self, buffer: &'a Buffer) -> ReadbackSlot {
        self.buffers.push(buffer);
        ReadbackSlot::from(self.buffers.len() - 1)
    }
}

impl StagedResults {
    /// Convert a staged device representation into initialized host values.
    pub fn get<T: bytemuck::Pod>(&self, slot: ReadbackSlot) -> Result<Vec<T>, GenerateError> {
        Ok(self.bytes.get(slot)?.decode()?)
    }

    /// Decode the first protocol word, rejecting an empty status buffer.
    pub fn status(&self, slot: ReadbackSlot) -> Result<ReadbackStatusWord, GenerateError> {
        self.get::<u32>(slot)?
            .first()
            .copied()
            .map(ReadbackStatusWord::from)
            .ok_or(GenerateError::Readback(ReadbackError::EmptyStatus))
    }
}

impl ArmorGpu {
    /// Read every staged buffer back with one mapping, after everything
    /// submitted so far.
    pub fn read_staged(&self, staging: Staging) -> Result<StagedResults, GenerateError> {
        let mut batch = self.batch(("armor readback").into());
        let readback = Readback::record(
            self.context(),
            &mut batch,
            ReadbackSources::from(&staging.buffers[..]),
        )?;
        batch.submit();
        let bytes =
            pollster::block_on(readback.read(self.context())).map_err(GenerateError::Readback)?;
        Ok(StagedResults { bytes })
    }
}
