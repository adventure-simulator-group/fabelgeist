//! Ping-pong resource roles remain paired across each digit and final copy.
use super::quantities::SortTileCount;
use super::*;
use crate::kernel::KernelBatch;

pub(super) struct SortPassBuffers {
    source_keys: Buffer,
    source_values: Buffer,
    target_keys: Buffer,
    target_values: Buffer,
    histogram: Buffer,
    count: SortItemCount,
    tiles: SortTileCount,
}
impl SortPassBuffers {
    pub(super) fn new(
        keys: &Buffer,
        values: &Buffer,
        scratch: &SortScratch,
        count: SortItemCount,
    ) -> Self {
        Self {
            source_keys: keys.clone(),
            source_values: values.clone(),
            target_keys: scratch.keys.clone(),
            target_values: scratch.values.clone(),
            histogram: scratch.histogram.clone(),
            count,
            tiles: count.tiles(),
        }
    }
    pub(super) fn record_digit(
        &mut self,
        batch: &mut KernelBatch,
        sort: &RadixSort,
        digit: SortDigit,
    ) -> std::result::Result<(), SortError> {
        let mut parameters = PassParameters::new();
        self.count.bind(&mut parameters);
        digit.bind(&mut parameters);
        self.tiles.bind(&mut parameters);
        parameters.insert("pad".into(), 0u32.into());
        parameters.insert("histogram".into(), self.histogram.clone().into());
        let mut histogram = parameters.clone();
        histogram.insert("keys".into(), self.source_keys.clone().into());
        batch
            .dispatch(&sort.histogram, &histogram, self.tiles.grid())
            .map_err(|source: KernelDispatchError| -> SortError {
                SortError::Dispatch {
                    stage: SortDispatchStage {
                        kernel: SortKernelRole::Histogram,
                        digit,
                    },
                    source: Box::new(source),
                }
            })?;
        batch
            .dispatch(&sort.scan, &parameters, [1, 1, 1].into())
            .map_err(|source: KernelDispatchError| -> SortError {
                SortError::Dispatch {
                    stage: SortDispatchStage {
                        kernel: SortKernelRole::Scan,
                        digit,
                    },
                    source: Box::new(source),
                }
            })?;
        let mut scatter = parameters;
        scatter.insert("keys_in".into(), self.source_keys.clone().into());
        scatter.insert("values_in".into(), self.source_values.clone().into());
        scatter.insert("keys_out".into(), self.target_keys.clone().into());
        scatter.insert("values_out".into(), self.target_values.clone().into());
        batch
            .dispatch(&sort.scatter, &scatter, self.tiles.grid())
            .map_err(|source: KernelDispatchError| -> SortError {
                SortError::Dispatch {
                    stage: SortDispatchStage {
                        kernel: SortKernelRole::Scatter,
                        digit,
                    },
                    source: Box::new(source),
                }
            })?;
        std::mem::swap(&mut self.source_keys, &mut self.target_keys);
        std::mem::swap(&mut self.source_values, &mut self.target_values);
        Ok(())
    }
    pub(super) fn copy_back(
        &self,
        batch: &mut KernelBatch,
        keys: &Buffer,
        values: &Buffer,
    ) -> std::result::Result<(), SortError> {
        batch
            .copy_buffer(&self.source_keys, keys, self.count.word_bytes())
            .map_err(|source: BufferCopyError| -> SortError {
                SortError::Copy {
                    role: SortBufferRole::Keys,
                    source,
                }
            })?;
        batch
            .copy_buffer(&self.source_values, values, self.count.word_bytes())
            .map_err(|source: BufferCopyError| -> SortError {
                SortError::Copy {
                    role: SortBufferRole::Values,
                    source,
                }
            })?;
        Ok(())
    }
}
