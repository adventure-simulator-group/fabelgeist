//! Default geometric charts with optional construction-authored coordinates.
use super::PartBuild;
use crate::{ArmorGpu, GenerateError};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

impl PartBuild {
    pub(crate) fn record_textile_chart(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        frame: &Buffer,
        points: &[u32],
    ) -> Result<(), GenerateError> {
        let coordinates = super::super::textile_chart::record(
            gpu,
            batch,
            &self.carriers,
            self.layout.carrier_count,
            frame,
        )?;
        super::super::textile_chart::record_authored(gpu, batch, &coordinates, points)?;
        self.textile_coordinates = Some(coordinates);
        Ok(())
    }
}
