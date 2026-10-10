//! Mixamo as a source rig.
//!
//! This is a data file, not a feature. It names Mixamo's joints and says which
//! humanoid role each one plays; the retargeter never sees any of these
//! strings. Supporting another rig means writing a sibling of this file, not
//! touching the algorithm.

use crate::animation::retarget::profile::{
    ChainBinding, JointRequirement, ReferencePose, RigProfile, RootSource,
};
use crate::animation::retarget::semantic::{HumanoidChain, HumanoidJoint};
use crate::skeleton::mixamo::MixamoRig;

const JOINT_MAPPINGS: [JointMapping; 22] = [
    JointMapping {
        role: HumanoidJoint::Pelvis,
        source_name: "Hips",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::SpineLower,
        source_name: "Spine",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::SpineMid,
        source_name: "Spine1",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::Chest,
        source_name: "Spine2",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::Neck,
        source_name: "Neck",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::Head,
        source_name: "Head",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::ClavicleLeft,
        source_name: "LeftShoulder",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::UpperArmLeft,
        source_name: "LeftArm",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::LowerArmLeft,
        source_name: "LeftForeArm",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::HandLeft,
        source_name: "LeftHand",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::ClavicleRight,
        source_name: "RightShoulder",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::UpperArmRight,
        source_name: "RightArm",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::LowerArmRight,
        source_name: "RightForeArm",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::HandRight,
        source_name: "RightHand",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::UpperLegLeft,
        source_name: "LeftUpLeg",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::LowerLegLeft,
        source_name: "LeftLeg",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::FootLeft,
        source_name: "LeftFoot",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::ToeLeft,
        source_name: "LeftToeBase",
        requirement: JointRequirement::Optional,
    },
    JointMapping {
        role: HumanoidJoint::UpperLegRight,
        source_name: "RightUpLeg",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::LowerLegRight,
        source_name: "RightLeg",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::FootRight,
        source_name: "RightFoot",
        requirement: JointRequirement::Required,
    },
    JointMapping {
        role: HumanoidJoint::ToeRight,
        source_name: "RightToeBase",
        requirement: JointRequirement::Optional,
    },
];

struct JointMapping {
    role: HumanoidJoint,
    source_name: &'static str,
    requirement: JointRequirement,
}

impl MixamoRig {
    /// The Mixamo rig described in humanoid terms.
    ///
    /// Mixamo keeps locomotion on the hips — there is no separate root joint —
    /// which is exactly the sort of per-rig convention a profile exists to
    /// record.
    pub fn profile() -> RigProfile {
        // Mixamo binds in a T-pose, so straightening is very nearly a no-op —
        // but saying so is what lets a target rig that binds differently
        // measure its motion against the same posture.
        let mut profile = RigProfile::new("Mixamo")
            .with_root(RootSource::Pelvis)
            .with_reference(ReferencePose::TPose)
            .with_markers([joint("Hips"), joint("Spine"), joint("LeftUpLeg")]);

        for mapping in JOINT_MAPPINGS {
            profile = match mapping.requirement {
                JointRequirement::Required => {
                    profile.with_required(mapping.role, joint(mapping.source_name))
                }
                JointRequirement::Optional => {
                    profile.with(mapping.role, joint(mapping.source_name))
                }
            };
        }

        // Fingers are optional everywhere: a rig without them still retargets
        // its body, and a body-only clip is a perfectly good clip. Mixamo
        // calls the little finger "Pinky".
        for side in ["Left", "Right"] {
            for finger in ["Thumb", "Index", "Middle", "Ring", "Pinky"] {
                for (segment, role) in finger_roles(side, finger).into_iter().enumerate() {
                    profile =
                        profile.with(role, joint(&format!("{side}Hand{finger}{}", segment + 1)));
                }
            }
        }

        // Mixamo's spine is three joints; rigs it is retargeted onto rarely
        // agree. Declaring the chain lets motion be spread over however many
        // the target has.
        profile.with_chain(
            HumanoidChain::Spine,
            ChainBinding::new([joint("Spine"), joint("Spine1"), joint("Spine2")]),
        )
    }
}

/// Mixamo prefixes every joint; exports occasionally use `mixamorig1:` and
/// some pipelines strip the namespace entirely. The resolver normalizes
/// namespaces away, so binding the canonical name covers all three.
fn joint(name: &str) -> String {
    format!("mixamorig:{name}")
}

/// The three humanoid segments of one finger.
fn finger_roles(side: &str, finger: &str) -> [HumanoidJoint; 3] {
    use HumanoidJoint::*;
    match (side, finger) {
        ("Left", "Thumb") => [ThumbProximalLeft, ThumbIntermediateLeft, ThumbDistalLeft],
        ("Left", "Index") => [IndexProximalLeft, IndexIntermediateLeft, IndexDistalLeft],
        ("Left", "Middle") => [MiddleProximalLeft, MiddleIntermediateLeft, MiddleDistalLeft],
        ("Left", "Ring") => [RingProximalLeft, RingIntermediateLeft, RingDistalLeft],
        ("Left", _) => [LittleProximalLeft, LittleIntermediateLeft, LittleDistalLeft],
        (_, "Thumb") => [ThumbProximalRight, ThumbIntermediateRight, ThumbDistalRight],
        (_, "Index") => [IndexProximalRight, IndexIntermediateRight, IndexDistalRight],
        (_, "Middle") => [
            MiddleProximalRight,
            MiddleIntermediateRight,
            MiddleDistalRight,
        ],
        (_, "Ring") => [RingProximalRight, RingIntermediateRight, RingDistalRight],
        (_, _) => [
            LittleProximalRight,
            LittleIntermediateRight,
            LittleDistalRight,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_profile_resolves_against_the_mixamo_skeleton() {
        let skeleton = MixamoRig::skeleton();
        let resolved = MixamoRig::profile()
            .resolve(&skeleton)
            .expect("the Mixamo profile must resolve against the Mixamo rig");

        assert_eq!(
            resolved.joint(HumanoidJoint::Pelvis),
            skeleton.find_joint_by_name("mixamorig:Hips")
        );
        assert_eq!(
            resolved.joint(HumanoidJoint::LowerArmRight),
            skeleton.find_joint_by_name("mixamorig:RightForeArm")
        );
        assert_eq!(
            resolved.joint(HumanoidJoint::IndexDistalLeft),
            skeleton.find_joint_by_name("mixamorig:LeftHandIndex3")
        );
        assert!(
            resolved.missing.is_empty(),
            "unmapped: {:?}",
            resolved.missing
        );
        assert_eq!(resolved.chains[&HumanoidChain::Spine].len(), 3);
    }

    #[test]
    fn detection_is_by_marker_joints() {
        assert!(MixamoRig::profile().matches(&MixamoRig::skeleton()));
    }
}
