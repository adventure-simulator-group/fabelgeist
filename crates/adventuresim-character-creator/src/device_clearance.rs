//! The clearance fit of long plates on the device.
//!
//! Thirteen stations along the plate each measure the convex section of the
//! skin near them; every carrier point then moves radially to clear the
//! section blended from its two nearest stations. A station's section is a
//! sort and a hull over a few hundred points, which one invocation per station
//! does; the carriers move one invocation each.

use anyhow::Result;
use fabelgeist_armor::gpu::device_error;
use fabelgeist_armor::{CuisseDesign, GreaveDesign, Millimeters, RerebraceDesign};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::armor_frames::FitRegion;
use crate::device_frames::{DeviceFrame, DeviceWearer};

mod wgsl;
use wgsl::{FIT, SECTIONS, fit_source, support_source};

const STATIONS: u32 = 13;
const STATION_HALF_WIDTH_M: f32 = 0.024;
const UPPER_ARM_STATION_HALF_WIDTH_M: f32 = 0.012;
const FIT_MARGIN_M: f32 = 0.004;
/// Section samples a station may gather; beyond this the fit fails rather
/// than silently drop skin.
const STATION_CAPACITY: u32 = 8192;
/// Floats per measured section: centre, then 64 radii.
const SECTION_WORDS: u32 = 66;

/// The anatomical fit and style allowance of a plate, separate from any
/// embossed relief.
#[derive(Clone, Copy)]
pub enum PlateFit<'a> {
    Greave(&'a GreaveDesign),
    Cuisse(&'a CuisseDesign),
    Rerebrace(&'a RerebraceDesign),
    Mitten {
        cuff_length: f32,
        cuff_clearance: Millimeters,
    },
}

/// Style codes and floats the fit kernel reads.
fn style_words(style: PlateFit<'_>) -> (u32, [f32; 8]) {
    match style {
        PlateFit::Greave(d) => (0, greave_words(d)),
        PlateFit::Cuisse(d) => (1, [d.knee_taper.unit(), -1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        PlateFit::Rerebrace(d) => (
            2,
            [
                d.distal_taper.unit(),
                -1.0,
                1.0,
                d.section_depth.unit(),
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        ),
        PlateFit::Mitten {
            cuff_length,
            cuff_clearance,
        } => (
            3,
            [
                cuff_clearance.metres(),
                -1.0,
                0.94 + 2.0 * cuff_length,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        ),
    }
}

fn greave_words(d: &GreaveDesign) -> [f32; 8] {
    [
        d.calf_height.unit(),
        -1.0,
        1.0,
        d.ankle_taper.unit(),
        d.knee_taper.unit(),
        d.ankle_extension.metres(),
        0.0,
        0.0,
    ]
}

/// A clearance fit recorded against a part's carriers.
pub struct ClearanceFit<'a> {
    pub region: FitRegion,
    pub clearance: f32,
    pub thickness: f32,
    pub style: PlateFit<'a>,
    /// Carriers to fit: the first `count` of the arena. The first `cuff` of
    /// them take the mitten's extra cuff room.
    pub count: u32,
    pub cuff: u32,
}

impl DeviceWearer<'_> {
    /// Record which support lists name each body vertex: bit 0 for the
    /// region's own skin, bit 1 for the skin it extends into.
    fn record_support(
        &self,
        batch: &mut KernelBatch,
        region: FitRegion,
        frame: &DeviceFrame,
        support: &Buffer,
    ) -> Result<()> {
        let owned = |region: FitRegion| self.host.owned_joints(&region.owners());
        let primary = owned(region);
        let (extra, filtered) = match region {
            FitRegion::Hand(side) => (owned(FitRegion::Forearm(side)), 0u32),
            FitRegion::LowerLeg(side) => (owned(FitRegion::Foot(side)), 1),
            _ => (vec![0; primary.len()], 0),
        };
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.body.vertex_count);
        parameters.insert("filtered", filtered);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert("joint_indices", self.body.joint_indices.clone());
        parameters.insert("joint_weights", self.body.joint_weights.clone());
        parameters.insert("primary", self.gpu.upload(&primary)?);
        parameters.insert("extra", self.gpu.upload(&extra)?);
        parameters.insert("frames", frame.frame.clone());
        parameters.insert("support", support.clone());
        let kernel = self
            .gpu
            .cache()
            .get(self.gpu.context(), &support_source())
            .map_err(device_error)?;
        batch
            .dispatch_items(&kernel, &parameters, self.body.vertex_count)
            .map_err(device_error)?;
        Ok(())
    }

    /// Record the clearance fit of a part's carriers in `frame`.
    pub fn record_clearance_fit(
        &self,
        batch: &mut KernelBatch,
        frame: &DeviceFrame,
        fit: &ClearanceFit,
        carriers: &Buffer,
        status: &Buffer,
    ) -> Result<()> {
        let gpu = self.gpu;
        let support = gpu.scratch(self.body.vertex_count as u64 * 4, "clearance support")?;
        self.record_support(batch, fit.region, frame, &support)?;
        let half_width = if matches!(fit.region, FitRegion::UpperArm(_) | FitRegion::LowerLeg(_)) {
            UPPER_ARM_STATION_HALF_WIDTH_M
        } else {
            STATION_HALF_WIDTH_M
        };
        let (style, words) = style_words(fit.style);
        let sections = gpu.scratch(
            STATIONS as u64 * SECTION_WORDS as u64 * 4,
            "clearance sections",
        )?;
        let scratch = gpu.scratch(
            STATIONS as u64 * STATION_CAPACITY as u64 * 12,
            "clearance section samples",
        )?;
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.body.vertex_count);
        parameters.insert("carriers_count", fit.count);
        parameters.insert("cuff", fit.cuff);
        parameters.insert("style", style);
        parameters.insert("hand", u32::from(matches!(fit.region, FitRegion::Hand(_))));
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        parameters.insert("half_width", half_width);
        parameters.insert("gap", fit.clearance + fit.thickness + FIT_MARGIN_M);
        parameters.insert("pad3", 0.0f32);
        parameters.insert("pad4", 0.0f32);
        for (i, word) in words.iter().enumerate() {
            parameters.insert(format!("style{i}"), *word);
        }
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert("support", support);
        parameters.insert("frames", frame.frame.clone());
        parameters.insert("sections", sections);
        parameters.insert("samples", scratch);
        parameters.insert("carriers", carriers.clone());
        parameters.insert("status", status.clone());
        let compile = |entry: &str| {
            gpu.cache()
                .get(gpu.context(), &fit_source(entry))
                .map_err(device_error)
        };
        batch
            .dispatch(&*compile(SECTIONS)?, &parameters, [STATIONS, 1, 1])
            .map_err(device_error)?;
        batch
            .dispatch_items(&*compile(FIT)?, &parameters, fit.count)
            .map_err(device_error)?;
        Ok(())
    }
}
