//! Anatomical anchors shared by fitted upper cloth in unposed Y-up metres.
use crate::armor_frames::Wearer;
use crate::surface_cut::{Plane, dot};
use anyhow::{Context, Result};
use fabelgeist_armor::PartFrame;
use fabelgeist_rig::RigJointName;

/// Numerical construction boundary: joints are metres; x/z are the fitted
/// head frame's lateral/anterior axes, not animation ownership.
pub(crate) struct ClothTorsoFrame {
    pub waist: [f32; 3],
    pub neck: [f32; 3],
    pub shoulders: [[f32; 3]; 2],
    pub lateral: [f32; 3],
    pub anterior: [f32; 3],
}

impl ClothTorsoFrame {
    pub fn new(body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        Self::from_axes(body, head.axes[0], head.axes[2])
    }

    /// Native canonical lateral/anterior unit axes; landmarks come from this body.
    pub fn from_axes(
        body: &Wearer<'_>,
        lateral_axis: [f32; 3],
        anterior_axis: [f32; 3],
    ) -> Result<Self> {
        let joint = |name: RigJointName| -> Result<[f32; 3]> {
            let index = name.require_in(body.joint_names)?;
            let state = body
                .joints
                .get(index)
                .context("missing bodice joint state")?;
            Ok([state[0], state[1], state[2]])
        };
        let shoulders = [joint(RigJointName::R_UPARM)?, joint(RigJointName::L_UPARM)?];
        let span = std::array::from_fn(|axis| shoulders[1][axis] - shoulders[0][axis]);
        let lateral = if dot(span, lateral_axis) < 0. {
            lateral_axis.map(|v| -v)
        } else {
            lateral_axis
        };
        Ok(Self {
            waist: joint(RigJointName::ROOT)?,
            neck: joint(RigJointName::C_NECK)?,
            shoulders,
            lateral,
            anterior: anterior_axis,
        })
    }

    pub fn breadth(&self) -> f32 {
        dot(self.shoulders[1], self.lateral) - dot(self.shoulders[0], self.lateral)
    }

    pub fn depth_half(&self, side: f32) -> Plane {
        Plane {
            normal: self.anterior.map(|v| -side * v),
            offset: -side * dot(self.neck, self.anterior),
        }
    }
}
