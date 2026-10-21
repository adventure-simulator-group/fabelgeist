//! The wearer's forearm, as a bracer is fitted to it, and the body
//! realizations every fitted piece follows.

use fabelgeist_rig::{RigJointMembership, RigJointName, RigJointOrdinal};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForearmSide {
    Left,
    Right,
}

impl ForearmSide {
    pub(crate) fn lowarm(self) -> RigJointName {
        match self {
            Self::Left => RigJointName::L_LOWARM,
            Self::Right => RigJointName::R_LOWARM,
        }
    }

    pub(crate) fn wrist(self) -> RigJointName {
        match self {
            Self::Left => RigJointName::L_WRIST,
            Self::Right => RigJointName::R_WRIST,
        }
    }

    fn upperarm(self) -> RigJointName {
        match self {
            Self::Left => RigJointName::L_UPPERARM,
            Self::Right => RigJointName::R_UPPERARM,
        }
    }

    fn owns_forearm_joint(self, name: &RigJointName) -> RigJointMembership {
        crate::clothing::rig_selection::skin_family(name, &self.lowarm())
    }

    pub(crate) fn supports_forearm_boundary(self, name: &RigJointName) -> RigJointMembership {
        RigJointMembership::from(
            self.owns_forearm_joint(name) == RigJointMembership::Included
                || name == &self.wrist()
                || crate::clothing::rig_selection::skin_family(name, &self.upperarm())
                    == RigJointMembership::Included,
        )
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
    pub joint_indices: &'a fabelgeist_rig::PackedRigSkin<8>,
    pub joint_weights: &'a [[f32; 8]],
    pub joint_names: &'a fabelgeist_rig::RigJointTable<RigJointName>,
    pub global_joint_states: &'a fabelgeist_rig::RigJointTable<[f32; 8]>,
    pub morphs: &'a [ForearmMorphSample],
}

pub struct ForearmMorphSample {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Global rig state fitted for this exact body realization.  Garment
    /// semantic curves must be evaluated from these landmarks rather than
    /// translating neutral landmarks by a nearest-envelope approximation.
    pub global_joint_states: fabelgeist_rig::RigJointTable<[f32; 8]>,
    /// This realization on the armor device, once a piece has uploaded it.
    pub device: crate::device_body::DeviceBody,
}

#[derive(Debug, thiserror::Error)]
pub enum ForearmInputError {
    #[error("forearm surface inputs are inconsistent")]
    InconsistentSurface,
    #[error("MHR rig has no {owner} skin joints")]
    MissingSkinFamily { owner: RigJointName },
}

impl ForearmSurfaceInput<'_> {
    pub(crate) fn validate(&self) -> Result<(), ForearmInputError> {
        let vertices = self.positions.len();
        if self.domain.trim().is_empty()
            || self.normals.len() != vertices
            || self.joint_indices.len() != vertices
            || self.joint_weights.len() != vertices
            || self.faces.len() != self.texcoord_faces.len()
            || self.joint_names.len() != self.global_joint_states.len()
            || self.joint_indices.joint_count() != self.joint_names.joint_count()
            || self
                .morphs
                .iter()
                .any(|morph| morph.positions.len() != vertices
                    || morph.normals.len() != vertices
                    || morph.global_joint_states.len() != self.joint_names.len())
        {
            return Err(ForearmInputError::InconsistentSurface);
        }
        let forearm_joints = self
            .joint_names
            .indexed()
            .filter_map(
                |(index, name): (RigJointOrdinal, &RigJointName)| -> Option<RigJointOrdinal> {
                    (self.side.owns_forearm_joint(name) == RigJointMembership::Included)
                        .then_some(index)
                },
            )
            .collect::<BTreeSet<_>>();
        if forearm_joints.is_empty() {
            return Err(ForearmInputError::MissingSkinFamily {
                owner: self.side.lowarm(),
            });
        }
        Ok(())
    }
}
