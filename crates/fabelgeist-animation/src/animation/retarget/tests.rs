//! Tests for the generic retargeter, written against rigs that exist only
//! here. Nothing in this file mentions a real-world rig: if the algorithm
//! needs to know about one, it is not generic.

use super::{MissingRequiredJoint, RetargetError, RetargetStrictness};
use crate::animation::retarget::profile::*;
use crate::animation::retarget::semantic::{HumanoidChain, HumanoidJoint};
use crate::animation::retarget::{Retargeter, retarget};
use crate::animation::{
    Animation, Curve, JointTrack, JointTransform, LocalPose, model_pose, rest_pose,
};
use crate::skeleton::SkinJointOrdinal;
use crate::skeleton::{Joint, Skeleton};
use fabelgeist_math::matrix::Mat4;
use fabelgeist_math::transform::Transform;
use fabelgeist_math::vector::{Vec3, Vec4};
use fabelgeist_rig::RigJointOrdinal;
use fabelgeist_rig::{RigJointName, RigJointPrefix};

#[derive(Clone)]
struct PoseAssertionContext(String);
impl From<&str> for PoseAssertionContext {
    fn from(context: &str) -> Self {
        Self(context.to_owned())
    }
}
impl From<String> for PoseAssertionContext {
    fn from(context: String) -> Self {
        Self(context)
    }
}
impl From<&RigJointName> for PoseAssertionContext {
    fn from(name: &RigJointName) -> Self {
        Self(name.to_string())
    }
}
impl std::fmt::Display for PoseAssertionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Builds a skeleton from `(name, parent, position, euler rotation)` rows.
fn skeleton(rows: &[(RigJointName, Option<RigJointName>, Vec3, Vec3)]) -> Skeleton {
    let joints = rows
        .iter()
        .enumerate()
        .map(|(index, (name, parent, position, rotation))| {
            let parent_index = parent.as_ref().map(|parent| {
                rows.iter()
                    .position(|row| &row.0 == parent)
                    .map(RigJointOrdinal::from)
                    .expect("parent must be declared before its children")
            });
            Joint::new(
                name.clone(),
                RigJointOrdinal::from(index),
                parent_index,
                Mat4::identity(),
                Transform::new(*position, *rotation, Vec3::ones()),
                Some(SkinJointOrdinal::from(index)),
            )
        })
        .collect();
    Skeleton::new(joints)
}

/// A humanoid test rig. `size` scales every bone, so two rigs of different
/// size are otherwise identical; `arm` is the shoulders' rest rotation, which
/// is what makes one rig a T-pose and another an A-pose.
fn humanoid(prefix: &RigJointPrefix, size: f32, arm: Vec3) -> Skeleton {
    let name = |part: RigJointName| -> RigJointName { prefix.apply(&part) };
    let position = |x: f32, y: f32, z: f32| Vec3::new(x * size, y * size, z * size);
    let none = Vec3::new(0.0, 0.0, 0.0);

    let rows: Vec<(RigJointName, Option<RigJointName>, Vec3, Vec3)> = vec![
        (name("hips".into()), None, position(0.0, 1.0, 0.0), none),
        (
            name("spine".into()),
            Some(name("hips".into())),
            position(0.0, 0.15, 0.0),
            none,
        ),
        (
            name("chest".into()),
            Some(name("spine".into())),
            position(0.0, 0.15, 0.0),
            none,
        ),
        (
            name("neck".into()),
            Some(name("chest".into())),
            position(0.0, 0.2, 0.0),
            none,
        ),
        (
            name("head".into()),
            Some(name("neck".into())),
            position(0.0, 0.1, 0.0),
            none,
        ),
        (
            name("shoulder_l".into()),
            Some(name("chest".into())),
            position(0.05, 0.1, 0.0),
            arm,
        ),
        (
            name("upperarm_l".into()),
            Some(name("shoulder_l".into())),
            position(0.1, 0.0, 0.0),
            none,
        ),
        (
            name("lowerarm_l".into()),
            Some(name("upperarm_l".into())),
            position(0.25, 0.0, 0.0),
            none,
        ),
        (
            name("hand_l".into()),
            Some(name("lowerarm_l".into())),
            position(0.25, 0.0, 0.0),
            none,
        ),
        (
            name("shoulder_r".into()),
            Some(name("chest".into())),
            position(-0.05, 0.1, 0.0),
            -arm,
        ),
        (
            name("upperarm_r".into()),
            Some(name("shoulder_r".into())),
            position(-0.1, 0.0, 0.0),
            none,
        ),
        (
            name("lowerarm_r".into()),
            Some(name("upperarm_r".into())),
            position(-0.25, 0.0, 0.0),
            none,
        ),
        (
            name("hand_r".into()),
            Some(name("lowerarm_r".into())),
            position(-0.25, 0.0, 0.0),
            none,
        ),
        (
            name("upperleg_l".into()),
            Some(name("hips".into())),
            position(0.1, -0.05, 0.0),
            none,
        ),
        (
            name("lowerleg_l".into()),
            Some(name("upperleg_l".into())),
            position(0.0, -0.45, 0.0),
            none,
        ),
        (
            name("foot_l".into()),
            Some(name("lowerleg_l".into())),
            position(0.0, -0.45, 0.0),
            none,
        ),
        (
            name("upperleg_r".into()),
            Some(name("hips".into())),
            position(-0.1, -0.05, 0.0),
            none,
        ),
        (
            name("lowerleg_r".into()),
            Some(name("upperleg_r".into())),
            position(0.0, -0.45, 0.0),
            none,
        ),
        (
            name("foot_r".into()),
            Some(name("lowerleg_r".into())),
            position(0.0, -0.45, 0.0),
            none,
        ),
    ];

    skeleton(&rows)
}

