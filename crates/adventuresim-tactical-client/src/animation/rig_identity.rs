//! Admit Bevy scene labels once, retaining exact identity and animation roles.

use super::{BoneRole, ClipLayer};
use bevy::prelude::*;
use fabelgeist_rig::{RigJointName, RigJointPart};

#[derive(Component, Clone, Debug)]
pub(crate) struct AuthoredRigJoint {
    pub(crate) name: RigJointName,
    pub(super) body_region: AnimationBodyRegion,
    pub(super) role: Option<BoneRole>,
    pub(super) pelvis: PelvisRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnimationBodyRegion {
    Upper,
    Lower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PelvisRole {
    Pelvis,
    Other,
}

impl AuthoredRigJoint {
    pub(crate) fn new(name: RigJointName) -> Self {
        let body_region = AnimationBodyRegion::from(&name);
        let role = BoneRole::from_name(&name);
        let pelvis = if name.eq_ignore_ascii_case(&RigJointName::ROOT) {
            PelvisRole::Pelvis
        } else {
            PelvisRole::Other
        };
        Self {
            name,
            body_region,
            role,
            pelvis,
        }
    }
}

impl From<&RigJointName> for AnimationBodyRegion {
    fn from(name: &RigJointName) -> Self {
        if [
            RigJointName::SKELETON,
            RigJointName::BODY_WORLD,
            RigJointName::ROOT,
        ]
        .iter()
        .any(|root| name.eq_ignore_ascii_case(root))
            || [
                RigJointPart::Upleg,
                RigJointPart::Lowleg,
                RigJointPart::Foot,
                RigJointPart::Talocrural,
                RigJointPart::Subtalar,
                RigJointPart::Transversetarsal,
                RigJointPart::Ball,
            ]
            .into_iter()
            .any(|part| {
                name.as_ref()
                    .to_ascii_lowercase()
                    .contains(&format!("_{part}"))
            })
        {
            Self::Lower
        } else {
            Self::Upper
        }
    }
}

impl AnimationBodyRegion {
    pub(super) fn includes(self, layer: ClipLayer, pelvis: PelvisRole) -> bool {
        match layer {
            ClipLayer::Whole => true,
            ClipLayer::Upper => self == Self::Upper,
            ClipLayer::Lower | ClipLayer::CombatLower => self == Self::Lower,
            ClipLayer::CombatUpper => self == Self::Upper || pelvis == PelvisRole::Pelvis,
            ClipLayer::MainHand | ClipLayer::Offhand => false,
        }
    }
}
