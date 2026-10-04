//! The clearance fit of long plates on the device.
//!
//! Thirteen stations along the plate each measure the convex section of the
//! skin near them; every carrier point then moves radially to clear the
//! section blended from its two nearest stations. A station's section is a
//! sort and a hull over a few hundred points, which one invocation per station
//! does; the carriers move one invocation each.

use anyhow::Result;
use fabelgeist_gpu::prelude::BufferUpload;

use fabelgeist_armor::{CuisseDesign, GreaveDesign, Millimeters, RerebraceDesign};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::PassParameterName;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};
use fabelgeist_rig::RigJointMembership;

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
            _ => (vec![RigJointMembership::Excluded; primary.len()], 0),
        };
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.body.vertex_count).into());
        parameters.insert("filtered".into(), (filtered).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("pad2".into(), (0u32).into());
        parameters.insert("positions".into(), (self.body.positions.clone()).into());
        parameters.insert(
            "joint_indices".into(),
            (self.body.joint_indices.clone()).into(),
        );
        parameters.insert(
            "joint_weights".into(),
            (self.body.joint_weights.clone()).into(),
        );
        parameters.insert(
            "primary".into(),
            (self.gpu.upload(BufferUpload::from_elements(
                &primary.into_iter().map(u32::from).collect::<Vec<_>>(),
            ))?)
            .into(),
        );
        parameters.insert(
            "extra".into(),
            (self.gpu.upload(BufferUpload::from_elements(
                &extra.into_iter().map(u32::from).collect::<Vec<_>>(),
            ))?)
            .into(),
        );
        parameters.insert("frames".into(), (frame.frame.clone()).into());
        parameters.insert("support".into(), (support.clone()).into());
        let kernel = self
            .gpu
            .cache()
            .get(self.gpu.context(), &ShaderSource::from(support_source()))
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (self.body.vertex_count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
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
        let support = gpu.scratch(
            (self.body.vertex_count as u64 * 4).into(),
            ("clearance support").into(),
        )?;
        self.record_support(batch, fit.region, frame, &support)?;
        let half_width = if matches!(fit.region, FitRegion::UpperArm(_) | FitRegion::LowerLeg(_)) {
            UPPER_ARM_STATION_HALF_WIDTH_M
        } else {
            STATION_HALF_WIDTH_M
        };
        let (style, words) = style_words(fit.style);
        let sections = gpu.scratch(
            (STATIONS as u64 * SECTION_WORDS as u64 * 4).into(),
            ("clearance sections").into(),
        )?;
        let scratch = gpu.scratch(
            (STATIONS as u64 * STATION_CAPACITY as u64 * 12).into(),
            ("clearance section samples").into(),
        )?;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.body.vertex_count).into());
        parameters.insert("carriers_count".into(), (fit.count).into());
        parameters.insert("cuff".into(), (fit.cuff).into());
        parameters.insert("style".into(), (style).into());
        parameters.insert(
            "hand".into(),
            (u32::from(matches!(fit.region, FitRegion::Hand(_)))).into(),
        );
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("pad2".into(), (0u32).into());
        parameters.insert("half_width".into(), (half_width).into());
        parameters.insert(
            "gap".into(),
            (fit.clearance + fit.thickness + FIT_MARGIN_M).into(),
        );
        parameters.insert("pad3".into(), (0.0f32).into());
        parameters.insert("pad4".into(), (0.0f32).into());
        for (i, word) in words.iter().enumerate() {
            parameters.insert(PassParameterName::from(format!("style{i}")), (*word).into());
        }
        parameters.insert("positions".into(), (self.body.positions.clone()).into());
        parameters.insert("support".into(), (support).into());
        parameters.insert("frames".into(), (frame.frame.clone()).into());
        parameters.insert("sections".into(), (sections).into());
        parameters.insert("samples".into(), (scratch).into());
        parameters.insert("carriers".into(), (carriers.clone()).into());
        parameters.insert("status".into(), (status.clone()).into());
        let compile = |entry: &str| {
            gpu.cache()
                .get(gpu.context(), &ShaderSource::from(fit_source(entry)))
                .map_err(fabelgeist_armor::GenerateError::from)
        };
        batch
            .dispatch(&*compile(SECTIONS)?, &parameters, ([STATIONS, 1, 1]).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&*compile(FIT)?, &parameters, (fit.count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(())
    }
}
