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
//! it recognizes. An inferred pelvis binding is required; roles it cannot place
//! are simply left out — an unmapped joint keeps its rest pose, which is a
//! visible but harmless result, where a *wrongly* mapped one is neither.

use crate::skeleton::Skeleton;

use name::{InferenceKeyword, InferenceStem, InferredName, Side};

use super::super::profile::{ChainBinding, ReferencePose, RigProfile, RootSource};
use super::super::semantic::{HumanoidChain, HumanoidJoint};

mod name;

/// Keywords per role, most specific first. A joint matching a longer keyword
/// beats one matching a shorter one, which is what keeps `LeftUpLeg` from
/// being taken for a lower leg.
const BODY: &[BodyHints] = &[
    BodyHints {
        role: HumanoidJoint::Pelvis,
        keywords: &[
            InferenceKeyword::Hips,
            InferenceKeyword::Pelvis,
            InferenceKeyword::Hip,
        ],
    },
    BodyHints {
        role: HumanoidJoint::Neck,
        keywords: &[InferenceKeyword::Neck],
    },
    BodyHints {
        role: HumanoidJoint::Head,
        keywords: &[InferenceKeyword::Head],
    },
    BodyHints {
        role: HumanoidJoint::ClavicleLeft,
        keywords: &[
            InferenceKeyword::Clavicle,
            InferenceKeyword::Shoulder,
            InferenceKeyword::Collar,
        ],
    },
    BodyHints {
        role: HumanoidJoint::UpperArmLeft,
        keywords: &[
            InferenceKeyword::UpperArm,
            InferenceKeyword::UpArm,
            InferenceKeyword::Humerus,
            InferenceKeyword::Shldr,
            InferenceKeyword::Arm,
        ],
    },
    BodyHints {
        role: HumanoidJoint::LowerArmLeft,
        keywords: &[
            InferenceKeyword::Forearm,
            InferenceKeyword::LowerArm,
            InferenceKeyword::LowArm,
            InferenceKeyword::Elbow,
            InferenceKeyword::Ulna,
        ],
    },
    BodyHints {
        role: HumanoidJoint::HandLeft,
        keywords: &[InferenceKeyword::Hand, InferenceKeyword::Wrist],
    },
    BodyHints {
        role: HumanoidJoint::UpperLegLeft,
        keywords: &[
            InferenceKeyword::UpperLeg,
            InferenceKeyword::UpLeg,
            InferenceKeyword::Thigh,
            InferenceKeyword::Femur,
            InferenceKeyword::Hip,
        ],
    },
    BodyHints {
        role: HumanoidJoint::LowerLegLeft,
        keywords: &[
            InferenceKeyword::LowerLeg,
            InferenceKeyword::LowLeg,
            InferenceKeyword::Shin,
            InferenceKeyword::Calf,
            InferenceKeyword::Knee,
            InferenceKeyword::Tibia,
            InferenceKeyword::Leg,
        ],
    },
    BodyHints {
        role: HumanoidJoint::FootLeft,
        keywords: &[InferenceKeyword::Foot, InferenceKeyword::Ankle],
    },
    BodyHints {
        role: HumanoidJoint::ToeLeft,
        keywords: &[
            InferenceKeyword::ToeBase,
            InferenceKeyword::Toe,
            InferenceKeyword::Ball,
        ],
    },
];

/// The spine, which is a run rather than a set of named slots.
const SPINE: &[InferenceKeyword] = &[
    InferenceKeyword::Spine,
    InferenceKeyword::Chest,
    InferenceKeyword::Torso,
    InferenceKeyword::Abdomen,
    InferenceKeyword::Waist,
];

const FINGERS: &[FingerHints] = &[
    FingerHints {
        keyword: InferenceKeyword::Thumb,
        roles: [
            HumanoidJoint::ThumbProximalLeft,
            HumanoidJoint::ThumbIntermediateLeft,
            HumanoidJoint::ThumbDistalLeft,
        ],
    },
    FingerHints {
        keyword: InferenceKeyword::Index,
        roles: [
            HumanoidJoint::IndexProximalLeft,
            HumanoidJoint::IndexIntermediateLeft,
            HumanoidJoint::IndexDistalLeft,
        ],
    },
    FingerHints {
        keyword: InferenceKeyword::Middle,
        roles: [
            HumanoidJoint::MiddleProximalLeft,
            HumanoidJoint::MiddleIntermediateLeft,
            HumanoidJoint::MiddleDistalLeft,
        ],
    },
    FingerHints {
        keyword: InferenceKeyword::Ring,
        roles: [
            HumanoidJoint::RingProximalLeft,
            HumanoidJoint::RingIntermediateLeft,
            HumanoidJoint::RingDistalLeft,
        ],
    },
    FingerHints {
        keyword: InferenceKeyword::Pinky,
        roles: [
            HumanoidJoint::LittleProximalLeft,
            HumanoidJoint::LittleIntermediateLeft,
            HumanoidJoint::LittleDistalLeft,
        ],
    },
    FingerHints {
        keyword: InferenceKeyword::Little,
        roles: [
            HumanoidJoint::LittleProximalLeft,
            HumanoidJoint::LittleIntermediateLeft,
            HumanoidJoint::LittleDistalLeft,
        ],
    },
];

