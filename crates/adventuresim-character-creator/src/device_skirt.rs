//! A skirt fitted around both legs and the actual worn inner layers.
//! All geometry is evaluated in the canonical unposed Y-up fitting frame.

use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::{
    SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS, SKIRT_PROFILE_FIRST_WORD, SKIRT_SECTION_WORDS,
    record_skirt,
};
use fabelgeist_armor::{SKIRT_AXIAL_SEGMENTS, SKIRT_RING_SEGMENTS, SkirtDesign};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_rig::RigJointName;

use crate::armor_frames::{FitRegion, Side};
use crate::armor_layer::ArmorLayerSurface;
use crate::device_frames::DeviceWearer;
use crate::device_garment::{JointHeightAnchor, JointReachTarget, UprightSpan};
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::device_piece::{DeviceCheck, DeviceRecording};
use crate::device_textile_support::TextileSupport;
use crate::skirt_join::SkirtJoinPlan;

/// Each body triangle contributes its corners and two plane-edge crossings.
const SAMPLES_PER_TRIANGLE: u32 = 9;
const SECTION_HALF_WIDTH_M: f32 = 0.014;
const PROFILE_STATUS: &str = r#"
@compute @workgroup_size(1)
fn main() { atomicOr(&status[0], atomicLoad(&section_status[0])); }
"#;

impl DeviceWearer<'_> {
    pub(crate) fn record_fitted_skirt_textile(
        &self,
        batch: &mut KernelBatch,
        design: &SkirtDesign,
        layers: &[ArmorLayerSurface<'_>],
        attachment: Option<&SkirtJoinPlan>,
    ) -> Result<DeviceRecording> {
        design.validate()?;
        let hips = self.record_frame(batch, FitRegion::Hips)?;
        let head = self.record_frame(batch, FitRegion::Head)?;
        let fit_words = SKIRT_FIT_WORDS
            + attachment.map_or(0, |plan| {
                (plan.patch.point_count + SKIRT_RING_SEGMENTS as u32) * SKIRT_ATTACHMENT_POINT_WORDS
            });
        let fit = self.record_upright(
            batch,
            fit_words,
            &hips,
            &head,
            UprightSpan {
                top: JointHeightAnchor {
                    joint: RigJointName::ROOT,
                    offset: design.waist_rise.signed_length(),
                },
                bottom: JointHeightAnchor {
                    joint: RigJointName::ROOT,
                    offset: fabelgeist_geometry::SignedLength::from_millimetres(0),
                },
                reach: JointReachTarget {
                    joint: RigJointName::L_FOOT,
                    coefficient: fabelgeist_math::units::Ratio::from_ratio(design.length.unit()),
                },
            },
        )?;
        let support = TextileSupport::new(
            self.host,
            &[
                FitRegion::Hips,
                FitRegion::Torso,
                FitRegion::WholeLeg(Side::Left),
                FitRegion::WholeLeg(Side::Right),
                FitRegion::Foot(Side::Left),
                FitRegion::Foot(Side::Right),
            ],
            layers,
        )?;
        ensure!(!support.faces.is_empty(), "skirt has no body support");
        let status = self.record_skirt_profile(batch, design, &fit, &support)?;
        if let Some(plan) = attachment {
            self.record_skirt_attachment(
                batch,
                &fit,
                plan,
                design.clearance.metres() + design.thickness.metres(),
            )?;
        }
        // The profile must be written before the coordinate kernel reads it.
        let part = record_skirt(
            self.gpu,
            batch,
            design,
            &fit,
            attachment.map(|plan| &plan.patch),
        )?;
        dispatch(
            self,
            batch,
            PROFILE_STATUS,
            &[
                atomic("section_status", &status),
                atomic("status", part.status()),
            ],
            &[],
            Grid::Singles(1),
        )?;
        self.record_fit_status(batch, &fit, &part)?;
        let clearance = design.clearance.metres();
        let checks = if attachment.is_none() && design.support.is_none() && design.pleats.is_none()
        {
            vec![DeviceCheck::Mesh(Box::new(move |part| {
                crate::skirt_clearance::validate(part, &support, clearance)
            }))]
        } else {
            Vec::new()
        };
        Ok(DeviceRecording {
            part,
            frames: vec![(hips, FitRegion::Hips), (head, FitRegion::Head)],
            checks,
        })
    }
    /// Record triangle-band hulls into the skirt's shared fitted profile.
    fn record_skirt_profile(
        &self,
        batch: &mut KernelBatch,
        design: &SkirtDesign,
        fit: &fabelgeist_gpu::prelude::Buffer,
        support: &TextileSupport,
    ) -> Result<fabelgeist_gpu::prelude::Buffer> {
        let gpu = self.gpu;
        let face_count = u32::try_from(support.faces.len())?;
        let capacity = face_count
            .checked_mul(SAMPLES_PER_TRIANGLE)
            // One carrier-axis sample and one hull construction endpoint.
            .and_then(|v| v.checked_add(2))
            .ok_or_else(|| anyhow::anyhow!("skirt section capacity overflow"))?;
        let sections = SKIRT_AXIAL_SEGMENTS as u32 + 1;
        let positions = gpu.upload(BufferUpload::from_elements(&support.positions))?;
        let faces = gpu.upload(BufferUpload::from_elements(&support.faces))?;
        let bytes = u64::from(capacity) * u64::from(sections) * 8;
        let samples = gpu.scratch(bytes, "skirt section samples")?;
        let hulls = gpu.scratch(bytes, "skirt section hulls")?;
        let status = gpu.upload(BufferUpload::from_elements(&[0u32]))?;
        let source = format!(
            "const SECTIONS: u32 = {sections}u;\nconst RADII: u32 = {SKIRT_RING_SEGMENTS}u;\nconst FIRST_SECTION: u32 = {SKIRT_PROFILE_FIRST_WORD}u;\nconst SECTION_WORDS: u32 = {SKIRT_SECTION_WORDS}u;\nconst HALF_WIDTH: f32 = {SECTION_HALF_WIDTH_M};\n{}\n{}",
            include_str!("device_section_hull.wgsl"),
            include_str!("device_skirt_sections.wgsl"),
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read("positions", &positions),
                read_u32("faces", &faces),
                write("samples", &samples),
                write("hulls", &hulls),
                atomic("status", &status),
            ],
            &[
                Word::U("capacity", capacity),
                Word::U("face_count", face_count),
                Word::F(
                    "allowance",
                    design.clearance.metres() + design.thickness.metres(),
                ),
            ],
            Grid::Singles(sections),
        )?;
        Ok(status)
    }
}
