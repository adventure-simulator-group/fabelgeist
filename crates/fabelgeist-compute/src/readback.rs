//! Many results back to the host for the price of one.
//!
//! [`Buffer::read`] copies its buffer to a staging buffer, submits, and
//! waits for the queue: one round trip per buffer. A pipeline that ends with a
//! dozen small results -- positions, normals, indices, a few status words --
//! pays that wait a dozen times. A [`Readback`] records every copy into the
//! batch that produced the results, lays them end to end in one staging
//! buffer, and maps it once after the batch is submitted.

use crate::prelude::*;

/// Results staged for one mapping, with checked logical byte ranges.
#[derive(Debug)]
pub struct Readback {
    staging: wgpu::Buffer,
    ranges: Vec<ReadbackRange>,
}

impl Readback {
    /// Record ordered copies after the commands already present in `batch`.
    pub fn record(
        context: &WgpuContext,
        batch: &mut KernelBatch,
        sources: ReadbackSources<'_>,
    ) -> std::result::Result<Self, ReadbackError> {
        let mut copies = Vec::new();
        let mut offset = BufferByteOffset::START;
        for buffer in sources {
            let length = buffer.length();
            let extent = length.copy_aligned()?;
            let available = BufferByteLength::from(buffer.buffer.size());
            if extent > available {
                return Err(ReadbackError::CopyStorage {
                    required: extent,
                    available,
                });
            }
            let range = ReadbackRange::new(offset, length)?;
            offset = offset.after(extent)?;
            copies.push((buffer, range, extent));
        }
        let staging = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: u64::from(offset).max(wgpu::COPY_BUFFER_ALIGNMENT),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut ranges = Vec::with_capacity(copies.len());
        for (buffer, range, extent) in copies {
            if extent != BufferByteLength::from(0u64) {
                batch.encoder().copy_buffer_to_buffer(
                    &buffer.buffer,
                    0,
                    &staging,
                    u64::from(range.start()),
                    u64::from(extent),
                );
            }
            ranges.push(range);
        }
        Ok(Self { staging, ranges })
    }

    /// Admit and allocate every host destination before obtaining a mapped view.
    pub async fn read(
        self,
        context: &WgpuContext,
    ) -> std::result::Result<ReadbackBlocks, ReadbackError> {
        let mut layouts = Vec::with_capacity(self.ranges.len());
        let mut results = Vec::with_capacity(self.ranges.len());
        for range in &self.ranges {
            layouts.push(BufferReadback::<u8>::new(range.length())?);
        }
        let mapping = ReadbackMapping::new(context, &self.staging).await?;
        let view = mapping.view()?;
        for (layout, range) in layouts.into_iter().zip(self.ranges) {
            results.push(layout.copy_from(view.bytes().segment(range)?)?);
        }
        Ok(ReadbackBlocks::new(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn every_buffer_comes_back_in_order() -> Result<()> {
        let context = WgpuContext::new().await?;
        let definition = BufferDefinition::storage();
        // A three-byte logical result needs padding before the next copy.
        let a = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[1u32, 2, 3]),
            definition.clone(),
        )?;
        let mut b = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[7u8, 8, 9, 10, 11, 12, 13, 14]),
            definition.clone(),
        )?;
        b = b.with_logical_length(3u64.into())?;
        let mut empty = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0u32]),
            definition.clone(),
        )?;
        empty = empty.with_logical_length(BufferByteLength::default())?;
        let c = Buffer::from_upload(&context, BufferUpload::from_elements(&[0.5f32]), definition)?;
        let mut batch = KernelBatch::new(&context);
        let readback = Readback::record(
            &context,
            &mut batch,
            ReadbackSources::from(&[&a, &b, &empty, &c][..]),
        )?;
        batch.submit();
        let results = readback.read(&context).await?;
        assert_eq!(
            results.get(ReadbackSlot::from(0))?.decode::<u32>()?,
            &[1, 2, 3]
        );
        assert_eq!(
            results.get(ReadbackSlot::from(1))?.decode::<u8>()?,
            [7, 8, 9]
        );
        assert!(
            results
                .get(ReadbackSlot::from(2))?
                .decode::<u32>()?
                .is_empty()
        );
        assert_eq!(results.get(ReadbackSlot::from(3))?.decode::<f32>()?, &[0.5]);
        let mut batch = KernelBatch::new(&context);
        let readback = Readback::record(&context, &mut batch, ReadbackSources::from(&[][..]))?;
        batch.submit();
        assert!(matches!(
            readback.read(&context).await?.get(ReadbackSlot::from(0)),
            Err(ReadbackError::MissingSlot(_))
        ));
        Ok(())
    }
}
