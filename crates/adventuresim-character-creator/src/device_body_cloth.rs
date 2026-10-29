//! Evaluate a frozen bodice cut on every body realization before thickening.
use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, record_body_cloth};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

use crate::body_cloth_plan::BodyClothPlan;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;
use crate::surface_cut::POINT_WORDS;

impl DeviceWearer<'_> {
    pub(crate) fn record_body_cloth(
        &self,
        batch: &mut KernelBatch,
        plan: &BodyClothPlan,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
    ) -> Result<DeviceRecording> {
        let count = plan
            .layers
            .iter()
            .map(|layer| layer.point_count)
            .sum::<u32>();
        ensure!(
            plan.table.len() / POINT_WORDS == count as usize,
            "cloth has unresolved construction points"
        );
        let fit = self.record_cloth_sources(batch, plan, count, layers)?;
        let part = record_body_cloth(self.gpu, batch, &plan.layers, &fit)?;
        Ok(DeviceRecording {
            part,
            frames: Vec::new(),
            checks: Vec::new(),
        })
    }

    /// Native fitted-table adapter. Additional collar/panel points must be
    /// written by the construction's device kernel before shell evaluation.
    pub(crate) fn record_cloth_sources(
        &self,
        batch: &mut KernelBatch,
        plan: &BodyClothPlan,
        capacity: u32,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
    ) -> Result<Buffer> {
        ensure!(
            plan.table.len().is_multiple_of(POINT_WORDS),
            "partial cloth source record"
        );
        let count = u32::try_from(plan.table.len() / POINT_WORDS)?;
        ensure!(
            count > 0 && count <= capacity,
            "invalid cloth source capacity"
        );
        let mut words =
            vec![0_f32; (BODY_CLOTH_FIRST_POINT + capacity * BODY_CLOTH_POINT_WORDS) as usize];
        // Body cuts use canonical global metres. The shared shell kernel takes
        // an explicit identity frame rather than applying an anatomical frame twice.
        words[3] = 1.;
        words[7] = 1.;
        words[11] = 1.;
        let fit = self.gpu.upload(BufferUpload::from_elements(&words))?;
        let table = self.gpu.upload(BufferUpload::from_elements(&plan.table))?;
        let geometry = plan
            .source
            .buffers(self, layers, &plan.table, &plan.source_faces)?;
        let source = format!(
            "const SOURCE_WORDS: u32 = {POINT_WORDS}u;\nconst FIRST_POINT: u32 = {BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS: u32 = {BODY_CLOTH_POINT_WORDS}u;\n{}",
            include_str!("device_body_cloth.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", &fit),
                read("positions", &geometry.positions),
                read("normals", &geometry.normals),
                read_u32("table", &table),
            ],
            &[Word::U("count", count)],
            Grid::Items(count),
        )?;
        Ok(fit)
    }
}
