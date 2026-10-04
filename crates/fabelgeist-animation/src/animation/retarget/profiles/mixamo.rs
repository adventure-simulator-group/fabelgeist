//! Mixamo as a source rig.
//!
//! This is a data file, not a feature. It names Mixamo's joints and says which
//! humanoid role each one plays; the retargeter never sees any of these
//! strings. Supporting another rig means writing a sibling of this file, not
//! touching the algorithm.

use super::super::JointRequirement;
use super::mixamo_finger::MixamoFinger;
use crate::animation::retarget::profile::{ChainBinding, ReferencePose, RigProfile, RootSource};
use crate::animation::retarget::semantic::{HumanoidChain, HumanoidJoint};
use crate::skeleton::mixamo::MixamoRig;
use fabelgeist_rig::{RigJointName, RigJointPrefix, RigSide};

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
        let mut profile = RigProfile::new("Mixamo".into())
            .with_root(RootSource::Pelvis)
            .with_reference(ReferencePose::TPose)
            .with_markers([
                RigJointPrefix::MIXAMO.apply(&"Hips".into()),
                RigJointPrefix::MIXAMO.apply(&"Spine".into()),
                RigJointPrefix::MIXAMO.apply(&"LeftUpLeg".into()),
            ]);

        for (role, name, required) in Self::BODY_BINDINGS {
            profile = if *required == JointRequirement::Required {
                profile.with_required(*role, RigJointPrefix::MIXAMO.apply(name))
            } else {
                profile.with(*role, RigJointPrefix::MIXAMO.apply(name))
            };
        }

        // Fingers are optional everywhere: a rig without them still retargets
        // its body, and a body-only clip is a perfectly good clip. Mixamo
        // calls the little finger "Pinky".
        for side in [RigSide::Left, RigSide::Right] {
            for finger in MixamoFinger::ALL {
                for (segment, role) in finger.segments(side) {
                    profile = profile.with(role, finger.name(side, segment));
                }
            }
        }

        // Mixamo's spine is three joints; rigs it is retargeted onto rarely
        // agree. Declaring the chain lets motion be spread over however many
        // the target has.
        profile.with_chain(
            HumanoidChain::Spine,
            ChainBinding::new([
                RigJointPrefix::MIXAMO.apply(&"Spine".into()),
                RigJointPrefix::MIXAMO.apply(&"Spine1".into()),
                RigJointPrefix::MIXAMO.apply(&"Spine2".into()),
            ]),
        )
    }
}

impl MixamoRig {
    const BODY_BINDINGS: &'static [(HumanoidJoint, RigJointName, JointRequirement)] = &[
        (
            HumanoidJoint::Pelvis,
            RigJointName::from_static("Hips"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::SpineLower,
            RigJointName::from_static("Spine"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::SpineMid,
            RigJointName::from_static("Spine1"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::Chest,
            RigJointName::from_static("Spine2"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::Neck,
            RigJointName::from_static("Neck"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::Head,
            RigJointName::from_static("Head"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::ClavicleLeft,
            RigJointName::from_static("LeftShoulder"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::UpperArmLeft,
            RigJointName::from_static("LeftArm"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::LowerArmLeft,
            RigJointName::from_static("LeftForeArm"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::HandLeft,
            RigJointName::from_static("LeftHand"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::ClavicleRight,
            RigJointName::from_static("RightShoulder"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::UpperArmRight,
            RigJointName::from_static("RightArm"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::LowerArmRight,
            RigJointName::from_static("RightForeArm"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::HandRight,
            RigJointName::from_static("RightHand"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::UpperLegLeft,
            RigJointName::from_static("LeftUpLeg"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::LowerLegLeft,
            RigJointName::from_static("LeftLeg"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::FootLeft,
            RigJointName::from_static("LeftFoot"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::ToeLeft,
            RigJointName::from_static("LeftToeBase"),
            JointRequirement::Optional,
        ),
        (
            HumanoidJoint::UpperLegRight,
            RigJointName::from_static("RightUpLeg"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::LowerLegRight,
            RigJointName::from_static("RightLeg"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::FootRight,
            RigJointName::from_static("RightFoot"),
            JointRequirement::Required,
        ),
        (
            HumanoidJoint::ToeRight,
            RigJointName::from_static("RightToeBase"),
            JointRequirement::Optional,
        ),
    ];
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
            skeleton.find_joint_by_name(&"mixamorig:Hips".into())
        );
        assert_eq!(
            resolved.joint(HumanoidJoint::LowerArmRight),
            skeleton.find_joint_by_name(&"mixamorig:RightForeArm".into())
        );
        assert_eq!(
            resolved.joint(HumanoidJoint::IndexDistalLeft),
            skeleton.find_joint_by_name(&"mixamorig:LeftHandIndex3".into())
        );
        assert!(
            resolved.missing.is_empty(),
            "unmapped: {:?}",
            resolved.missing
        );
        assert_eq!(
            resolved.chains[&HumanoidChain::Spine].count(),
            crate::animation::retarget::ChainJointCount::from(3_usize)
        );
    }

    #[test]
    fn detection_is_by_marker_joints() {
        assert!(MixamoRig::profile().matches(&MixamoRig::skeleton()));
    }
}