/// The humanoid rig described in humanoid terms.
fn humanoid_profile(prefix: &RigJointPrefix) -> RigProfile {
    use HumanoidJoint::*;
    let name = |part: RigJointName| -> RigJointName { prefix.apply(&part) };
    RigProfile::new(format!("test:{prefix}").into())
        .with_required(Pelvis, name("hips".into()))
        .with(SpineLower, name("spine".into()))
        .with(Chest, name("chest".into()))
        .with(Neck, name("neck".into()))
        .with(Head, name("head".into()))
        .with(ClavicleLeft, name("shoulder_l".into()))
        .with_required(UpperArmLeft, name("upperarm_l".into()))
        .with(LowerArmLeft, name("lowerarm_l".into()))
        .with(HandLeft, name("hand_l".into()))
        .with(ClavicleRight, name("shoulder_r".into()))
        .with_required(UpperArmRight, name("upperarm_r".into()))
        .with(LowerArmRight, name("lowerarm_r".into()))
        .with(HandRight, name("hand_r".into()))
        .with(UpperLegLeft, name("upperleg_l".into()))
        .with(LowerLegLeft, name("lowerleg_l".into()))
        .with(FootLeft, name("foot_l".into()))
        .with(UpperLegRight, name("upperleg_r".into()))
        .with(LowerLegRight, name("lowerleg_r".into()))
        .with(FootRight, name("foot_r".into()))
}

fn settings() -> RetargetSettings {
    RetargetSettings::default().with_root_motion(RootMotionPolicy::Keep)
}

fn quat(axis: Vec3, degrees: f32) -> Vec4 {
    Vec4::from_axis_angle(axis, degrees.to_radians())
}

#[track_caller]
fn assert_quat_eq(actual: Vec4, expected: Vec4, what: &PoseAssertionContext) {
    // A quaternion and its negation are the same rotation.
    let alignment = actual.dot(expected).abs();
    assert!(
        (alignment - 1.0).abs() < 1.0e-4,
        "{what}: {actual} is not the rotation {expected}"
    );
}

#[track_caller]
fn assert_vec_eq(actual: Vec3, expected: Vec3, tolerance: f32, what: &PoseAssertionContext) {
    assert!(
        (actual - expected).length() < tolerance,
        "{what}: {actual} is not {expected}"
    );
}

/// Poses a source rig by overriding some joints' local rotations.
fn pose(source: &Skeleton, overrides: &[(RigJointName, Vec4)]) -> LocalPose {
    let mut locals = rest_pose(source);
    for (name, rotation) in overrides {
        let index = source
            .find_joint_by_name(name)
            .unwrap_or_else(|| panic!("no joint named {name}"));
        locals[index].rotation = *rotation;
    }
    locals
}

/// A joint's motion away from its rest pose, in model space. This is the
/// quantity retargeting is supposed to preserve.
fn model_delta(skeleton: &Skeleton, locals: &LocalPose, joint: &RigJointName) -> Vec4 {
    let index = skeleton.find_joint_by_name(joint).expect("joint exists");
    let rest = model_pose(skeleton, &rest_pose(skeleton))[index].rotation;
    model_pose(skeleton, locals)[index]
        .rotation
        .mul_quat(rest.conjugate())
        .normalize()
}

#[test]
fn exact_labels_beat_folded_collisions_and_duplicates_keep_the_first_joint() {
    let zero = Vec3::new(0.0, 0.0, 0.0);
    let rig = skeleton(&[
        ("ns:Left_Arm".into(), None, zero, zero),
        ("LeftArm".into(), None, zero, zero),
        ("other:Left-Arm".into(), None, zero, zero),
        ("ns:Left_Arm".into(), None, zero, zero),
    ]);
    let resolved = RigProfile::new("collisions".into())
        .with(HumanoidJoint::UpperArmLeft, "LeftArm".into())
        .with(HumanoidJoint::LowerArmLeft, "missing|LEFT.ARM".into())
        .with(HumanoidJoint::HandLeft, "ns:Left_Arm".into())
        .resolve(&rig)
        .unwrap();
    assert_eq!(
        resolved.joint(HumanoidJoint::UpperArmLeft),
        Some(RigJointOrdinal::from(1_usize))
    );
    assert_eq!(
        resolved.joint(HumanoidJoint::LowerArmLeft),
        Some(RigJointOrdinal::from(0_usize))
    );
    assert_eq!(
        resolved.joint(HumanoidJoint::HandLeft),
        Some(RigJointOrdinal::from(0_usize))
    );
    assert_eq!(
        rig.find_joint_by_name(&"ns:Left_Arm".into()),
        Some(RigJointOrdinal::from(0_usize))
    );
}

