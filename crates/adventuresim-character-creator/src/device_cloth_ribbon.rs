//! Folded tapes query the actual fitted cloth surface before thickening.
use crate::cloth_ribbon_plan::ClothRibbonPlan;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use anyhow::{Result, ensure};
use fabelgeist_compute::{KernelBatch, QueryHits, TriangleTargets};
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

/// Native float-table layout: first point word and position/normal stride.
#[derive(Clone, Copy)]
pub(crate) struct ClothFitLayout {
    pub first_point: u32,
    pub point_words: u32,
}
impl DeviceWearer<'_> {
    pub(crate) fn record_cloth_ribbon(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        ribbon: &ClothRibbonPlan,
        layout: ClothFitLayout,
    ) -> Result<()> {
        let tape = self
            .gpu
            .upload(BufferUpload::from_elements(&ribbon.words))?;
        let triangles = self
            .gpu
            .upload(BufferUpload::from_elements(&ribbon.carrier_indices))?;
        let count = ribbon.point_count();
        let points = self
            .gpu
            .scratch(u64::from(ribbon.carrier_points) * 12, "bound cloth surface")?;
        let queries = self
            .gpu
            .scratch(u64::from(count) * 12, "tape seating queries")?;
        let source = ribbon.source(layout.first_point, layout.point_words);
        dispatch(
            self,
            batch,
            &source,
            &[
                read("fit", fit),
                read_u32("tape", &tape),
                write("cloth_points", &points),
                write("queries", &queries),
            ],
            &[
                Word::U("count", count),
                Word::U("carrier_count", ribbon.carrier_points),
                Word::F("carrier_offset", ribbon.carrier_offset_metres),
            ],
            Grid::Items(count.max(ribbon.carrier_points)),
        )?;
        // fit_piece_async records and submits the reference before any morph.
        // Freezing its query prevents a tape point changing face ownership
        // between the morph targets that the runtime interpolates.
        if ribbon.reference_projection.get().is_none() {
            let hits = QueryHits::new(self.gpu.context(), count)?;
            self.gpu.query().record_closest_triangles(
                batch,
                &queries,
                count,
                TriangleTargets {
                    positions: &points,
                    triangles: &triangles,
                    candidates: None,
                    count: ribbon.carrier_indices.len() as u32 / 3,
                },
                &hits,
            )?;
            ensure!(
                ribbon.reference_projection.set(hits).is_ok(),
                "cloth tape reference projection was recorded concurrently"
            );
        }
        let hits = ribbon.reference_projection.get().ok_or_else(|| {
            anyhow::anyhow!("cloth tape reference projection has not been recorded")
        })?;
        let source = format!(
            "const FIRST_POINT:u32={}u;\nconst POINT_WORDS:u32={}u;\n{}",
            layout.first_point,
            layout.point_words,
            include_str!("device_cloth_ribbon_projection.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read("cloth_points", &points),
                read_u32("triangles", &triangles),
                read_u32("nearest", &hits.nearest),
                read("weights", &hits.weights),
            ],
            &[
                Word::U("first", ribbon.first),
                Word::U("count", count),
                Word::F("carrier_offset", ribbon.carrier_offset_metres),
            ],
            Grid::Items(count),
        )
    }
}
