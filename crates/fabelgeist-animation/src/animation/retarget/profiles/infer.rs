//! Guessing a rig profile from joint names.
//!
//! An explicit profile is always better and always available; this exists so
//! that a rig nobody has written one for — a mocap BVH, an unfamiliar FBX — can
//! be retargeted immediately, and so that writing the real profile starts from
//! something rather than nothing.
//!
//! The output is an ordinary [`RigProfile`]: inspect it with
//! [`ResolvedProfile::report`](super::super::ResolvedProfile::report), edit it,
//! serialize it, ship it. Nothing downstream can tell it was inferred.
//!
//! Inference is deliberately conservative. It claims a joint only on a keyword
//! it recognizes, requires the pelvis only when it finds one, and roles it cannot place
//! are simply left out — an unmapped joint keeps its rest pose, which is a
//! visible but harmless result, where a *wrongly* mapped one is neither.

use super::inference_name::{
    ClaimState, InferenceKeyword, InferenceStem, InferredName, KeywordSpecificity, Side,
};
use crate::skeleton::Skeleton;
use InferenceKeyword::*;
use fabelgeist_rig::{RigJointMembership, RigJointOrdinal};

use super::super::profile::{ChainBinding, ReferencePose, RigProfile, RootSource};
use super::super::semantic::{HumanoidChain, HumanoidJoint};

/// Keywords per role, most specific first. A joint matching a longer keyword
/// beats one matching a shorter one, which is what keeps `LeftUpLeg` from
/// being taken for a lower leg.
const BODY: &[(HumanoidJoint, &[InferenceKeyword])] = &[
    (HumanoidJoint::Pelvis, &[Hips, Pelvis, Hip]),
    (HumanoidJoint::Neck, &[Neck]),
    (HumanoidJoint::Head, &[Head]),
    (HumanoidJoint::ClavicleLeft, &[Clavicle, Shoulder, Collar]),
    (
        HumanoidJoint::UpperArmLeft,
        &[Upperarm, Uparm, Humerus, Shldr, Arm],
    ),
    (
        HumanoidJoint::LowerArmLeft,
        &[Forearm, Lowerarm, Lowarm, Elbow, Ulna],
    ),
    (HumanoidJoint::HandLeft, &[Hand, Wrist]),
    (
        HumanoidJoint::UpperLegLeft,
        &[Upperleg, Upleg, Thigh, Femur, Hip],
    ),
    (
        HumanoidJoint::LowerLegLeft,
        &[Lowerleg, Lowleg, Shin, Calf, Knee, Tibia, Leg],
    ),
    (HumanoidJoint::FootLeft, &[Foot, Ankle]),
    (HumanoidJoint::ToeLeft, &[Toebase, Toe, Ball]),
];

/// The spine, which is a run rather than a set of named slots.
const SPINE: &[InferenceKeyword] = &[Spine, Chest, Torso, Abdomen, Waist];

const FINGERS: &[(InferenceKeyword, [HumanoidJoint; 3])] = &[
    (
        Thumb,
        [
            HumanoidJoint::ThumbProximalLeft,
            HumanoidJoint::ThumbIntermediateLeft,
            HumanoidJoint::ThumbDistalLeft,
        ],
    ),
    (
        Index,
        [
            HumanoidJoint::IndexProximalLeft,
            HumanoidJoint::IndexIntermediateLeft,
            HumanoidJoint::IndexDistalLeft,
        ],
    ),
    (
        Middle,
        [
            HumanoidJoint::MiddleProximalLeft,
            HumanoidJoint::MiddleIntermediateLeft,
            HumanoidJoint::MiddleDistalLeft,
        ],
    ),
    (
        Ring,
        [
            HumanoidJoint::RingProximalLeft,
            HumanoidJoint::RingIntermediateLeft,
            HumanoidJoint::RingDistalLeft,
        ],
    ),
    (
        Pinky,
        [
            HumanoidJoint::LittleProximalLeft,
            HumanoidJoint::LittleIntermediateLeft,
            HumanoidJoint::LittleDistalLeft,
        ],
    ),
    (
        Little,
        [
            HumanoidJoint::LittleProximalLeft,
            HumanoidJoint::LittleIntermediateLeft,
            HumanoidJoint::LittleDistalLeft,
        ],
    ),
];