#[test]
fn missing_required_bindings_retain_role_and_candidate_order() {
    let rig = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RigProfile::new("diagnostic".into())
        .with_joint(
            HumanoidJoint::Head,
            JointBinding::new("a:cranium".into())
                .with_alias("a:skull".into())
                .required(),
        )
        .with_required(HumanoidJoint::ToeLeft, "a:toe".into());
    let error = profile.resolve(&rig).unwrap_err();
    assert_eq!(
        error,
        RetargetError::RequiredJointsMissing {
            profile: "diagnostic".into(),
            bindings: vec![
                MissingRequiredJoint {
                    role: HumanoidJoint::Head,
                    candidates: vec!["a:cranium".into(), "a:skull".into()]
                },
                MissingRequiredJoint {
                    role: HumanoidJoint::ToeLeft,
                    candidates: vec!["a:toe".into()]
                },
            ],
        }
    );
    assert_eq!(
        error.to_string(),
        "rig profile \"diagnostic\" requires joints the skeleton does not have: Head (expected a:cranium | a:skull), ToeLeft (expected a:toe)"
    );
}

#[test]
fn strict_resolution_retains_missing_target_roles() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let mut target_profile = humanoid_profile(&"b:".into());
    target_profile.joints.shift_remove(&HumanoidJoint::Head);
    target_profile.joints.shift_remove(&HumanoidJoint::Neck);
    let mut settings = settings();
    settings.strict = RetargetStrictness::Strict;
    let error = RetargetProfile::new(humanoid_profile(&"a:".into()), target_profile)
        .with_settings(settings)
        .resolve(&source, &target)
        .unwrap_err();
    assert_eq!(
        error,
        RetargetError::StrictTargetRolesMissing {
            roles: vec![HumanoidJoint::Neck, HumanoidJoint::Head],
        }
    );
    assert_eq!(
        error.to_string(),
        "strict retargeting: the target rig has no joint for Neck, Head"
    );
}

#[test]
fn profile_and_clip_labels_keep_scalar_wire_shapes_and_boolean_choices() {
    let profile = RigProfile::new(" profile:α \0".into())
        .with_required(HumanoidJoint::Pelvis, "ns:Hips".into())
        .with(HumanoidJoint::Head, " HEAD ".into());
    let recipe = RetargetProfile::new(profile.clone(), profile);
    let wire = serde_json::to_value(&recipe).unwrap();
    assert_eq!(wire["source"]["name"], " profile:α \0");
    assert_eq!(
        wire["source"]["joints"]["Pelvis"]["names"],
        serde_json::json!(["ns:Hips"])
    );
    assert_eq!(wire["source"]["joints"]["Pelvis"]["required"], true);
    assert_eq!(wire["source"]["joints"]["Head"]["required"], false);
    assert_eq!(wire["settings"]["strict"], false);
    assert_eq!(
        serde_json::from_value::<RetargetProfile>(wire).unwrap(),
        recipe
    );
    let mut clip = Animation::new("clip:α \0".into());
    clip.tracks.push(JointTrack::new("joint:β \0".into()));
    let wire = serde_json::to_value(&clip).unwrap();
    assert_eq!(wire["name"], "clip:α \0");
    assert_eq!(wire["tracks"][0]["joint"], "joint:β \0");
    assert_eq!(serde_json::from_value::<Animation>(wire).unwrap(), clip);
}

#[test]
fn identical_rigs_reproduce_the_source_motion() {
    let rig = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"a:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&rig, &rig, &profile).expect("profile resolves");

    let source = pose(
        &rig,
        &[
            ("a:upperarm_l".into(), quat(Vec3::new(0.0, 0.0, 1.0), 40.0)),
            ("a:lowerarm_l".into(), quat(Vec3::new(0.0, 1.0, 0.0), -30.0)),
            ("a:spine".into(), quat(Vec3::new(1.0, 0.0, 0.0), 15.0)),
        ],
    );
    let result = retargeter.pose(&source);

    assert!((retargeter.scale() - 1.0).abs() < 1.0e-5);
    for (index, joint) in rig.joints.ordinals().zip(rig.joints.iter()) {
        assert_quat_eq(
            result[index].rotation,
            source[index].rotation,
            &PoseAssertionContext::from(format!("joint {}", joint.name)),
        );
        assert_vec_eq(
            result[index].translation,
            source[index].translation,
            1.0e-4,
            &PoseAssertionContext::from(format!("joint {}", joint.name)),
        );
    }
}

#[test]
fn a_source_at_rest_leaves_the_target_at_rest() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.7, Vec3::new(0.0, 0.0, -45.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let result = retargeter.pose(&rest_pose(&source));
    for (index, joint) in target.joints.ordinals().zip(target.joints.iter()) {
        let rest = JointTransform::from_transform(&joint.local_transform);
        assert_quat_eq(
            result[index].rotation,
            rest.rotation,
            &PoseAssertionContext::from(&joint.name),
        );
        assert_vec_eq(
            result[index].translation,
            rest.translation,
            1.0e-4,
            &PoseAssertionContext::from(&joint.name),
        );
    }
}

