//! Public behavior of the private inference matching boundary.

use fabelgeist_animation::animation::retarget::{
    HumanoidChain, HumanoidJoint, JointRequirement, RetargetProfile, RetargetSettings,
    RetargetStrictness, Retargeter, RigProfile, RigProfileName, RootMotionPolicy, RootSource,
    ScalePolicy, TranslationPolicy,
};
use fabelgeist_animation::animation::{Animation, AnimationClipName, Curve, JointTrack};
use fabelgeist_animation::skeleton::{Joint, Skeleton};
use fabelgeist_math::matrix::Mat4;
use fabelgeist_math::transform::Transform;
use fabelgeist_math::vector::{Vec3, Vec4};

struct BodyExpectation {
    role: HumanoidJoint,
    hints: &'static [&'static str],
}

fn skeleton(names: &[&str]) -> Skeleton {
    Skeleton::new(
        names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Joint::new(
                    (*name).to_owned(),
                    index,
                    None,
                    Mat4::identity(),
                    Transform::identity(),
                    Some(index),
                )
            })
            .collect(),
    )
}

#[test]
fn every_authored_hint_reaches_public_inference_and_resolution() {
    use HumanoidJoint::*;
    let body = [
        BodyExpectation {
            role: Pelvis,
            hints: &["hips", "pelvis", "hip"],
        },
        BodyExpectation {
            role: Neck,
            hints: &["neck"],
        },
        BodyExpectation {
            role: Head,
            hints: &["head"],
        },
        BodyExpectation {
            role: ClavicleLeft,
            hints: &["clavicle", "shoulder", "collar"],
        },
        BodyExpectation {
            role: UpperArmLeft,
            hints: &["upperarm", "uparm", "humerus", "shldr", "arm"],
        },
        BodyExpectation {
            role: LowerArmLeft,
            hints: &["forearm", "lowerarm", "lowarm", "elbow", "ulna"],
        },
        BodyExpectation {
            role: HandLeft,
            hints: &["hand", "wrist"],
        },
        BodyExpectation {
            role: UpperLegLeft,
            hints: &["upperleg", "upleg", "thigh", "femur", "hip"],
        },
        BodyExpectation {
            role: LowerLegLeft,
            hints: &["lowerleg", "lowleg", "shin", "calf", "knee", "tibia", "leg"],
        },
        BodyExpectation {
            role: FootLeft,
            hints: &["foot", "ankle"],
        },
        BodyExpectation {
            role: ToeLeft,
            hints: &["toebase", "toe", "ball"],
        },
    ];
    for row in body {
        for hint in row.hints {
            let name = if matches!(row.role, Pelvis | Neck | Head) {
                (*hint).to_owned()
            } else {
                format!("Left{hint}")
            };
            let names = match row.role {
                Pelvis => vec![name.as_str()],
                LowerArmLeft => vec!["hips", "LeftUpperArm", name.as_str()],
                _ => vec!["hips", name.as_str()],
            };
            let rig = skeleton(&names);
            let profile = RigProfile::infer(&rig);
            let resolved = profile.resolve(&rig).expect("authored hint resolves");
            let index = resolved.joint(row.role).expect("authored hint recognized");
            assert_eq!(rig.joints[index].name, name);
        }
    }
    for hint in ["spine", "chest", "torso", "abdomen", "waist"] {
        let rig = skeleton(&["hips", hint]);
        let resolved = RigProfile::infer(&rig)
            .resolve(&rig)
            .expect("spine hint resolves");
        assert_eq!(resolved.joint(SpineLower), Some(1));
    }
    for hint in ["thumb", "index", "middle", "ring", "pinky", "little"] {
        let name = format!("Left{hint}");
        let rig = skeleton(&["hips", &name]);
        let resolved = RigProfile::infer(&rig)
            .resolve(&rig)
            .expect("finger hint resolves");
        assert_eq!(resolved.joints.len(), 2);
        assert!(resolved.joints.values().any(|index| *index == 1));
    }
    for hint in ["root", "reference", "armature"] {
        let rig = skeleton(&[hint, "hips"]);
        assert_eq!(
            RigProfile::infer(&rig).root,
            RootSource::Joint(hint.to_owned())
        );
    }
}