/// The right-hand counterpart of a left-hand role.
fn mirrored(role: HumanoidJoint) -> HumanoidJoint {
    use HumanoidJoint::*;
    match role {
        ClavicleLeft => ClavicleRight,
        UpperArmLeft => UpperArmRight,
        LowerArmLeft => LowerArmRight,
        HandLeft => HandRight,
        UpperLegLeft => UpperLegRight,
        LowerLegLeft => LowerLegRight,
        FootLeft => FootRight,
        ToeLeft => ToeRight,
        ThumbProximalLeft => ThumbProximalRight,
        ThumbIntermediateLeft => ThumbIntermediateRight,
        ThumbDistalLeft => ThumbDistalRight,
        IndexProximalLeft => IndexProximalRight,
        IndexIntermediateLeft => IndexIntermediateRight,
        IndexDistalLeft => IndexDistalRight,
        MiddleProximalLeft => MiddleProximalRight,
        MiddleIntermediateLeft => MiddleIntermediateRight,
        MiddleDistalLeft => MiddleDistalRight,
        RingProximalLeft => RingProximalRight,
        RingIntermediateLeft => RingIntermediateRight,
        RingDistalLeft => RingDistalRight,
        LittleProximalLeft => LittleProximalRight,
        LittleIntermediateLeft => LittleIntermediateRight,
        LittleDistalLeft => LittleDistalRight,
        other => other,
    }
}

/// A joint as inference sees it.
struct Candidate {
    index: RigJointOrdinal,
    side: Side,
    simple: InferenceStem,
    claimed: ClaimState,
}

impl RigProfile {
    /// Builds a profile for a rig by reading its joint names.
    ///
    /// Always check the result — [`Retargeter::report`](super::super::Retargeter::report)
    /// prints the whole mapping — and prefer an explicit profile for any rig
    /// you will use more than once.
    pub fn infer(skeleton: &Skeleton) -> Self {
        let mut candidates: Vec<Candidate> = skeleton
            .joints
            .iter()
            .enumerate()
            .map(|(index, joint)| {
                let InferredName { side, stem: simple } = InferredName::from(&joint.name);
                Candidate {
                    index: RigJointOrdinal::from(index),
                    side,
                    simple,
                    claimed: ClaimState::Available,
                }
            })
            .collect();

        // Nothing here can know what posture the rig was bound in, and a
        // reference that is merely "whatever this rig happened to bind as" is
        // not a reference at all — it only lines up with the target rig by
        // luck. Straightening from the rig's own geometry makes it definite,
        // and costs nothing on a rig that was already T-posed.
        let mut profile = RigProfile::new("inferred".into()).with_reference(ReferencePose::TPose);

        profile = profile.with_inferred_spine(skeleton, &mut candidates);

        for (role, keywords) in BODY {
            for side in [Side::Center, Side::Left, Side::Right] {
                let role = match (side, role) {
                    (Side::Right, role) => mirrored(*role),
                    (_, role) => *role,
                };
                // Centre roles only apply to roles that have no side.
                let wanted = match role {
                    HumanoidJoint::Pelvis | HumanoidJoint::Neck | HumanoidJoint::Head => {
                        Side::Center
                    }
                    _ => side,
                };
                if wanted != side || profile.binding(role).is_some() {
                    continue;
                }
                if let Some(index) = best_match(&candidates, side, keywords) {
                    candidates[usize::from(index)].claimed = ClaimState::Claimed;
                    profile = profile.with(role, skeleton.joint_name(index).clone());
                }
            }
        }

        for (word, roles) in FINGERS {
            for side in [Side::Left, Side::Right] {
                let segments: Vec<RigJointOrdinal> = candidates
                    .iter()
                    .filter(|candidate| {
                        candidate.claimed == ClaimState::Available
                            && candidate.side == side
                            && candidate.simple.contains(*word) == RigJointMembership::Included
                    })
                    .map(|candidate| candidate.index)
                    .collect();
                for (segment, index) in segments.iter().take(3).enumerate() {
                    let role = match side {
                        Side::Right => mirrored(roles[segment]),
                        _ => roles[segment],
                    };
                    if profile.binding(role).is_none() {
                        candidates[usize::from(*index)].claimed = ClaimState::Claimed;
                        profile = profile.with(role, skeleton.joint_name(*index).clone());
                    }
                }
            }
        }

        // The pelvis is where locomotion lives on most rigs; a rig with a
        // dedicated node above it says so by having one.
        let root = candidates
            .iter()
            .find(|candidate| {
                candidate.side == Side::Center
                    && candidate.simple.root_membership() == RigJointMembership::Included
                    && skeleton.joints[candidate.index].parent_index.is_none()
            })
            .map(|candidate| skeleton.joint_name(candidate.index).clone());
        profile.root = match root {
            Some(name) => RootSource::Joint(name),
            None => RootSource::Pelvis,
        };

        // The pelvis is the one joint the retargeter cannot do without.
        if let Some(binding) = profile.joints.get_mut(&HumanoidJoint::Pelvis) {
            *binding = binding.clone().required();
        }
        profile
    }
    /// Claims the center spine in hierarchy order before other keyword matches.
    fn with_inferred_spine(mut self, skeleton: &Skeleton, candidates: &mut [Candidate]) -> Self {
        // The spine first: it is a run of joints in hierarchy order, and
        // claiming it stops `chest` being mistaken for anything else.
        let spine: Vec<RigJointOrdinal> = candidates
            .iter()
            .filter(|candidate| {
                candidate.side == Side::Center
                    && SPINE.iter().any(|word| {
                        candidate.simple.starts_with(*word) == RigJointMembership::Included
                    })
            })
            .map(|candidate| candidate.index)
            .collect();
        for index in &spine {
            candidates[usize::from(*index)].claimed = ClaimState::Claimed;
        }
        for (position, index) in spine.iter().enumerate() {
            let role = match position {
                0 => HumanoidJoint::SpineLower,
                1 => HumanoidJoint::SpineMid,
                2 => HumanoidJoint::Chest,
                3 => HumanoidJoint::UpperChest,
                // Longer spines keep going through the chain rather than the
                // vocabulary, which is what chains are for.
                _ => break,
            };
            self = self.with(role, skeleton.joint_name(*index).clone());
        }
        if spine.len() > 1 {
            self = self.with_chain(
                HumanoidChain::Spine,
                ChainBinding::new(
                    spine
                        .iter()
                        .map(|index| skeleton.joint_name(*index).clone()),
                ),
            );
        }

        self
    }
}

