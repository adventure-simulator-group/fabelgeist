//! Sewn collar and separate smooth bands share fitted neckline correspondence.
use anyhow::Result;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS};
use fabelgeist_armor::{PartletDesign, StandingClothCollar};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::partlet_collar::{CollarPlan, MAX_COLLAR_COLUMNS};
use crate::partlet_plan::PartletPlan;

/// Numerical kernel boundary: dimensions in canonical metres, native ordinals.
struct CollarSheet {
    first: u32,
    rows: u32,
    first_row: u32,
    bottom: f32,
    height: f32,
    pleats: f32,
    depth: f32,
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
        let sheet = CollarSheet {
            first: plan.cloth.surface.points.len() as u32,
            rows: collar.rows,
            first_row: 1,
            bottom: 0.,
            height: geometry.height.metres(),
            pleats: geometry
                .pleats
                .as_ref()
                .map_or(0., |p| f32::from(p.count.0)),
            depth: geometry.pleats.as_ref().map_or(0., |p| p.depth.metres()),
        };
        self.record_collar_sheet(batch, fit, &samples, collar, geometry, &sheet)?;
        for band in &plan.bands {
            let sheet = CollarSheet {
                first: band.first,
                rows: band.rows,
                first_row: 0,
                bottom: band.bottom,
                height: band.height,
                pleats: 0.,
                depth: 0.,
            };
            self.record_collar_sheet(batch, fit, &samples, collar, geometry, &sheet)?;
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
        if let Some(pleats) = &geometry.pleats {
            dispatch(
                self,
                batch,
                &collar_source(include_str!("device_collar_check.wgsl")),
                &[write("fit", fit), read_u32("samples", &samples)],
                &[
                    Word::U("columns", collar.columns),
                    Word::U("pleats", u32::from(pleats.count.0)),
                    Word::F("depth", pleats.depth.metres()),
                    Word::F("thickness", design.thickness.metres()),
                    Word::F("top_scale", geometry.top_scale.unit()),
                ],
                Grid::Singles(1),
            )?;
        }
        Ok(())
    }

    fn record_collar_sheet(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        samples: &Buffer,
        collar: &CollarPlan,
        geometry: &StandingClothCollar,
        sheet: &CollarSheet,
    ) -> Result<()> {
        dispatch(
            self,
            batch,
            &collar_source(include_str!("device_partlet.wgsl")),
            &[
                write("fit", fit),
                read_u32("samples", samples),
                read("positions", &self.body.positions),
                read_u32("faces", &self.body.faces),
            ],
            &[
                Word::U("first", sheet.first),
                Word::U("columns", collar.columns),
                Word::U("rows", sheet.rows),
                Word::U("first_row", sheet.first_row),
                Word::F("bottom", sheet.bottom),
                Word::F("height", sheet.height),
                Word::F("full_height", geometry.height.metres()),
                Word::F("top_scale", geometry.top_scale.unit()),
                Word::U("face_count", self.body.face_count),
                Word::F("pleats", sheet.pleats),
                Word::F("depth", sheet.depth),
            ],
            Grid::Items(sheet.rows * collar.columns),
        )
    }
}

fn collar_source(shader: &str) -> String {
    format!(
        "const FIRST_POINT:u32={BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS:u32={BODY_CLOTH_POINT_WORDS}u;\nconst MAX_COLLAR_COLUMNS:u32={MAX_COLLAR_COLUMNS}u;\n{shader}"
    )
}
