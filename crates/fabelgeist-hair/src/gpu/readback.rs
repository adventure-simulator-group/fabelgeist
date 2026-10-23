//! Grouped native word copies and one host mapping for grooming results.

use super::{GpuGroomer, HairGpuError, Result};
use fabelgeist_compute::{BufferCopy, KernelBatch, Readback};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferByteOffset};

impl GpuGroomer {
    /// One word of `buffer`, `at` words in, read back.
    pub(super) fn read_word(&mut self, batch: KernelBatch, buffer: &Buffer, at: u32) -> Result<u32> {
        // The single requested source always yields one decoded word.
        Ok(self.read_words(batch, &[(buffer.clone(), at)])?[0])
    }

    /// Words of buffers (each `at` words in), read back after `batch` in
    /// one mapping. Word indices become byte addresses at this native layout.
    pub(super) fn read_words(
        &mut self,
        mut batch: KernelBatch,
        words: &[(Buffer, u32)],
    ) -> Result<Vec<u32>> {
        let expected = words
            .len()
            .checked_mul(std::mem::size_of::<u32>())
            .map(BufferByteLength::from)
            .ok_or(HairGpuError::ReadbackExtentOverflow)?;
        let out =
            self.scratch
                .word
                .fit(&self.context, expected, "hair readback words".into())?;
        let word_bytes = BufferByteLength::from(std::mem::size_of::<u32>());
        for (index, (buffer, at)) in words.iter().enumerate() {
            // u32 word indices times four fit u64. The checked total above
            // proves every destination byte offset fits the host extent.
            batch.copy_buffer(
                buffer,
                &out,
                BufferCopy {
                    source_offset: BufferByteOffset::from(u64::from(*at) * u64::from(word_bytes)),
                    destination_offset: BufferByteOffset::from(
                        (index * std::mem::size_of::<u32>()) as u64,
                    ),
                    bytes: word_bytes,
                },
            )?;
        }
        let readback = Readback::record(&mut batch, &[&out])?;
        batch.submit()?;
        let results = pollster::block_on(readback.read())?;
        // Exactly one source was recorded, including the empty-word request.
        let bytes = &results[0];
        if BufferByteLength::from(bytes.len()) < expected {
            return Err(HairGpuError::ShortReadback {
                expected,
                actual: BufferByteLength::from(bytes.len()),
            });
        }
        Ok(bytes
            .chunks_exact(std::mem::size_of::<u32>())
            .take(words.len())
            .map(|word| u32::from_ne_bytes([word[0], word[1], word[2], word[3]]))
            .collect())
    }
}
