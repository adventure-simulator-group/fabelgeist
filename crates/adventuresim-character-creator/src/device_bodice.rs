//! Evaluate a frozen bodice cut on every body realization before thickening.
use anyhow::Result;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS, record_body_cloth};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;

use crate::bodice_plan::BodicePlan;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_piece::DeviceRecording;
use crate::surface_cut::POINT_WORDS;

impl DeviceWearer<'_> {
    pub(crate) fn record_bodice(
        &self,
        batch: &mut KernelBatch,
        plan: &BodicePlan,
    ) -> Result<DeviceRecording> {
        let count = plan
            .layers
            .iter()
            .map(|layer| layer.point_count)
            .sum::<u32>();
        let mut words =
            vec![0_f32; (BODY_CLOTH_FIRST_POINT + count * BODY_CLOTH_POINT_WORDS) as usize];
        // Body cuts use canonical global metres. The shared shell kernel takes
        // an explicit identity frame rather than applying an anatomical frame twice.
        words[3] = 1.;
        words[7] = 1.;
        words[11] = 1.;
        let fit = self.gpu.upload(BufferUpload::from_elements(&words))?;
        let table = self.gpu.upload(BufferUpload::from_elements(&plan.table))?;
        let source = format!(
            "const SOURCE_WORDS: u32 = {POINT_WORDS}u;\nconst FIRST_POINT: u32 = {BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS: u32 = {BODY_CLOTH_POINT_WORDS}u;\n{}",
            include_str!("device_bodice.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", &fit),
                read("positions", &self.body.positions),
                read("normals", &self.body.normals),
                read_u32("table", &table),
            ],
            &[Word::U("count", count)],
            Grid::Items(count),
        )?;
        let part = record_body_cloth(self.gpu, batch, &plan.layers, &fit)?;
        Ok(DeviceRecording {
            part,
            frames: Vec::new(),
            checks: Vec::new(),
        })
    }
}
