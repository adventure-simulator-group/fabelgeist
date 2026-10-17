//! One smooth torso/lower-body enclosure for released cloth.
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::gown_ease::NECK_SAMPLE_WORDS;
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::POINT_WORDS;
use anyhow::Result;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

impl DeviceWearer<'_> {
    pub(crate) fn record_gown_enclosure(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SkirtJoinPlan,
        allowance: f32,
        table: &Buffer,
        neckline: &Buffer,
    ) -> Result<Buffer> {
        let enclosure = self.gpu.upload(BufferUpload::from_elements(&[0f32; 2]))?;
        if !plan.release_weights.iter().any(|&value| value > 0.) {
            return Ok(enclosure);
        }
        let support = self
            .gpu
            .upload(BufferUpload::from_elements(&plan.enclosure_table))?;
        let source = format!(
            "const SOURCE_WORDS:u32={POINT_WORDS}u;\nconst NECK_WORDS:u32={NECK_SAMPLE_WORDS}u;\nconst RADII:u32={}u;\nconst ROWS:u32={}u;\n{}\n{}",
            fabelgeist_armor::SKIRT_RING_SEGMENTS,
            fabelgeist_armor::SKIRT_AXIAL_SEGMENTS,
            include_str!("device_cloth_source.wgsl"),
            include_str!("device_gown_enclosure.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                read("fit", fit),
                read("positions", &self.body.positions),
                read("normals", &self.body.normals),
                read_u32("table", table),
                read_u32("neckline", neckline),
                read_u32("enclosure_table", &support),
                write("enclosure", &enclosure),
            ],
            &[
                Word::U("sources", (plan.table.len() / POINT_WORDS) as u32),
                Word::U(
                    "enclosure_sources",
                    (plan.enclosure_table.len() / POINT_WORDS) as u32,
                ),
                Word::F("allowance", allowance),
                Word::F("ease", plan.ease.metres()),
            ],
            Grid::Singles(1),
        )?;
        Ok(enclosure)
    }
}
