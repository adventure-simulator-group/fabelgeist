//! Evaluate the frozen body patch on each wearer realization, on the device.

use anyhow::Result;
use fabelgeist_armor::gpu::{SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::gown_ease::NECK_SAMPLE_WORDS;
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::POINT_WORDS;

impl DeviceWearer<'_> {
    pub(crate) fn record_skirt_attachment(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SkirtJoinPlan,
        allowance: f32,
        status: &Buffer,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
    ) -> Result<()> {
        let table = self.gpu.upload(BufferUpload::from_elements(&plan.table))?;
        let neckline = self
            .gpu
            .upload(BufferUpload::from_elements(&plan.neckline_samples))?;
        let samples = self
            .gpu
            .upload(BufferUpload::from_elements(&plan.samples))?;
        let count = plan.patch.point_count + fabelgeist_armor::SKIRT_RING_SEGMENTS as u32;
        let geometry = plan
            .source
            .buffers(self, layers, &plan.table, &plan.source_faces)?;
        let source = format!(
            "const SOURCE_WORDS: u32 = {POINT_WORDS}u;\nconst FIRST_POINT: u32 = {SKIRT_FIT_WORDS}u;\nconst OUTPUT_WORDS: u32 = {SKIRT_ATTACHMENT_POINT_WORDS}u;\nconst NECK_WORDS:u32={NECK_SAMPLE_WORDS}u;\nconst RADII:u32={}u;\nconst ROWS:u32={}u;\n{}\n{}",
            fabelgeist_armor::SKIRT_RING_SEGMENTS,
            fabelgeist_armor::SKIRT_AXIAL_SEGMENTS,
            include_str!("device_cloth_source.wgsl"),
            include_str!("device_skirt_join.wgsl"),
        );
        let enclosure = self.record_gown_enclosure(
            batch,
            fit,
            plan,
            allowance,
            crate::device_gown_enclosure::EnclosureSources {
                table: &table,
                neckline: &neckline,
                geometry: &geometry,
            },
        )?;
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read("positions", &geometry.positions),
                read("normals", &geometry.normals),
                read_u32("table", &table),
                read_u32("samples", &samples),
                read_u32("neckline", &neckline),
                read("enclosure", &enclosure),
                atomic("waist_status", status),
            ],
            &[
                Word::U("count", count),
                Word::U("points", plan.patch.point_count),
                Word::U("sources", (plan.table.len() / POINT_WORDS) as u32),
                Word::F("allowance", allowance),
                Word::F("ease", plan.ease.metres()),
                Word::F("fullness_release", plan.release.unit()),
                Word::U(
                    "loose",
                    u32::from(plan.release_weights.iter().any(|&value| value > 0.)),
                ),
            ],
            Grid::Items(count),
        )?;
        self.record_cloth_seating(batch, fit, plan, allowance)
    }
}