#[test]
fn different_rest_orientations_are_corrected() {
    // A T-posed source and an A-posed target: copying local rotations across
    // would put the target's arm 45 degrees off.
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, -45.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let lift = quat(Vec3::new(1.0, 0.0, 0.0), 35.0);
    let source_pose = pose(&source, &[("a:upperarm_l".into(), lift)]);
    let result = retargeter.pose(&source_pose);

    let source_delta = model_delta(&source, &source_pose, &"a:upperarm_l".into());
    let target_delta = model_delta(&target, &result, &"b:upperarm_l".into());
    assert_quat_eq(
        target_delta,
        source_delta,
        &PoseAssertionContext::from("upper arm motion"),
    );

    // And it is genuinely a correction, not a copy of the local rotation.
    let source_local =
        source_pose[source.find_joint_by_name(&"a:upperarm_l".into()).unwrap()].rotation;
    let target_local = result[target.find_joint_by_name(&"b:upperarm_l".into()).unwrap()].rotation;
    assert!(
        target_local.dot(source_local).abs() < 0.999,
        "the target's local rotation should differ from the source's"
    );
}

#[test]
fn missing_required_joints_fail_with_a_useful_message() {
    let rig = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile =
        humanoid_profile(&"a:".into()).with_required(HumanoidJoint::Head, "a:cranium".into());
    let error = profile
        .resolve(&rig)
        .expect_err("a required joint the rig lacks must fail");
    let message = format!("{error}");
    assert!(message.contains("Head"), "{message}");
    assert!(message.contains("a:cranium"), "{message}");
}

#[test]
fn missing_optional_joints_are_reported_and_retargeting_continues() {
    let rig = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    // The rig has no toes and no fingers; neither is required.
    let profile = humanoid_profile(&"a:".into())
        .with(HumanoidJoint::ToeLeft, "a:toe_l".into())
        .with(HumanoidJoint::IndexProximalLeft, "a:index_l".into());
    let resolved = profile
        .resolve(&rig)
        .expect("optional joints may be absent");
    assert!(resolved.missing.contains(&HumanoidJoint::ToeLeft));
    assert!(resolved.missing.contains(&HumanoidJoint::IndexProximalLeft));
    assert!(resolved.has(HumanoidJoint::FootLeft));
}

#[test]
fn unmapped_source_bones_are_ignored_and_extra_target_bones_keep_their_rest_pose() {
    let mut source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let hips = source.find_joint_by_name(&"a:hips".into()).unwrap();
    source.joints.push(Joint::new(
        "a:prop".into(),
        RigJointOrdinal::from(usize::from(source.joints.count())),
        Some(hips),
        Mat4::identity(),
        Transform::from_position(Vec3::new(0.3, 0.0, 0.0)),
        Some(SkinJointOrdinal::from(usize::from(source.joints.count()))),
    ));

    let mut target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let chest = target.find_joint_by_name(&"b:chest".into()).unwrap();
    let tail_rest = Transform::new(
        Vec3::new(0.0, 0.05, -0.2),
        Vec3::new(0.0, 30.0, 0.0),
        Vec3::ones(),
    );
    target.joints.push(Joint::new(
        "b:tail".into(),
        RigJointOrdinal::from(usize::from(target.joints.count())),
        Some(chest),
        Mat4::identity(),
        tail_rest,
        Some(SkinJointOrdinal::from(usize::from(target.joints.count()))),
    ));

    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let source_pose = pose(
        &source,
        &[
            ("a:prop".into(), quat(Vec3::new(0.0, 1.0, 0.0), 90.0)),
            ("a:spine".into(), quat(Vec3::new(1.0, 0.0, 0.0), 20.0)),
        ],
    );
    let result = retargeter.pose(&source_pose);

    let tail = target.find_joint_by_name(&"b:tail".into()).unwrap();
    let rest = JointTransform::from_transform(&tail_rest);
    assert_quat_eq(
        result[tail].rotation,
        rest.rotation,
        &PoseAssertionContext::from("extra target bone"),
    );
    assert_vec_eq(
        result[tail].translation,
        rest.translation,
        1.0e-5,
        &PoseAssertionContext::from("extra target bone"),
    );

    let clip = retargeter.clip(&clip_from_pose(&source, &source_pose, 1.0));
    assert!(
        clip.track(&"b:tail".into()).is_none(),
        "an unmapped target joint should not get a track"
    );
    assert!(
        clip.tracks
            .iter()
            .all(|track| track.joint != "a:prop".into()),
        "an unmapped source joint should not leak into the output"
    );
}

/// Wraps a single pose into a two-key clip, so pose-level expectations can be
/// checked through the clip path as well.
fn clip_from_pose(skeleton: &Skeleton, locals: &LocalPose, duration: f32) -> Animation {
    let mut clip = Animation::new("test".into());
    clip.duration = duration;
    for (index, local) in locals.iter().enumerate() {
        clip.tracks.push(JointTrack {
            joint: skeleton.joints[RigJointOrdinal::from(index)].name.clone(),
            rotation: Some(Curve::new(
                vec![0.0, duration],
                vec![local.rotation, local.rotation],
            )),
            translation: Some(Curve::new(
                vec![0.0, duration],
                vec![local.translation, local.translation],
            )),
            scale: None,
        });
    }
    clip
}

