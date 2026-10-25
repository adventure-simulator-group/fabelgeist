//! Read many staged buffer prefixes through one grouped mapping.
//!
//! Sources retain their staging order. Handle producing submission results
//! before reading; the readback then waits for its copies and mapping, using
//! native blocking progress or browser callbacks on the producing device.

use super::{ArmorGpu, device_error};
use crate::GenerateError;
use fabelgeist_compute::Readback;
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferReadError};
use std::sync::Arc;

pub type StagingResult<T> = Result<T, StagingError>;

/// Buffers waiting to be read back together.
#[derive(Default)]
pub struct Staging<'a> {
    session: Arc<StagingSession>,
    buffers: Vec<&'a Buffer>,
}

/// A buffer's position in one particular staged readback.
#[derive(Clone, Debug)]
pub struct Staged {
    session: Arc<StagingSession>,
    index: usize,
}

/// The bytes of every staged buffer, associated with their issued tickets.
pub struct StagedResults {
    session: Arc<StagingSession>,
    bytes: Vec<Vec<u8>>,
}

#[derive(Debug, Default)]
struct StagingSession;

/// A ticket or requested host prefix does not fit its staged bytes.
#[derive(Debug, thiserror::Error)]
pub enum StagingError {
    #[error("the staged ticket does not belong to these results")]
    InvalidTicket,
    #[error("staged prefix of {elements} {element_bytes}-byte elements exceeds host addresses")]
    PrefixSizeOverflow {
        elements: usize,
        element_bytes: usize,
    },
    #[error("armor staged bytes: {0}")]
    Decode(#[from] BufferReadError),
}

impl<'a> Staging<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stage(&mut self, buffer: &'a Buffer) -> Staged {
        let index = self.buffers.len();
        self.buffers.push(buffer);
        Staged {
            session: self.session.clone(),
            index,
        }
    }
}

impl StagedResults {
    /// Decode exactly `count` whole elements, excluding allocation padding.
    ///
    /// Zero elements are valid even when the allocation contains a sentinel
    /// word. Foreign tickets, zero-sized elements and short prefixes fail;
    /// missing bytes are never synthesized by padding or truncation.
    pub fn prefix<T: bytemuck::Pod>(&self, staged: &Staged, count: usize) -> StagingResult<Vec<T>> {
        if !Arc::ptr_eq(&self.session, &staged.session) {
            return Err(StagingError::InvalidTicket);
        }
        let bytes = self
            .bytes
            .get(staged.index)
            .ok_or(StagingError::InvalidTicket)?;
        let element_bytes = std::mem::size_of::<T>();
        if element_bytes == 0 {
            return Err(BufferReadError::ZeroSizedElement.into());
        }
        let requested =
            count
                .checked_mul(element_bytes)
                .ok_or(StagingError::PrefixSizeOverflow {
                    elements: count,
                    element_bytes,
                })?;
        let bytes = bytes
            .get(..requested)
            .ok_or_else(|| BufferReadError::Truncated {
                requested: BufferByteLength::from(requested),
                available: BufferByteLength::from(bytes.len()),
            })?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(BufferReadError::HostAllocation)?;
        values.resize(count, T::zeroed());
        bytemuck::cast_slice_mut(&mut values).copy_from_slice(bytes);
        Ok(values)
    }

    /// Decode the first complete native u32 status word.
    pub fn status(&self, staged: &Staged) -> StagingResult<u32> {
        // prefix proves this fixed one-element index before it is used.
        Ok(self.prefix::<u32>(staged, 1)?[0])
    }
}

impl ArmorGpu {
    /// Read all staged sources with one grouped mapping on their device.
    ///
    /// Every source must belong to this facade's device, and the context queue
    /// must be paired with that device. Sources must be available for the GPU
    /// copies under their existing mapping ownership. Handle producing
    /// submission results before calling this method.
    pub fn read_staged(&self, staging: Staging) -> Result<StagedResults, GenerateError> {
        pollster::block_on(self.read_staged_async(staging))
    }

    /// Read staged sources using this facade's device and paired queue.
    ///
    /// Every source must belong to that device and be available for the GPU
    /// copies under its existing mapping ownership. Handle producing
    /// submission results first; the grouped staging allocation is mapped
    /// using native progress or browser callbacks.
    pub async fn read_staged_async(
        &self,
        staging: Staging<'_>,
    ) -> Result<StagedResults, GenerateError> {
        let mut batch = self.batch("armor readback");
        let readback = Readback::record(&mut batch, &staging.buffers)?;
        batch.submit().map_err(device_error)?;
        let bytes = readback.read().await?;
        Ok(StagedResults {
            session: staging.session,
            bytes,
        })
    }
}