/// The unclaimed joint on `side` matching the most specific keyword.
///
/// Ties go to the joint nearer the start of the skeleton, which is nearer the
/// root in every importer the engine has.
fn best_match(
    candidates: &[Candidate],
    side: Side,
    keywords: &[InferenceKeyword],
) -> Option<RigJointOrdinal> {
    let mut best: Option<(KeywordSpecificity, RigJointOrdinal)> = None;
    for candidate in candidates {
        if candidate.claimed == ClaimState::Claimed || candidate.side != side {
            continue;
        }
        let Some(length) = keywords
            .iter()
            .filter(|keyword| candidate.simple.contains(**keyword) == RigJointMembership::Included)
            .map(|keyword| keyword.specificity())
            .max()
        else {
            continue;
        };
        if best.is_none_or(|(best_length, _)| length > best_length) {
            best = Some((length, candidate.index));
        }
    }
    best.map(|(_, index)| index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Joint;
    use crate::skeleton::SkinJointOrdinal;
    use fabelgeist_math::matrix::Mat4;
    use fabelgeist_rig::RigJointName;

    fn skeleton(names: &[RigJointName]) -> Skeleton {
        Skeleton::new(
            names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    Joint::new(
                        name.clone(),
                        RigJointOrdinal::from(index),
                        index.checked_sub(1).map(RigJointOrdinal::from),
                        Mat4::identity(),
                        Default::default(),
                        Some(SkinJointOrdinal::from(index)),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn sides_are_read_however_a_rig_spells_them() {
        assert_eq!(
            InferredName::from(&RigJointName::from("LeftUpLeg")),
            InferredName {
                side: Side::Left,
                stem: InferenceStem::from("upleg".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("mixamorig:RightArm")),
            InferredName {
                side: Side::Right,
                stem: InferenceStem::from("arm".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("l_uparm")),
            InferredName {
                side: Side::Left,
                stem: InferenceStem::from("uparm".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("upperarm.R")),
            InferredName {
                side: Side::Right,
                stem: InferenceStem::from("upperarm".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("LHipJoint")),
            InferredName {
                side: Side::Left,
                stem: InferenceStem::from("hipjoint".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("lFemur")),
            InferredName {
                side: Side::Left,
                stem: InferenceStem::from("femur".to_owned())
            }
        );
        // Words that merely start with l or r are not sides.
        assert_eq!(
            InferredName::from(&RigJointName::from("LowerLeg")),
            InferredName {
                side: Side::Center,
                stem: InferenceStem::from("lowerleg".to_owned())
            }
        );
        assert_eq!(
            InferredName::from(&RigJointName::from("Hips")),
            InferredName {
                side: Side::Center,
                stem: InferenceStem::from("hips".to_owned())
            }
        );
    }

    #[test]
    fn a_mixamo_style_rig_is_inferred() {
        let rig = skeleton(&[
            "mixamorig:Hips".into(),
            "mixamorig:Spine".into(),
            "mixamorig:Spine1".into(),
            "mixamorig:Spine2".into(),
            "mixamorig:Neck".into(),
            "mixamorig:Head".into(),
            "mixamorig:LeftShoulder".into(),
            "mixamorig:LeftArm".into(),
            "mixamorig:LeftForeArm".into(),
            "mixamorig:LeftHand".into(),
            "mixamorig:RightShoulder".into(),
            "mixamorig:RightArm".into(),
            "mixamorig:RightForeArm".into(),
            "mixamorig:RightHand".into(),
            "mixamorig:LeftUpLeg".into(),
            "mixamorig:LeftLeg".into(),
            "mixamorig:LeftFoot".into(),
            "mixamorig:LeftToeBase".into(),
        ]);
        let profile = RigProfile::infer(&rig);
        let resolved = profile.resolve(&rig).expect("an inferred profile resolves");
        let named = |role| {
            resolved
                .joint(role)
                .map(|index| rig.joints[index].name.clone())
        };

        assert_eq!(named(HumanoidJoint::Pelvis), Some("mixamorig:Hips".into()));
        assert_eq!(
            named(HumanoidJoint::SpineLower),
            Some("mixamorig:Spine".into())
        );
        assert_eq!(named(HumanoidJoint::Chest), Some("mixamorig:Spine2".into()));
        assert_eq!(named(HumanoidJoint::Head), Some("mixamorig:Head".into()));
        assert_eq!(
            named(HumanoidJoint::ClavicleRight),
            Some("mixamorig:RightShoulder".into())
        );
        // The trap: "LeftArm" is an upper arm and "LeftLeg" a lower leg.
        assert_eq!(
            named(HumanoidJoint::UpperArmLeft),
            Some("mixamorig:LeftArm".into())
        );
        assert_eq!(
            named(HumanoidJoint::LowerArmLeft),
            Some("mixamorig:LeftForeArm".into())
        );
        assert_eq!(
            named(HumanoidJoint::UpperLegLeft),
            Some("mixamorig:LeftUpLeg".into())
        );
        assert_eq!(
            named(HumanoidJoint::LowerLegLeft),
            Some("mixamorig:LeftLeg".into())
        );
        assert_eq!(
            named(HumanoidJoint::ToeLeft),
            Some("mixamorig:LeftToeBase".into())
        );
        assert_eq!(
            resolved.chains[&HumanoidChain::Spine].count(),
            crate::animation::retarget::ChainJointCount::from(3_usize)
        );
    }

    #[test]
    fn a_mocap_rig_with_different_names_is_inferred() {
        // The CMU/BioVision naming a lot of BVH files use.
        let rig = skeleton(&[
            "Hips".into(),
            "LowerBack".into(),
            "Spine".into(),
            "Spine1".into(),
            "Neck".into(),
            "Head".into(),
            "LeftShoulder".into(),
            "LeftArm".into(),
            "LeftForeArm".into(),
            "LeftHand".into(),
            "LHipJoint".into(),
            "LeftUpLeg".into(),
            "LeftLeg".into(),
            "LeftFoot".into(),
            "LeftToeBase".into(),
        ]);
        let profile = RigProfile::infer(&rig);
        let resolved = profile.resolve(&rig).expect("an inferred profile resolves");
        let named = |role| {
            resolved
                .joint(role)
                .map(|index| rig.joints[index].name.clone())
        };

        assert_eq!(named(HumanoidJoint::Pelvis), Some("Hips".into()));
        assert_eq!(named(HumanoidJoint::UpperLegLeft), Some("LeftUpLeg".into()));
        assert_eq!(named(HumanoidJoint::LowerLegLeft), Some("LeftLeg".into()));
        assert_eq!(named(HumanoidJoint::FootLeft), Some("LeftFoot".into()));
        assert_eq!(named(HumanoidJoint::Head), Some("Head".into()));
    }

    #[test]
    fn a_rig_of_nonsense_names_yields_an_empty_mapping_rather_than_a_wrong_one() {
        let rig = skeleton(&["bone_000".into(), "bone_001".into(), "bone_002".into()]);
        let profile = RigProfile::infer(&rig);
        assert!(
            profile.joints.is_empty(),
            "inferred {:?} from nothing",
            profile.joints.keys().collect::<Vec<_>>()
        );
        // And it fails loudly rather than retargeting garbage.
        assert!(profile.resolve(&rig).is_ok());
    }

    #[test]
    fn an_inferred_profile_is_just_data() {
        let rig = skeleton(&["Hips".into(), "Spine".into(), "Neck".into(), "Head".into()]);
        let profile = RigProfile::infer(&rig);
        let json = serde_json::to_string(&profile).expect("it serializes");
        let restored: RigProfile = serde_json::from_str(&json).expect("it deserializes");
        assert_eq!(profile, restored);
    }
}
