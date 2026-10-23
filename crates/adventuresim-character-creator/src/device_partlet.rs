//! Evaluate frozen panel sources and evenly sampled collar points on the device.
use crate::device_frames::DeviceWearer;
use crate::device_piece::DeviceRecording;
use crate::partlet_plan::PartletPlan;
use anyhow::Result;
use fabelgeist_armor::PartletDesign;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, record_body_cloth};
use fabelgeist_compute::KernelBatch;

impl DeviceWearer<'_> {
    pub(crate) fn record_partlet(
        &self,
        batch: &mut KernelBatch,
        design: &PartletDesign,
        plan: &PartletPlan,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
    ) -> Result<DeviceRecording> {
        let count = plan.cloth.layers.iter().map(|l| l.point_count).sum();
        let fit = self.record_cloth_sources(batch, &plan.cloth, count, layers)?;
        self.record_partlet_collar(batch, design, plan, &fit)?;
        self.record_partlet_collar_seating(batch, &fit, plan)?;
        if let Some(binding) = &plan.binding {
            self.record_cloth_ribbon(
                batch,
                &fit,
                binding,
                crate::device_cloth_ribbon::ClothFitLayout {
                    first_point: BODY_CLOTH_FIRST_POINT,
                    point_words: BODY_CLOTH_POINT_WORDS,
                },
            )?;
        }
        self.record_partlet_ties(batch, design, plan, &fit, layers)?;
        let part = record_body_cloth(self.gpu, batch, &plan.cloth.layers, &fit)?;
        self.record_fit_status(batch, &fit, &part)?;
        Ok(DeviceRecording {
            part,
            frames: Vec::new(),
            checks: Vec::new(),
        })
    }
}