#[test]
fn side_parsing_preserves_exact_joint_spelling_and_lossy_matching() {
    let left_names = [
        "LeftArm",
        "l_arm",
        "l-arm",
        "l.arm",
        "l arm",
        "arm_l",
        "arm-l",
        "arm.l",
        "arm l",
        "LArm",
        "lArm",
        "ns:LeftArm",
        "ns|LeftArm",
        "RightLeftArm",
        "骨Left\n\t\0Arm",
        "larm",
    ];
    for name in left_names {
        let rig = skeleton(&["hips", name]);
        let profile = RigProfile::infer(&rig);
        assert_eq!(
            profile
                .binding(HumanoidJoint::UpperArmLeft)
                .expect("left arm")
                .names,
            [name.to_owned()]
        );
        let wire = serde_json::to_string(&profile).expect("serialize exact spelling");
        let decoded: RigProfile = serde_json::from_str(&wire).expect("decode exact spelling");
        assert_eq!(decoded, profile);
    }
    // A center candidate is considered before a more specific left candidate.
    let rig = skeleton(&["hips", "larm", "LeftUpperArm"]);
    let profile = RigProfile::infer(&rig);
    let resolved = profile.resolve(&rig).expect("center-first inference");
    assert_eq!(resolved.joint(HumanoidJoint::UpperArmLeft), Some(1));
    assert_eq!(resolved.unmapped, [2]);
    // R before uppercase O is a side marker, even in a root-like spelling.
    let rig = skeleton(&["hips", "ns:ROOT"]);
    assert_eq!(RigProfile::infer(&rig).root, RootSource::Pelvis);
    let right_names = [
        "RightArm",
        "r_arm",
        "r-arm",
        "r.arm",
        "r arm",
        "arm_r",
        "arm-r",
        "arm.r",
        "arm r",
        "RArm",
        "rArm",
        "ns:RightArm",
    ];
    for name in right_names {
        let rig = skeleton(&["hips", name]);
        assert_eq!(
            RigProfile::infer(&rig)
                .binding(HumanoidJoint::UpperArmRight)
                .expect("right arm")
                .names,
            [name.to_owned()]
        );
    }
    let rig = skeleton(&["", "骨架", "L", "R", "\n\t\0"]);
    assert!(RigProfile::infer(&rig).joints.is_empty());
}

#[test]
fn specificity_claims_and_duplicate_names_keep_skeleton_order() {
    let rig = skeleton(&[
        "hips",
        "LeftArm",
        "LeftForeArm",
        "LeftUpperArm",
        "LeftWristA",
        "LeftWristB",
        "RightHand",
        "RightHand",
    ]);
    let profile = RigProfile::infer(&rig);
    let resolved = profile.resolve(&rig).expect("competition resolves");
    assert_eq!(resolved.joint(HumanoidJoint::UpperArmLeft), Some(3));
    assert_eq!(resolved.joint(HumanoidJoint::LowerArmLeft), Some(2));
    assert_eq!(resolved.joint(HumanoidJoint::HandLeft), Some(4));
    assert_eq!(resolved.joint(HumanoidJoint::HandRight), Some(6));
    assert_eq!(
        profile
            .binding(HumanoidJoint::Pelvis)
            .expect("pelvis")
            .required,
        JointRequirement::Required
    );
    assert_eq!(profile.name, RigProfileName::from("inferred"));
}

#[test]
fn spine_finger_and_root_policies_survive_public_round_trip() {
    let names = [
        "hips",
        "Armature",
        "spine03",
        "spine01",
        "chest",
        "spine04",
        "spine05",
        "LeftPinky02",
        "LeftPinky04",
        "LeftLittle01",
        "LeftLittle03",
    ];
    let rig = Skeleton::new(
        names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                Joint::new(
                    (*name).to_owned(),
                    index,
                    index.checked_sub(1),
                    Mat4::identity(),
                    Transform::identity(),
                    Some(index),
                )
            })
            .collect(),
    );
    let profile = RigProfile::infer(&rig);
    assert_eq!(profile.root, RootSource::Pelvis);
    let wire = serde_json::to_string(&profile).expect("serialize inferred recipe");
    let decoded: RigProfile = serde_json::from_str(&wire).expect("deserialize recipe");
    assert_eq!(decoded, profile);
    let resolved = decoded.resolve(&rig).expect("inferred recipe resolves");
    assert_eq!(resolved.chains[&HumanoidChain::Spine], [2, 3, 4, 5, 6]);
    assert_eq!(resolved.joint(HumanoidJoint::UpperChest), Some(5));
    assert_eq!(resolved.joint(HumanoidJoint::LittleProximalLeft), Some(7));
    assert_eq!(
        resolved.joint(HumanoidJoint::LittleIntermediateLeft),
        Some(8)
    );
    assert!(resolved.joint(HumanoidJoint::LittleDistalLeft).is_none());
}