#[test]
fn clip_duration_and_key_timing_survive_retargeting() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.4, Vec3::new(0.0, 0.0, -20.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());

    let mut clip = Animation::new("walk".into());
    clip.tracks.push(JointTrack {
        joint: "a:upperarm_l".into(),
        rotation: Some(Curve::new(
            vec![0.0, 0.25, 0.75, 1.5],
            vec![
                Vec4::quat_identity(),
                quat(Vec3::new(0.0, 0.0, 1.0), 20.0),
                quat(Vec3::new(0.0, 0.0, 1.0), -20.0),
                Vec4::quat_identity(),
            ],
        )),
        ..Default::default()
    });
    clip.tracks.push(JointTrack {
        joint: "a:hips".into(),
        translation: Some(Curve::new(
            vec![0.0, 1.0],
            vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.1, 0.0)],
        )),
        ..Default::default()
    });
    clip.recompute_duration();

    let retargeted = retarget(&source, &clip, &target, &profile).expect("retargeting succeeds");

    assert_eq!(retargeted.name, "walk".into());
    assert!((retargeted.duration - clip.duration).abs() < 1.0e-6);
    assert_eq!(retargeted.key_times(), clip.key_times());
    let arm = retargeted
        .track(&"b:upperarm_l".into())
        .expect("the arm is animated");
    assert_eq!(
        arm.rotation.as_ref().expect("rotation track").times,
        vec![0.0, 0.25, 0.75, 1.0, 1.5]
    );
}

#[test]
fn output_quaternions_stay_normalized() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 2.3, Vec3::new(0.0, 0.0, -45.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let source_pose = pose(
        &source,
        &[
            ("a:hips".into(), quat(Vec3::new(0.2, 1.0, 0.3), 65.0)),
            ("a:spine".into(), quat(Vec3::new(1.0, 0.0, 0.4), 25.0)),
            ("a:upperarm_r".into(), quat(Vec3::new(0.0, 0.3, 1.0), -80.0)),
            ("a:lowerleg_l".into(), quat(Vec3::new(1.0, 0.0, 0.0), 55.0)),
        ],
    );
    for local in retargeter.pose(&source_pose) {
        let length = local.rotation.dot(local.rotation).sqrt();
        assert!(
            (length - 1.0).abs() < 1.0e-5,
            "quaternion length {length} is not 1"
        );
    }
}

#[test]
fn scale_normalization_follows_the_rigs_proportions() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 2.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    // The rigs differ only in size, so pelvis-to-head gives exactly 2.
    assert!(
        (retargeter.scale() - 2.0).abs() < 1.0e-4,
        "{}",
        retargeter.scale()
    );

    // A pelvis lift of 0.1 source units becomes 0.2 target units.
    let mut source_pose = rest_pose(&source);
    let hips = source.find_joint_by_name(&"a:hips".into()).unwrap();
    source_pose[hips].translation += Vec3::new(0.0, 0.1, 0.0);

    let result = retargeter.pose(&source_pose);
    let target_hips = target.find_joint_by_name(&"b:hips".into()).unwrap();
    let rest = rest_pose(&target)[target_hips].translation;
    assert_vec_eq(
        result[target_hips].translation - rest,
        Vec3::new(0.0, 0.2, 0.0),
        1.0e-4,
        &PoseAssertionContext::from("scaled pelvis translation"),
    );
}

#[test]
fn a_fixed_scale_overrides_the_measurement() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 2.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings().with_scale(ScalePolicy::Fixed(0.5)));
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");
    assert!((retargeter.scale() - 0.5).abs() < 1.0e-5);
}

#[test]
fn limb_translations_are_ignored_so_proportions_survive() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 2.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    // A source clip that stretches the forearm must not stretch the target.
    let mut source_pose = rest_pose(&source);
    let forearm = source.find_joint_by_name(&"a:lowerarm_l".into()).unwrap();
    source_pose[forearm].translation *= 3.0;

    let result = retargeter.pose(&source_pose);
    let target_forearm = target.find_joint_by_name(&"b:lowerarm_l".into()).unwrap();
    assert_vec_eq(
        result[target_forearm].translation,
        rest_pose(&target)[target_forearm].translation,
        1.0e-5,
        &PoseAssertionContext::from("forearm translation"),
    );
}

#[test]
fn root_motion_is_separated_from_the_pose_and_scaled() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 2.0, Vec3::new(0.0, 0.0, 0.0));
    let profile =
        RetargetProfile::new(
            humanoid_profile(&"a:".into()),
            humanoid_profile(&"b:".into()),
        )
        .with_settings(RetargetSettings::default().with_root_motion(
            RootMotionPolicy::Extract(RootMotionChannels {
                horizontal: true,
                vertical: false,
                yaw: true,
            }),
        ));
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let hips = source.find_joint_by_name(&"a:hips".into()).unwrap();
    let mut source_pose = rest_pose(&source);
    source_pose[hips].translation += Vec3::new(1.0, 0.2, 0.0);
    source_pose[hips].rotation = quat(Vec3::new(0.0, 1.0, 0.0), 90.0);

    let (locals, locomotion) = retargeter.pose_with_root(&source_pose);

    // Horizontal travel and yaw left the pose ...
    assert_vec_eq(
        locomotion.translation,
        Vec3::new(2.0, 0.0, 0.0),
        1.0e-4,
        &PoseAssertionContext::from("extracted travel, in target units"),
    );
    assert_quat_eq(
        locomotion.rotation,
        quat(Vec3::new(0.0, 1.0, 0.0), 90.0),
        &PoseAssertionContext::from("extracted yaw"),
    );

    // ... while the vertical component stayed in it, scaled with the rig.
    let target_hips = target.find_joint_by_name(&"b:hips".into()).unwrap();
    let rest = rest_pose(&target)[target_hips].translation;
    assert_vec_eq(
        locals[target_hips].translation - rest,
        Vec3::new(0.0, 0.4, 0.0),
        1.0e-4,
        &PoseAssertionContext::from("vertical motion stays in the pose"),
    );
    assert_quat_eq(
        locals[target_hips].rotation,
        Vec4::quat_identity(),
        &PoseAssertionContext::from("the pelvis faces its rest direction once yaw is extracted"),
    );
}