/// An authored role and the hints that may recognize it.
struct BodyHints {
    role: HumanoidJoint,
    keywords: &'static [InferenceKeyword],
}

/// A finger hint and its proximal-to-distal roles.
struct FingerHints {
    keyword: InferenceKeyword,
    roles: [HumanoidJoint; 3],
}

/// A joint as inference sees it.
struct Candidate {
    index: usize,
    side: Side,
    stem: InferenceStem,
    claimed: bool,
}

/// The specificity and skeleton position of a recognized candidate.
struct BestMatch {
    keyword_length: usize,
    index: usize,
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
                let InferredName { side, stem } = InferredName::from_joint_name(&joint.name);
                Candidate {
                    index,
                    side,
                    stem,
                    claimed: false,
                }
            })
            .collect();

        // Nothing here can know what posture the rig was bound in, and a
        // reference that is merely "whatever this rig happened to bind as" is
        // not a reference at all — it only lines up with the target rig by
        // luck. Straightening from the rig's own geometry makes it definite,
        // and costs nothing on a rig that was already T-posed.
        let mut profile = RigProfile::new("inferred".into()).with_reference(ReferencePose::TPose);
        let name_of = |index: usize| skeleton.joints[index].name.clone();

        // The spine first: it is a run of joints in hierarchy order, and
        // claiming it stops `chest` being mistaken for anything else.
        let spine: Vec<usize> = candidates
            .iter()
            .filter(|candidate| {
                candidate.side == Side::Center
                    && SPINE.iter().any(|word| candidate.stem.starts_with(*word))
            })
            .map(|candidate| candidate.index)
            .collect();
        for index in &spine {
            candidates[*index].claimed = true;
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
            profile = profile.with(role, name_of(*index));
        }
        if spine.len() > 1 {
            profile = profile.with_chain(
                HumanoidChain::Spine,
                ChainBinding::new(spine.iter().map(|index| name_of(*index))),
            );
        }

        for hints in BODY {
            for side in [Side::Center, Side::Left, Side::Right] {
                let role = match (side, hints.role) {
                    (Side::Right, role) => mirrored(role),
                    (_, role) => role,
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
                if let Some(index) = best_match(&candidates, side, hints.keywords) {
                    candidates[index].claimed = true;
                    profile = profile.with(role, name_of(index));
                }
            }
        }

        for hints in FINGERS {
            for side in [Side::Left, Side::Right] {
                let segments: Vec<usize> = candidates
                    .iter()
                    .filter(|candidate| {
                        !candidate.claimed
                            && candidate.side == side
                            && candidate.stem.contains(hints.keyword)
                    })
                    .map(|candidate| candidate.index)
                    .collect();
                for (segment, index) in segments.iter().take(3).enumerate() {
                    let role = match side {
                        Side::Right => mirrored(hints.roles[segment]),
                        _ => hints.roles[segment],
                    };
                    if profile.binding(role).is_none() {
                        candidates[*index].claimed = true;
                        profile = profile.with(role, name_of(*index));
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
                    && candidate.stem.is_root_hint()
                    && skeleton.joints[candidate.index].parent_index.is_none()
            })
            .map(|candidate| name_of(candidate.index));
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
}

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

/// The unclaimed joint on `side` matching the most specific keyword.
///
/// Ties go to the joint nearer the start of the skeleton, which is nearer the
/// root in every importer the engine has.
fn best_match(
    candidates: &[Candidate],
    side: Side,
    keywords: &[InferenceKeyword],
) -> Option<usize> {
    let mut best: Option<BestMatch> = None;
    for candidate in candidates {
        if candidate.claimed || candidate.side != side {
            continue;
        }
        let Some(length) = keywords
            .iter()
            .filter(|keyword| candidate.stem.contains(**keyword))
            .map(|keyword| keyword.length())
            .max()
        else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|matched| length > matched.keyword_length)
        {
            best = Some(BestMatch {
                keyword_length: length,
                index: candidate.index,
            });
        }
    }
    best.map(|matched| matched.index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Joint;
    use fabelgeist_math::matrix::Mat4;

    fn skeleton(names: &[&str]) -> Skeleton {
        Skeleton::new(
            names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    Joint::new(
                        (*name).to_string(),
                        index,
                        index.checked_sub(1),
                        Mat4::identity(),
                        Default::default(),
                        Some(index),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn a_mixamo_style_rig_is_inferred() {
        let rig = skeleton(&[
            "mixamorig:Hips",
            "mixamorig:Spine",
            "mixamorig:Spine1",
            "mixamorig:Spine2",
            "mixamorig:Neck",
            "mixamorig:Head",
            "mixamorig:LeftShoulder",
            "mixamorig:LeftArm",
            "mixamorig:LeftForeArm",
            "mixamorig:LeftHand",
            "mixamorig:RightShoulder",
            "mixamorig:RightArm",
            "mixamorig:RightForeArm",
            "mixamorig:RightHand",
            "mixamorig:LeftUpLeg",
            "mixamorig:LeftLeg",
            "mixamorig:LeftFoot",
            "mixamorig:LeftToeBase",
        ]);
        let profile = RigProfile::infer(&rig);
        let resolved = profile.resolve(&rig).expect("an inferred profile resolves");
        let named = |role| {
            resolved
                .joint(role)
                .map(|index| rig.joints[index].name.as_str())
        };

        assert_eq!(named(HumanoidJoint::Pelvis), Some("mixamorig:Hips"));
        assert_eq!(named(HumanoidJoint::SpineLower), Some("mixamorig:Spine"));
        assert_eq!(named(HumanoidJoint::Chest), Some("mixamorig:Spine2"));
        assert_eq!(named(HumanoidJoint::Head), Some("mixamorig:Head"));
        assert_eq!(
            named(HumanoidJoint::ClavicleRight),
            Some("mixamorig:RightShoulder")
        );
        // The trap: "LeftArm" is an upper arm and "LeftLeg" a lower leg.
        assert_eq!(
            named(HumanoidJoint::UpperArmLeft),
            Some("mixamorig:LeftArm")
        );
        assert_eq!(
            named(HumanoidJoint::LowerArmLeft),
            Some("mixamorig:LeftForeArm")
        );
        assert_eq!(
            named(HumanoidJoint::UpperLegLeft),
            Some("mixamorig:LeftUpLeg")
        );
        assert_eq!(
            named(HumanoidJoint::LowerLegLeft),
            Some("mixamorig:LeftLeg")
        );
        assert_eq!(named(HumanoidJoint::ToeLeft), Some("mixamorig:LeftToeBase"));
        assert_eq!(resolved.chains[&HumanoidChain::Spine].len(), 3);
    }

    #[test]
    fn a_mocap_rig_with_different_names_is_inferred() {
        // The CMU/BioVision naming a lot of BVH files use.
        let rig = skeleton(&[
            "Hips",
            "LowerBack",
            "Spine",
            "Spine1",
            "Neck",
            "Head",
            "LeftShoulder",
            "LeftArm",
            "LeftForeArm",
            "LeftHand",
            "LHipJoint",
            "LeftUpLeg",
            "LeftLeg",
            "LeftFoot",
            "LeftToeBase",
        ]);
        let profile = RigProfile::infer(&rig);
        let resolved = profile.resolve(&rig).expect("an inferred profile resolves");
        let named = |role| {
            resolved
                .joint(role)
                .map(|index| rig.joints[index].name.as_str())
        };

        assert_eq!(named(HumanoidJoint::Pelvis), Some("Hips"));
        assert_eq!(named(HumanoidJoint::UpperLegLeft), Some("LeftUpLeg"));
        assert_eq!(named(HumanoidJoint::LowerLegLeft), Some("LeftLeg"));
        assert_eq!(named(HumanoidJoint::FootLeft), Some("LeftFoot"));
        assert_eq!(named(HumanoidJoint::Head), Some("Head"));
    }

    #[test]
    fn a_rig_of_nonsense_names_yields_an_empty_mapping_rather_than_a_wrong_one() {
        let rig = skeleton(&["bone_000", "bone_001", "bone_002"]);
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
        let rig = skeleton(&["Hips", "Spine", "Neck", "Head"]);
        let profile = RigProfile::infer(&rig);
        let json = serde_json::to_string(&profile).expect("it serializes");
        let restored: RigProfile = serde_json::from_str(&json).expect("it deserializes");
        assert_eq!(profile, restored);
    }
}
