//! Sewn collar and separate smooth bands share fitted neckline correspondence.
use anyhow::Result;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS};
use fabelgeist_armor::{CollarSection, PartletDesign, StandingClothCollar};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::partlet_collar::{COLLAR_SAMPLE_WORDS, CollarPlan, MAX_COLLAR_COLUMNS};
use crate::partlet_plan::PartletPlan;

/// Numerical kernel boundary: dimensions in canonical metres, native ordinals.
struct CollarSheet {
    first: u32,
    rows: u32,
    first_row: u32,
    bottom: f32,
    height: f32,
    clearance_metres: f32,
}

/// Bound buffers for one collar recording; ordinals and metres remain device-native.
pub(crate) struct CollarBuffers<'a> {
    pub fit: &'a Buffer,
    pub samples: &'a Buffer,
    pub row_heights: &'a Buffer,
    pub seating: &'a Buffer,
    pub status: &'a Buffer,
}

impl DeviceWearer<'_> {
    pub(crate) fn record_partlet_collar(
        &self,
        batch: &mut KernelBatch,
        design: &PartletDesign,
        plan: &PartletPlan,
        fit: &Buffer,
    ) -> Result<()> {
        let (Some(collar), Some(geometry)) = (&plan.collar, &design.collar) else {
            return Ok(());
        };
        let samples = self
            .gpu
            .upload(BufferUpload::from_elements(&collar.samples))?;
        let row_heights = self
            .gpu
            .upload(BufferUpload::from_elements(&collar.heights))?;
        let seating = self.gpu.upload(BufferUpload::from_elements(&vec![
            0_f32;
            (collar.columns + crate::device_collar_seating::SEAT_COEFFICIENT_WORDS)
                as usize
        ]))?;
        let status = self.gpu.upload(BufferUpload::from_elements(&[0_u32]))?;
        let buffers = CollarBuffers {
            fit,
            samples: &samples,
            row_heights: &row_heights,
            seating: &seating,
            status: &status,
        };
        self.record_collar_seating(batch, &buffers, collar, geometry)?;
        // Generated collar rows preserve the same sampled resting minimum as
        // their fitted panel source; authored ease can request a larger gap.
        let clearance_metres = design
            .clearance
            .metres()
            .max(crate::cloth_panel::MINIMUM_CLOTH_CLEARANCE_M);
        let sheet = CollarSheet {
            first: plan.cloth.surface.points.len() as u32,
            rows: collar.rows,
            first_row: 1,
            bottom: 0.,
            height: geometry.height.metres(),
            clearance_metres,
        };
        self.record_collar_sheet(batch, &buffers, collar, geometry, &sheet)?;
        for band in &plan.bands {
            let sheet = CollarSheet {
                first: band.first,
                rows: band.rows,
                first_row: 0,
                bottom: band.bottom,
                height: band.height,
                clearance_metres,
            };
            self.record_collar_sheet(batch, &buffers, collar, geometry, &sheet)?;
            if matches!(geometry.section, CollarSection::Oval { .. }) {
                continue;
            }
            dispatch(
                self,
                batch,
                &collar_source(include_str!("device_cloth_band_smooth.wgsl")),
                &[write("fit", fit)],
                &[
                    Word::U("first", band.first),
                    Word::U("columns", collar.columns),
                    Word::U("rows", band.rows),
                ],
                Grid::Singles(band.rows),
            )?;
        }
        dispatch(
            self,
            batch,
            "@compute @workgroup_size(1) fn main() { if (atomicLoad(&collar_status[0])!=0u) {fit[FIT_FAILED]=1.0;} }",
            &[write("fit", fit), atomic("collar_status", &status)],
            &[],
            Grid::Singles(1),
        )?;
        Ok(())
    }

    fn record_collar_sheet(
        &self,
        batch: &mut KernelBatch,
        buffers: &CollarBuffers<'_>,
        collar: &CollarPlan,
        geometry: &StandingClothCollar,
        sheet: &CollarSheet,
    ) -> Result<()> {
        let mut words = self.collar_geometry_words(collar, geometry)?;
        words.extend([
            Word::U("first", sheet.first),
            Word::U("rows", sheet.rows),
            Word::U("first_row", sheet.first_row),
            Word::F("bottom", sheet.bottom),
            Word::F("height", sheet.height),
            Word::F("clearance", sheet.clearance_metres),
            Word::F("free_start", geometry.free_edge_start_metres()),
            Word::F("flare", geometry.flare.metres()),
            Word::F("taper_height", geometry.taper_height.metres()),
        ]);
        dispatch(
            self,
            batch,
            &collar_source(&format!(
                "{}\n{}\n{}\n{}",
                include_str!("device_collar_chart.wgsl"),
                include_str!("device_cloth_support.wgsl"),
                include_str!("device_collar_support.wgsl"),
                include_str!("device_partlet.wgsl")
            )),
            &[
                write("fit", buffers.fit),
                read_u32("samples", buffers.samples),
                read("row_heights", buffers.row_heights),
                read("seating", buffers.seating),
                atomic("collar_status", buffers.status),
                read("positions", &self.body.positions),
                read_u32("faces", &self.body.faces),
                read("joints", &self.body.joints),
            ],
            &words,
            Grid::Items(sheet.rows * collar.columns),
        )
    }
}

pub(crate) fn collar_source(shader: &str) -> String {
    format!(
        "const FIRST_POINT:u32={BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS:u32={BODY_CLOTH_POINT_WORDS}u;\nconst MAX_COLLAR_COLUMNS:u32={MAX_COLLAR_COLUMNS}u;\nconst SAMPLE_WORDS:u32={COLLAR_SAMPLE_WORDS}u;\n{shader}"
    )
}
