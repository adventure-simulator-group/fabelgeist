//! The wearer's forearm, as a bracer is fitted to it, and the body
//! realizations every fitted piece follows.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForearmSide {
    Left,
    Right,
}

impl ForearmSide {
    pub(crate) fn lowarm(self) -> &'static str {
        match self {
            Self::Left => "l_lowarm",
            Self::Right => "r_lowarm",
        }
    }

    pub(crate) fn wrist(self) -> &'static str {
        match self {
            Self::Left => "l_wrist",
            Self::Right => "r_wrist",
        }
    }

    fn upperarm(self) -> &'static str {
        match self {
            Self::Left => "l_upperarm",
            Self::Right => "r_upperarm",
        }
    }

    fn owns_forearm_joint(self, name: &str) -> bool {
        name == self.lowarm() || name.starts_with(&format!("{}_twist", self.lowarm()))
    }

    pub(crate) fn supports_forearm_boundary(self, name: &str) -> bool {
        self.owns_forearm_joint(name)
            || name == self.wrist()
            || name == self.upperarm()
            || name.starts_with(&format!("{}_twist", self.upperarm()))
    }
}

pub struct ForearmSurfaceInput<'a> {
    pub domain: &'a str,
    pub side: ForearmSide,
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
    pub texcoords: &'a [[f32; 2]],
    pub texcoord_faces: &'a [[u32; 3]],
    pub joint_indices: &'a [[u32; 8]],
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a [String],
    pub global_joint_states: &'a [[f32; 8]],
    pub morphs: &'a [ForearmMorphSample],
}

pub struct ForearmMorphSample {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Global rig state fitted for this exact body realization.  Garment
    /// semantic curves must be evaluated from these landmarks rather than
    /// translating neutral landmarks by a nearest-envelope approximation.
    pub global_joint_states: Vec<[f32; 8]>,
    /// This realization on the armor device, once a piece has uploaded it.
    pub device: crate::device_body::DeviceBody,
}

impl ForearmSurfaceInput<'_> {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let vertices = self.positions.len();
        if self.domain.trim().is_empty()
            || self.normals.len() != vertices
            || self.joint_indices.len() != vertices
            || self.joint_weights.len() != vertices
            || self.faces.len() != self.texcoord_faces.len()
            || self.joint_names.len() != self.global_joint_states.len()
            || self
                .morphs
                .iter()
                .any(|morph| morph.positions.len() != vertices || morph.normals.len() != vertices)
        {
            return Err("forearm surface inputs are inconsistent".into());
        }
        let forearm_joints = self
            .joint_names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| self.side.owns_forearm_joint(name).then_some(index))
            .collect::<BTreeSet<_>>();
        if forearm_joints.is_empty() {
            return Err(format!("MHR rig has no {} skin joints", self.side.lowarm()));
        }
        Ok(())
    }
}
