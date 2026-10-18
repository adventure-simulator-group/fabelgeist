//! Typed scratch allocation and source uploads at wgpu buffer boundaries.

use super::error::{HairGpuError, Result};
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferDefinition, BufferLabel, BufferUpload, WgpuContext,
};

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
        let bytes = bytes.max(BufferByteLength::from(16_u64));
        let mut buffer = if let Some(cached) = &self.buffer
            && cached.buffer.size() >= u64::from(bytes)
        {
            cached.clone()
        } else {
            // Byte-layout arithmetic at allocation: retain the previous 3/2
            // growth and 16-byte alignment, without changing shader layouts.
            let grown = u64::from(bytes)
                .max(self.buffer.as_ref().map_or(0, |b| b.buffer.size() * 3 / 2))
                .next_multiple_of(16);
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
