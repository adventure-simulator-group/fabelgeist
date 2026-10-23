//! Cloth closure tapes follow their panel anchors and the current torso surface.
use anyhow::Result;
use fabelgeist_armor::PartletDesign;
use fabelgeist_armor::gpu::{BODY_CLOTH_FIRST_POINT, BODY_CLOTH_POINT_WORDS};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};
use fabelgeist_rig::RigJointName;

use crate::armor_frames::FitRegion;
use crate::cloth_tie_plan::{TIE_POINT_WORDS, TiePath};
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, dispatch, read, read_u32, write};
use crate::device_textile_support::TextileSupport;
use crate::partlet_plan::PartletPlan;

impl DeviceWearer<'_> {
    pub(crate) fn record_partlet_ties(
        &self,
        batch: &mut KernelBatch,
        design: &PartletDesign,
        plan: &PartletPlan,
        fit: &Buffer,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
    ) -> Result<()> {
        let (Some(ties), Some(geometry)) = (&plan.ties, &design.ties) else {
            return Ok(());
        };
        let points = self.gpu.upload(BufferUpload::from_elements(&ties.words))?;
        let support = TextileSupport::new(self.host, &[FitRegion::Torso, FitRegion::Hips], layers)?;
        let support_positions = self
            .gpu
            .upload(BufferUpload::from_elements(&support.positions))?;
        let faces = self
            .gpu
            .upload(BufferUpload::from_elements(&support.faces))?;
        let mut source = format!(
            "const FIRST_POINT:u32={BODY_CLOTH_FIRST_POINT}u;\nconst POINT_WORDS:u32={BODY_CLOTH_POINT_WORDS}u;\nconst TIE_WORDS:u32={TIE_POINT_WORDS}u;\n"
        );
        for (name, path) in [
            ("FRONT_BAND", TiePath::FrontBand),
            ("BACK_BAND", TiePath::BackBand),
            ("LEFT_LOOP", TiePath::LeftLoop),
            ("RIGHT_LOOP", TiePath::RightLoop),
            ("LEFT_TAIL", TiePath::LeftTail),
            ("RIGHT_TAIL", TiePath::RightTail),
            ("KNOT", TiePath::Knot),
        ] {
            source.push_str(&format!("const {name}:u32={}u;\n", path as u32));
        }
        source.push_str(include_str!("device_cloth_support.wgsl"));
        source.push_str(include_str!("device_cloth_ties.wgsl"));
        let count = ties.words.len() as u32 / TIE_POINT_WORDS;
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read_u32("tape", &points),
                read("positions", &support_positions),
                read_u32("faces", &faces),
                read("joints", &self.body.joints),
            ],
            &[
                Word::U("first", ties.first),
                Word::U("count", count),
                Word::U("face_count", support.faces.len() as u32),
                Word::Joint(
                    crate::rig_parameters::RigJointParameter::Neck,
                    RigJointName::C_NECK.require_in(self.host.joint_names)?,
                ),
                Word::F("width", geometry.width.metres()),
                Word::F("tail", geometry.tail_length.metres()),
                Word::F("bow", geometry.bow_span.metres()),
                Word::F("gauge", geometry.thickness.metres()),
                Word::F("placement", plan.cloth.layers[0].offset_metres),
            ],
            Grid::Items(count),
        )
    }
}