#[test]
fn required_errors_precede_strict_transfer_checks() {
    let source = skeleton(&["hips", "LeftArm"]);
    let target = skeleton(&["hips"]);
    let missing_source =
        RigProfile::infer(&source).with_required(HumanoidJoint::Head, "missing-source");
    let missing_target =
        RigProfile::infer(&target).with_required(HumanoidJoint::Head, "missing-target");
    let settings = RetargetSettings {
        strict: RetargetStrictness::Strict,
        ..RetargetSettings::default()
    };
    let error = RetargetProfile::new(missing_source, missing_target.clone())
        .with_settings(settings.clone())
        .resolve(&source, &target)
        .expect_err("source first");
    assert_eq!(
        error.to_string(),
        "rig profile \"inferred\" requires joints the skeleton does not have: Head (expected missing-source)"
    );
    let error = RetargetProfile::new(RigProfile::infer(&source), missing_target)
        .with_settings(settings.clone())
        .resolve(&source, &target)
        .expect_err("target next");
    assert_eq!(
        error.to_string(),
        "rig profile \"inferred\" requires joints the skeleton does not have: Head (expected missing-target)"
    );
    let error = RetargetProfile::new(RigProfile::infer(&source), RigProfile::infer(&target))
        .with_settings(settings)
        .resolve(&source, &target)
        .expect_err("strict orphan last");
    assert_eq!(
        error.to_string(),
        "strict retargeting: the target rig has no joint for UpperArmLeft"
    );
}

#[test]
fn public_report_and_real_clip_keep_their_words() {
    let rig = skeleton(&["hips"]);
    let inferred = RigProfile::infer(&rig);
    let profile = RetargetProfile::new(inferred.clone(), inferred).with_settings(
        RetargetSettings::default()
            .with_scale(ScalePolicy::None)
            .with_translation(TranslationPolicy::Copy)
            .with_root_motion(RootMotionPolicy::Keep),
    );
    let resolved = profile.resolve(&rig, &rig).expect("self transfer resolves");
    let retargeter = Retargeter::new(&rig, &rig, &profile).expect("public transfer");
    assert_eq!(
        retargeter.report(),
        format!(
            "{}\nscale: 1.0000 target units per source unit\n",
            resolved.report(&rig, &rig)
        )
    );
    let track = JointTrack {
        translation: Some(Curve::new(
            vec![0.0, 1.0],
            vec![Vec3::default(), Vec3::new(1.0, 2.0, 3.0)],
        )),
        rotation: Some(Curve::new(vec![0.0, 1.0], vec![Vec4::quat_identity(); 2])),
        ..JointTrack::new("hips")
    };
    let mut clip = Animation {
        tracks: vec![track],
        ..Animation::new(AnimationClipName::from("inferred-motion"))
    };
    clip.recompute_duration();
    let output = retargeter.clip(&clip);
    assert_eq!(output.name, clip.name);
    assert_eq!(output.duration.to_bits(), 1.0_f32.to_bits());
    assert_eq!(
        output
            .key_times()
            .iter()
            .map(|time| time.to_bits())
            .collect::<Vec<_>>(),
        [0.0_f32.to_bits(), 1.0_f32.to_bits()]
    );
    let binding = output.bind(&rig);
    let samples = output.sample(&binding, 0.5);
    assert_eq!(samples.len(), 1);
    let position = samples[0].translation;
    assert_eq!(
        [
            position.x.to_bits(),
            position.y.to_bits(),
            position.z.to_bits()
        ],
        [0.5_f32.to_bits(), 1.0_f32.to_bits(), 1.5_f32.to_bits()]
    );
    let wire = serde_json::to_string(&output).expect("serialized clip");
    let decoded: Animation = serde_json::from_str(&wire).expect("clip round trip");
    assert_eq!(
        serde_json::to_value(decoded).expect("decoded clip value"),
        serde_json::to_value(output).expect("output clip value")
    );
}
