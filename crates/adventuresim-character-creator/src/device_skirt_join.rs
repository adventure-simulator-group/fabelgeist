//! Evaluate the frozen body patch on each wearer realization, on the device.

use anyhow::Result;
use fabelgeist_armor::gpu::{SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::POINT_WORDS;

impl DeviceWearer<'_> {
    pub(crate) fn record_skirt_attachment(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SkirtJoinPlan,
        allowance: f32,
    ) -> Result<()> {
        let table = self.gpu.upload(BufferUpload::from_elements(&plan.table))?;
        let samples = self
            .gpu
            .upload(BufferUpload::from_elements(&plan.samples))?;
        let count = plan.patch.point_count + fabelgeist_armor::SKIRT_RING_SEGMENTS as u32;
        let source = format!(
            "const SOURCE_WORDS: u32 = {POINT_WORDS}u;\nconst FIRST_POINT: u32 = {SKIRT_FIT_WORDS}u;\nconst OUTPUT_WORDS: u32 = {SKIRT_ATTACHMENT_POINT_WORDS}u;\n{}",
            include_str!("device_skirt_join.wgsl"),
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read("positions", &self.body.positions),
                read("normals", &self.body.normals),
                read_u32("table", &table),
                read_u32("samples", &samples),
            ],
            &[
                Word::U("count", count),
                Word::U("points", plan.patch.point_count),
                Word::F("allowance", allowance),
            ],
            Grid::Items(count),
        )
    }
}
