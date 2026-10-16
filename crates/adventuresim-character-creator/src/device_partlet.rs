//! Evaluate frozen panel sources and evenly sampled collar points on the device.
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read_u32, write};
use crate::device_piece::DeviceRecording;
use crate::partlet_plan::PartletPlan;
use anyhow::Result;
use fabelgeist_armor::PartletDesign;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, record_body_cloth};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;

impl DeviceWearer<'_> {
    pub(crate) fn record_partlet(
        &self,
        batch: &mut KernelBatch,
        design: &PartletDesign,
        plan: &PartletPlan,
    ) -> Result<DeviceRecording> {
        let count = plan.cloth.layers.iter().map(|l| l.point_count).sum();
        let fit = self.record_cloth_sources(batch, &plan.cloth, count)?;
        self.record_partlet_collar(batch, design, plan, &fit)?;
        if let Some(binding) = &plan.binding {
            let tape = self
                .gpu
                .upload(BufferUpload::from_elements(&binding.words))?;
            let source = format!(
                "const FIRST_POINT:u32={BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS:u32={BODY_CLOTH_POINT_WORDS}u;\n{}",
                include_str!("device_cloth_ribbon.wgsl")
            );
            dispatch(
                self,
                batch,
                &source,
                &[write("fit", &fit), read_u32("tape", &tape)],
                &[
                    Word::U("first", binding.first),
                    Word::U("count", binding.words.len() as u32 / 4),
                ],
                Grid::Items(binding.words.len() as u32 / 4),
            )?;
        }
        let part = record_body_cloth(self.gpu, batch, &plan.cloth.layers, &fit)?;
        self.record_fit_status(batch, &fit, &part)?;
        Ok(DeviceRecording {
            part,
            frames: Vec::new(),
            checks: Vec::new(),
        })
    }
}
