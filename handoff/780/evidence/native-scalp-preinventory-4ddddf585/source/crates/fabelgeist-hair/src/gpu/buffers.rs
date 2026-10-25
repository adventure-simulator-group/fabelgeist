//! Typed scratch allocation and source uploads at wgpu buffer boundaries.

use super::error::{HairGpuError, Result};
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferDefinition, BufferLabel, BufferUpload, WgpuContext,
};

// Native vec4-sized scratch binding: minimum allocation and alignment.
const SCRATCH_ALIGNMENT_BYTES: u64 = 16;

/// A buffer kept between grooms and grown by half again when too small, as
/// the solvers keep theirs, so that grooming the same head again
/// allocates nothing. Only for scratch: what a groom hands out is its own.
#[derive(Default)]
pub(super) struct Scratch {
    buffer: Option<Buffer>,
}

impl Scratch {
    pub(super) fn fit(
        &mut self,
        context: &WgpuContext,
        bytes: BufferByteLength,
        label: BufferLabel,
    ) -> Result<Buffer> {
        let bytes = bytes.max(BufferByteLength::from(SCRATCH_ALIGNMENT_BYTES));
        let mut buffer = if let Some(cached) = &self.buffer
            && cached.buffer.size() >= u64::from(bytes)
        {
            cached.clone()
        } else {
            let previous = self.buffer.as_ref().map_or(0, |buffer| buffer.buffer.size());
            // previous + previous / 2 retains the exact floor of 3/2 growth
            // without overflowing an intermediate product.
            let grown = previous
                .checked_add(previous / 2)
                .map(|grown| grown.max(u64::from(bytes)))
                .and_then(|grown| {
                    grown.div_ceil(SCRATCH_ALIGNMENT_BYTES)
                        .checked_mul(SCRATCH_ALIGNMENT_BYTES)
                })
                .ok_or(HairGpuError::ScratchCapacity {
                    requested: bytes,
                    previous: BufferByteLength::from(previous),
                })?;
            Buffer::new(
                context,
                BufferByteLength::from(grown),
                BufferDefinition::storage().with_label(label),
            )?
        };
        self.buffer = Some(buffer.clone());
        buffer.size = bytes;
        Ok(buffer)
    }
}

/// A storage buffer holding `data`, never empty.
pub(super) fn upload<T: bytemuck::NoUninit + Default>(
    context: &WgpuContext,
    data: &[T],
    label: BufferLabel,
) -> Result<Buffer> {
    if data.is_empty() {
        return Buffer::from_upload(
            context,
            BufferUpload::from_elements(&[T::default(); 4]),
            BufferDefinition::storage().with_label(label),
        )
        .map_err(HairGpuError::Backend);
    }
    Buffer::from_upload(
        context,
        BufferUpload::from_elements(data),
        BufferDefinition::storage().with_label(label),
    )
    .map_err(HairGpuError::Backend)
}

/// A fresh storage buffer of `bytes`.
pub(super) fn storage(
    context: &WgpuContext,
    bytes: BufferByteLength,
    label: BufferLabel,
) -> Result<Buffer> {
    Buffer::new(
        context,
        bytes.max(BufferByteLength::from(16_u64)),
        BufferDefinition::storage().with_label(label),
    )
    .map_err(HairGpuError::Backend)
}
