//! Evaluate sewn sleeve volumes in canonical metres, without cloth simulation.
use anyhow::Result;
use fabelgeist_armor::gpu::{SKIRT_ATTACHMENT_POINT_WORDS, SKIRT_FIT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};
use fabelgeist_rig::RigJointPart;

use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use crate::sewn_sleeves::{SLEEVE_BODY_STATIONS, SewnSleevePlan};

impl DeviceWearer<'_> {
    pub(crate) fn record_sewn_sleeves(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        plan: &SewnSleevePlan,
        status: &Buffer,
    ) -> Result<()> {
        let stations = self
            .gpu
            .upload(BufferUpload::from_elements(&SLEEVE_BODY_STATIONS))?;
        for seam in &plan.seams {
            let boundary = self
                .gpu
                .upload(BufferUpload::from_elements(&seam.boundary))?;
            let pattern = self.gpu.upload(BufferUpload::from_elements(&seam.table))?;
            let grid = self.gpu.upload(BufferUpload::from_elements(&seam.grid))?;
            let count = (seam.table.len() / crate::surface_cut::POINT_WORDS) as u32;
            let joint = |part| seam.side.joint(part).require_in(self.host.joint_names);
            let frill = plan.design.frill.as_ref();
            let folds = frill.and_then(|f| f.folds.as_ref());
            let source = format!(
                "const FIRST_POINT:u32={SKIRT_FIT_WORDS}u;\nconst POINT_WORDS:u32={SKIRT_ATTACHMENT_POINT_WORDS}u;\nconst BODY_ROWS:u32={}u;\nconst SOURCE_WORDS:u32={}u;\nconst GRID_WORDS:u32={}u;\n{}",
                SLEEVE_BODY_STATIONS.len(),
                crate::surface_cut::POINT_WORDS,
                crate::sewn_sleeve_pattern::SLEEVE_GRID_WORDS,
                include_str!("device_sewn_sleeves.wgsl"),
            );
            dispatch(
                self,
                batch,
                &source,
                &[
                    write("fit", fit),
                    atomic("sleeve_status", status),
                    read("joints", &self.body.joints),
                    read_u32("boundary", &boundary),
                    read_u32("pattern", &pattern),
                    read_u32("grid", &grid),
                    read("stations", &stations),
                ],
                &[
                    Word::U("count", count),
                    Word::U("first", seam.first),
                    Word::Joint("shoulder", joint(RigJointPart::Uparm)?),
                    Word::Joint("elbow", joint(RigJointPart::Lowarm)?),
                    Word::Joint("wrist", joint(RigJointPart::Wrist)?),
                    Word::F("reach", plan.design.length.unit()),
                    Word::F("proximal_fullness", plan.design.proximal_fullness.unit()),
                    Word::F("fullness_release", plan.design.fullness_release.unit()),
                    Word::F(
                        "radius",
                        plan.design.circumference.metres() / std::f32::consts::TAU,
                    ),
                    Word::F(
                        "cuff_radius",
                        plan.design.cuff_circumference.metres() / std::f32::consts::TAU,
                    ),
                    Word::F("cuff_length", plan.design.cuff_length.metres()),
                    Word::F("frill_length", frill.map_or(0., |f| f.length.metres())),
                    Word::F("frill_flare", frill.map_or(0., |f| f.flare.metres())),
                    Word::F("fold_count", folds.map_or(0., |f| f32::from(f.count.0))),
                    Word::F("fold_depth", folds.map_or(0., |f| f.depth.metres())),
                    Word::U("fold_profile", folds.map_or(0, |f| f.profile as u32)),
                    Word::F(
                        "fold_mouth",
                        fabelgeist_armor::ClothPleatProfile::MOUTH_SHARE.unit(),
                    ),
                    Word::F(
                        "fold_valley",
                        fabelgeist_armor::ClothPleatProfile::VALLEY_DEPTH_SHARE.unit(),
                    ),
                ],
                Grid::Items(count),
            )?;
        }
        Ok(())
    }
}
