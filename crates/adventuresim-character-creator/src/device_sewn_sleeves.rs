//! Evaluate sewn sleeve volumes in canonical metres, without cloth simulation.
use anyhow::Result;
use fabelgeist_armor::gpu::{SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};
use fabelgeist_rig::RigJointPart;

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::sewn_sleeves::{SLEEVE_BODY_STATIONS, SLEEVE_ROWS, SewnSleevePlan};

impl DeviceWearer<'_> {
    pub(crate) fn record_sewn_sleeves(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SewnSleevePlan,
    ) -> Result<()> {
        let stations = self
            .gpu
            .upload(BufferUpload::from_elements(&SLEEVE_BODY_STATIONS))?;
        for seam in &plan.seams {
            let boundary = self
                .gpu
                .upload(BufferUpload::from_elements(&seam.boundary))?;
            let angles = self.gpu.upload(BufferUpload::from_elements(&seam.angles))?;
            let count = (seam.boundary.len() * SLEEVE_ROWS) as u32;
            let source = format!(
                "const FIRST_POINT:u32={SKIRT_FIT_WORDS}u;\nconst POINT_WORDS:u32={SKIRT_ATTACHMENT_POINT_WORDS}u;\nconst BODY_ROWS:u32={}u;\n{}",
                SLEEVE_BODY_STATIONS.len(),
                include_str!("device_sewn_sleeves.wgsl"),
            );
            dispatch(
                self,
                batch,
                &source,
                &[
                    write("fit", fit),
                    read("joints", &self.body.joints),
                    read_u32("boundary", &boundary),
                    read("angles", &angles),
                    read("stations", &stations),
                ],
                &[
                    Word::U("count", count),
                    Word::U("first", seam.first),
                    Word::U("columns", seam.boundary.len() as u32),
                    Word::Joint(
                        crate::rig_parameters::RigJointParameter::SleeveShoulder,
                        seam.side
                            .joint(RigJointPart::Uparm)
                            .require_in(self.host.joint_names)?,
                    ),
                    Word::Joint(
                        crate::rig_parameters::RigJointParameter::SleeveElbow,
                        seam.side
                            .joint(RigJointPart::Lowarm)
                            .require_in(self.host.joint_names)?,
                    ),
                    Word::Joint(
                        crate::rig_parameters::RigJointParameter::Wrist,
                        seam.side
                            .joint(RigJointPart::Wrist)
                            .require_in(self.host.joint_names)?,
                    ),
                    Word::F("reach", plan.design.length.unit()),
                    Word::F(
                        "radius",
                        plan.design.circumference.metres() / std::f32::consts::TAU,
                    ),
                    Word::F(
                        "cuff_radius",
                        plan.design.cuff_circumference.metres() / std::f32::consts::TAU,
                    ),
                    Word::F("cuff_length", plan.design.cuff_length.metres()),
                ],
                Grid::Items(count),
            )?;
        }
        Ok(())
    }
}