#[test]
fn an_in_place_clip_gets_no_root_motion_track() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    );

    let source_pose = pose(
        &source,
        &[("a:spine".into(), quat(Vec3::new(1.0, 0.0, 0.0), 10.0))],
    );
    let clip = retarget(
        &source,
        &clip_from_pose(&source, &source_pose, 1.0),
        &target,
        &profile,
    )
    .expect("retargeting succeeds");
    assert!(clip.root_motion.is_none());
}

#[test]
fn a_longer_target_chain_receives_the_whole_source_bend() {
    // Two spine joints on the source, four on the target: the vocabulary
    // cannot pair these up, so the chain declaration has to.
    let source = skeleton(&[
        (
            "s:hips".into(),
            None,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:spine".into(),
            Some("s:hips".into()),
            Vec3::new(0.0, 0.2, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:chest".into(),
            Some("s:spine".into()),
            Vec3::new(0.0, 0.2, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:head".into(),
            Some("s:chest".into()),
            Vec3::new(0.0, 0.3, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
    ]);
    let target = skeleton(&[
        (
            "t:pelvis".into(),
            None,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:spine0".into(),
            Some("t:pelvis".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:spine1".into(),
            Some("t:spine0".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:spine2".into(),
            Some("t:spine1".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:spine3".into(),
            Some("t:spine2".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:head".into(),
            Some("t:spine3".into()),
            Vec3::new(0.0, 0.3, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
    ]);

    let source_profile = RigProfile::new("short spine".into())
        .with_required(HumanoidJoint::Pelvis, "s:hips".into())
        .with(HumanoidJoint::SpineLower, "s:spine".into())
        .with(HumanoidJoint::Chest, "s:chest".into())
        .with(HumanoidJoint::Head, "s:head".into())
        .with_chain(
            HumanoidChain::Spine,
            ChainBinding::new(["s:spine".into(), "s:chest".into()]),
        );
    let target_profile = RigProfile::new("long spine".into())
        .with_required(HumanoidJoint::Pelvis, "t:pelvis".into())
        .with(HumanoidJoint::SpineLower, "t:spine0".into())
        .with(HumanoidJoint::SpineMid, "t:spine1".into())
        .with(HumanoidJoint::Chest, "t:spine2".into())
        .with(HumanoidJoint::Head, "t:head".into())
        .with_chain(
            HumanoidChain::Spine,
            ChainBinding::new([
                "t:spine0".into(),
                "t:spine1".into(),
                "t:spine2".into(),
                "t:spine3".into(),
            ]),
        );

    let profile = RetargetProfile::new(source_profile, target_profile).with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let bend = quat(Vec3::new(1.0, 0.0, 0.0), 20.0);
    let source_pose = pose(
        &source,
        &[("s:spine".into(), bend), ("s:chest".into(), bend)],
    );
    let result = retargeter.pose(&source_pose);

    // Every target spine joint took a share ...
    for joint in ["t:spine0", "t:spine1", "t:spine2", "t:spine3"].map(RigJointName::from) {
        let index = target.find_joint_by_name(&joint).unwrap();
        assert!(
            result[index].rotation.w.abs() < 0.9999,
            "{joint} should carry part of the bend"
        );
    }
    // ... and the top of the chain ends up where the source's does.
    assert_quat_eq(
        model_delta(&target, &result, &"t:spine3".into()),
        model_delta(&source, &source_pose, &"s:chest".into()),
        &PoseAssertionContext::from("accumulated spine bend"),
    );
}

#[test]
fn a_shorter_target_chain_still_receives_the_full_bend() {
    let source = skeleton(&[
        (
            "s:hips".into(),
            None,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:spine0".into(),
            Some("s:hips".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:spine1".into(),
            Some("s:spine0".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "s:spine2".into(),
            Some("s:spine1".into()),
            Vec3::new(0.0, 0.1, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
    ]);
    let target = skeleton(&[
        (
            "t:hips".into(),
            None,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:spine".into(),
            Some("t:hips".into()),
            Vec3::new(0.0, 0.15, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
        (
            "t:chest".into(),
            Some("t:spine".into()),
            Vec3::new(0.0, 0.15, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ),
    ]);

    let profile = RetargetProfile::new(
        RigProfile::new("four".into())
            .with_required(HumanoidJoint::Pelvis, "s:hips".into())
            .with_chain(
                HumanoidChain::Spine,
                ChainBinding::new(["s:spine0".into(), "s:spine1".into(), "s:spine2".into()]),
            ),
        RigProfile::new("two".into())
            .with_required(HumanoidJoint::Pelvis, "t:hips".into())
            .with_chain(
                HumanoidChain::Spine,
                ChainBinding::new(["t:spine".into(), "t:chest".into()]),
            ),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let bend = quat(Vec3::new(0.0, 0.0, 1.0), 15.0);
    let source_pose = pose(
        &source,
        &[
            ("s:spine0".into(), bend),
            ("s:spine1".into(), bend),
            ("s:spine2".into(), bend),
        ],
    );
    let result = retargeter.pose(&source_pose);

    assert_quat_eq(
        model_delta(&target, &result, &"t:chest".into()),
        model_delta(&source, &source_pose, &"s:spine2".into()),
        &PoseAssertionContext::from("accumulated bend into a shorter chain"),
    );
}

#[test]
fn profiles_round_trip_through_serialization() {
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(
        settings()
            .with_translation(TranslationPolicy::PelvisOnly)
            .with_joint_translation(HumanoidJoint::HandLeft, TranslationPolicy::Scaled)
            .with_scale(ScalePolicy::Auto(ScaleMeasure::LegLength)),
    );

    let json = serde_json::to_string_pretty(&profile).expect("a profile serializes");
    let restored: RetargetProfile = serde_json::from_str(&json).expect("a profile deserializes");
    assert_eq!(profile, restored);
}

#[test]
fn clips_round_trip_through_serialization() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.6, Vec3::new(0.0, 0.0, -30.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    );

    let source_pose = pose(
        &source,
        &[("a:upperarm_l".into(), quat(Vec3::new(0.0, 0.0, 1.0), 30.0))],
    );
    let clip = retarget(
        &source,
        &clip_from_pose(&source, &source_pose, 2.0),
        &target,
        &profile,
    )
    .expect("retargeting succeeds");

    let json = serde_json::to_string(&clip).expect("a clip serializes");
    let restored: Animation = serde_json::from_str(&json).expect("a clip deserializes");
    assert_eq!(clip, restored);

    // And it still plays: sampling the restored clip reproduces the pose.
    let binding = restored.bind(&target);
    let sampled = restored.sample(&binding, 1.0);
    let direct = Retargeter::new(&source, &target, &profile)
        .expect("profile resolves")
        .pose(&source_pose);
    for (index, joint) in target.joints.ordinals().zip(target.joints.iter()) {
        assert_quat_eq(
            sampled[index].rotation,
            direct[index].rotation,
            &PoseAssertionContext::from(&joint.name),
        );
    }
}

#[test]
fn the_report_names_both_sides_of_every_mapping() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    );
    let report = Retargeter::new(&source, &target, &profile)
        .expect("profile resolves")
        .report()
        .to_string();

    assert!(report.contains("Pelvis:"), "{report}");
    assert!(report.contains("source = a:hips"), "{report}");
    assert!(report.contains("target = b:hips"), "{report}");
    assert!(report.contains("end effectors"), "{report}");
}

#[test]
fn a_rigs_armature_rotation_is_part_of_its_frame() {
    // The same rig twice, differing only in the armature transform that
    // relates its joints to the scene -- which is exactly what a
    // Blender-exported glTF does, keeping its joints Z-up and putting the
    // conversion on the armature.
    let mut source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    source.transform = Transform::from_rotation(Vec3::new(90.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));

    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let retargeter = Retargeter::new(&source, &target, &profile).expect("profile resolves");

    let lift = quat(Vec3::new(1.0, 0.0, 0.0), 40.0);
    let source_pose = pose(&source, &[("a:upperarm_l".into(), lift)]);
    let result = retargeter.pose(&source_pose);

    // The source's motion, expressed in the scene rather than in its own
    // joint space, is what should arrive on the target.
    let basis = JointTransform::from_transform(&source.transform).rotation;
    let expected = basis
        .mul_quat(model_delta(&source, &source_pose, &"a:upperarm_l".into()))
        .mul_quat(basis.conjugate());
    assert_quat_eq(
        model_delta(&target, &result, &"b:upperarm_l".into()),
        expected,
        &PoseAssertionContext::from("motion across a rotated armature"),
    );
}

#[test]
fn root_motion_is_measured_from_the_clips_own_first_frame() {
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let profile = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    );
    let hips = source.find_joint_by_name(&"a:hips".into()).unwrap();
    let rest = rest_pose(&source)[hips].translation;

    // A clip authored away from its rig's bind pose, but not travelling.
    // The offset is an authoring artifact, not locomotion.
    let mut parked = rest_pose(&source);
    parked[hips].translation = rest + Vec3::new(3.0, 0.0, -7.0);
    let clip = retarget(
        &source,
        &clip_from_pose(&source, &parked, 1.0),
        &target,
        &profile,
    )
    .expect("retargeting succeeds");
    assert!(
        clip.root_motion.is_none(),
        "a parked clip should read as in place, not as a seven metre teleport"
    );

    // A clip that does travel starts its track at zero and reports the delta.
    let mut walk = Animation::new("walk".into());
    walk.tracks.push(JointTrack {
        joint: "a:hips".into(),
        translation: Some(Curve::new(
            vec![0.0, 1.0],
            vec![
                rest + Vec3::new(3.0, 0.0, 0.0),
                rest + Vec3::new(5.0, 0.0, 0.0),
            ],
        )),
        ..Default::default()
    });
    walk.recompute_duration();

    let clip = retarget(&source, &walk, &target, &profile).expect("retargeting succeeds");
    let motion = clip.root_motion.as_ref().expect("the clip travels");
    let travel = motion.translation.as_ref().expect("a travel curve");
    assert_vec_eq(
        travel.values[0],
        Vec3::new(0.0, 0.0, 0.0),
        1.0e-4,
        &PoseAssertionContext::from("first key"),
    );
    assert_vec_eq(
        travel.values[1],
        Vec3::new(2.0, 0.0, 0.0),
        1.0e-4,
        &PoseAssertionContext::from("the delta"),
    );
}

#[test]
fn a_t_pose_reference_measures_motion_from_a_t_pose_not_the_bind_pose() {
    // A T-posed source and an A-posed target. The source at its bind pose *is*
    // a T-pose, so a target that declares the T-pose as its reference should
    // land in a T-pose too -- arms out, not down where it binds.
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(0.0, 0.0, -45.0));

    let arm_direction = |skeleton: &Skeleton, locals: &LocalPose, prefix: &RigJointPrefix| {
        let model = model_pose(skeleton, locals);
        let upper = model[skeleton
            .find_joint_by_name(&prefix.apply(&"upperarm_l".into()))
            .unwrap()]
        .translation;
        let lower = model[skeleton
            .find_joint_by_name(&prefix.apply(&"lowerarm_l".into()))
            .unwrap()]
        .translation;
        (lower - upper).normalize()
    };

    let referenced = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()).with_reference(ReferencePose::TPose),
    )
    .with_settings(settings());
    let posed = Retargeter::new(&source, &target, &referenced)
        .expect("profile resolves")
        .pose(&rest_pose(&source));
    let bone = arm_direction(&target, &posed, &"b:".into());
    assert!(
        bone.x > 0.99,
        "a T-pose reference should straighten the arm, got {bone}"
    );

    // The default is the rig's own bind pose, which leaves it where it binds.
    let bound = RetargetProfile::new(
        humanoid_profile(&"a:".into()),
        humanoid_profile(&"b:".into()),
    )
    .with_settings(settings());
    let posed = Retargeter::new(&source, &target, &bound)
        .expect("profile resolves")
        .pose(&rest_pose(&source));
    let bone = arm_direction(&target, &posed, &"b:".into());
    assert!(
        bone.x < 0.8 && bone.y < -0.5,
        "the bind reference should leave the arm down, got {bone}"
    );
}

#[test]
fn a_reference_pose_round_trips_through_serialization() {
    let profile = humanoid_profile(&"a:".into()).with_reference(ReferencePose::TPose);
    let json = serde_json::to_string(&profile).expect("it serializes");
    let restored: RigProfile = serde_json::from_str(&json).expect("it deserializes");
    assert_eq!(restored.reference, ReferencePose::TPose);
    assert_eq!(profile, restored);
}

#[test]
fn a_declared_hinge_is_rolled_onto_the_t_poses_own_hinge() {
    // Both rigs T-posed in *direction*, but the target's arm is rolled 60
    // degrees about its own bone. Straightening cannot see that: the bone
    // points the same way either way. Only the hinge declaration can.
    let source = humanoid(&"a:".into(), 1.0, Vec3::new(0.0, 0.0, 0.0));
    let target = humanoid(&"b:".into(), 1.0, Vec3::new(60.0, 0.0, -45.0));
    let hinge = Vec3::new(0.0, 0.0, 1.0);

    // The source flexes its elbow the way a T-posed body does, about -Y.
    let mut flexed = rest_pose(&source);
    flexed[source.find_joint_by_name(&"a:lowerarm_l".into()).unwrap()].rotation =
        quat(Vec3::new(0.0, -1.0, 0.0), 70.0);

    let elbow = target.find_joint_by_name(&"b:lowerarm_l".into()).unwrap();
    let target_rest = rest_pose(&target);
    let off_hinge = |profile: RigProfile| {
        let profile = RetargetProfile::new(
            humanoid_profile(&"a:".into()).with_reference(ReferencePose::TPose),
            profile.with_reference(ReferencePose::TPose),
        )
        .with_settings(settings());
        let posed = Retargeter::new(&source, &target, &profile)
            .expect("profile resolves")
            .pose(&flexed);
        let animated = target_rest[elbow]
            .rotation
            .conjugate()
            .mul_quat(posed[elbow].rotation)
            .normalize();
        let swing = animated
            .mul_quat(animated.twist_about(hinge).conjugate())
            .normalize();
        (2.0 * swing.w.abs().clamp(0.0, 1.0).acos()).to_degrees()
    };

    // Straightened but not aimed, the roll arrives as bend across the hinge --
    // motion a single-channel elbow would have to throw away.
    let unaimed = off_hinge(humanoid_profile(&"b:".into()));
    assert!(
        unaimed > 30.0,
        "the target's roll should show up as off-hinge motion, got {unaimed:.1} degrees"
    );

    // Declaring the hinge lets the reference roll the arm until the two agree.
    let aimed = off_hinge(humanoid_profile(&"b:".into()).with_joint(
        HumanoidJoint::LowerArmLeft,
        JointBinding::new("b:lowerarm_l".into()).with_hinge(hinge),
    ));
    assert!(
        aimed < 1.0,
        "a declared hinge should leave the bend on the hinge, got {aimed:.1} degrees"
    );
}
