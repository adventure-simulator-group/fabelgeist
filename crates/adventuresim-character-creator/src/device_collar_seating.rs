//! Fit one smooth oval seat; retain local body support below the stand.
use crate::device_cloth_collar::{CollarBuffers, collar_source};
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::partlet_collar::CollarPlan;
use anyhow::Result;
use fabelgeist_armor::{CollarSection, StandingClothCollar};
use fabelgeist_compute::KernelBatch;
use fabelgeist_rig::RigJointName;

pub(crate) const SEAT_COEFFICIENT_WORDS: u32 = 5;

impl DeviceWearer<'_> {
    pub(crate) fn record_collar_seating(
        &self,
        batch: &mut KernelBatch,
        buffers: &CollarBuffers<'_>,
        collar: &CollarPlan,
        geometry: &StandingClothCollar,
    ) -> Result<()> {
        if !matches!(geometry.section, CollarSection::Oval { .. }) {
            return Ok(());
        }
        let words = self.collar_geometry_words(collar, geometry)?;
        dispatch(
            self,
            batch,
            &collar_source(&format!(
                "{}\n{}\n{}",
                include_str!("device_collar_chart.wgsl"),
                include_str!("device_collar_support.wgsl"),
                include_str!("device_collar_seat.wgsl")
            )),
            &[
                write("fit", buffers.fit),
                read_u32("samples", buffers.samples),
                write("seating", buffers.seating),
                atomic("collar_status", buffers.status),
                read("positions", &self.body.positions),
                read_u32("faces", &self.body.faces),
                read("joints", &self.body.joints),
            ],
            &words,
            Grid::Items(collar.columns),
        )?;
        dispatch(
            self,
            batch,
            &format!(
                "const SEAT_TERMS:u32={SEAT_COEFFICIENT_WORDS}u;\n{}",
                include_str!("device_collar_plane.wgsl")
            ),
            &[
                read_u32("samples", buffers.samples),
                write("seating", buffers.seating),
                write("fit", buffers.fit),
            ],
            &[
                Word::U("columns", collar.columns),
                Word::U("sample_words", crate::partlet_collar::COLLAR_SAMPLE_WORDS),
            ],
            Grid::Singles(1),
        )?;
        Ok(())
    }

    pub(crate) fn collar_geometry_words(
        &self,
        collar: &CollarPlan,
        geometry: &StandingClothCollar,
    ) -> Result<Vec<Word>> {
        Ok(vec![
            Word::U("columns", collar.columns),
            Word::F("top_scale", geometry.top_scale.unit()),
            Word::F("full_height", geometry.height.metres()),
            Word::Joint(
                crate::rig_parameters::RigJointParameter::CollarSeatTop,
                RigJointName::C_HEAD.require_in(self.host.joint_names)?,
            ),
            Word::Joint(
                crate::rig_parameters::RigJointParameter::CollarSeatBottom,
                RigJointName::ROOT.require_in(self.host.joint_names)?,
            ),
            Word::Joint(
                crate::rig_parameters::RigJointParameter::Neck,
                RigJointName::C_NECK.require_in(self.host.joint_names)?,
            ),
            Word::F(
                "oval_depth",
                match geometry.section {
                    CollarSection::Fitted => 0.,
                    CollarSection::Oval { depth_to_width } => depth_to_width.unit(),
                },
            ),
            Word::F("oval_length", collar.oval_length),
            Word::F("lateral_x", collar.lateral[0]),
            Word::F("lateral_z", collar.lateral[2]),
            Word::F("anterior_x", collar.anterior[0]),
            Word::F("anterior_z", collar.anterior[2]),
            Word::U("face_count", self.body.face_count),
        ])
    }
}
