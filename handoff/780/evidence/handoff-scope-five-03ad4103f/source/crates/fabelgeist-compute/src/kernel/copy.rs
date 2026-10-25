//! Ordered byte-range copies inside a compute batch.

use super::{KernelBatch, KernelBatchError};
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use fabelgeist_gpu::prelude::ComputePassError;
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferByteOffset};
#[cfg(not(target_arch = "wasm32"))]
use fabelgeist_gpu::globals::ValidationScope;

/// Requested source and destination byte ranges for one buffer copy.
/// Bounds are admitted against the supplied buffers when the copy is recorded.
#[derive(Clone, Copy, Debug)]
pub struct BufferCopy {
    pub source_offset: BufferByteOffset,
    pub destination_offset: BufferByteOffset,
    pub bytes: BufferByteLength,
}

impl KernelBatch<'_> {
    /// Copy byte ranges in order with the surrounding batch operations.
    /// Logical bounds are checked before recording. Native buffer requirements
    /// follow the context's validation policy, including for zero-byte copies.
    /// Native encoding errors can remain deferred until batch submission.
    pub fn copy_buffer(
        &mut self,
        source: &Buffer,
        destination: &Buffer,
        copy: BufferCopy,
    ) -> Result<()> {
        // These comparisons are at the native byte-address receiving boundary.
        let source_fits = copy
            .source_offset
            .checked_after(copy.bytes)
            .is_some_and(|end| u64::from(end) <= u64::from(source.size));
        let destination_fits = copy
            .destination_offset
            .checked_after(copy.bytes)
            .is_some_and(|end| u64::from(end) <= u64::from(destination.size));
        if !source_fits || !destination_fits {
            return Err(KernelBatchError::CopyOutOfBounds {
                copy,
                source: source.size,
                destination: destination.size,
            }
            .into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let scope = ValidationScope::new(self.context);
        self.encoder.copy_buffer_to_buffer(
            &source.buffer,
            u64::from(copy.source_offset),
            &destination.buffer,
            u64::from(copy.destination_offset),
            u64::from(copy.bytes),
        );
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            scope.finish().map_err(ComputePassError::from)?;
        }
        Ok(())
    }
}
