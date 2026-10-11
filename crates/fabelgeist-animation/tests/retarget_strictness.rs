//! Public transfer policy, Boolean JSON and resolution behavior.

use fabelgeist_animation::animation::retarget::{
    ChainBinding, HumanoidChain, HumanoidJoint, JointBinding, RetargetProfile, RetargetSettings,
    RetargetStrictness, Retargeter, RigProfile, RigProfileName, RootMotionPolicy, RootSource,
    ScalePolicy, TranslationPolicy,
};
use fabelgeist_animation::animation::{Animation, AnimationClipName, Curve, JointTrack};
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform, vector::Vec3};

const REJECTED_SETTINGS: &[(&str, &str)] = &[
    (
        "{\"strict\":null}",
        "invalid type: null, expected a boolean at line 1 column 14",
    ),
    (
        "{\"strict\":1}",
        "invalid type: integer `1`, expected a boolean at line 1 column 11",
    ),
    (
        "{\"strict\":\"Strict\"}",
        "invalid type: string \"Strict\", expected a boolean at line 1 column 18",
    ),
    (
        "{\"strict\":[]}",
        "invalid type: sequence, expected a boolean at line 1 column 10",
    ),
    (
        "{\"strict\":{}}",
        "invalid type: map, expected a boolean at line 1 column 10",
    ),
    (
        "{\"strict\":false,\"strict\":true}",
        "duplicate field `strict` at line 1 column 24",
    ),
    (
        "{\"translation\":\"invalid\",\"strict\":null}",
        "unknown variant `invalid`, expected one of `Ignore`, `Copy`, `Scaled`, `RootOnly`, `PelvisOnly` at line 1 column 24",
    ),
    (
        "{\"strict\":null,\"translation\":\"invalid\"}",
        "invalid type: null, expected a boolean at line 1 column 14",
    ),
    (
        "null",
        "invalid type: null, expected struct RetargetSettings at line 1 column 4",
    ),
];

#[test]
fn boolean_settings_round_trips_retain_the_transfer_policy() {
    for (input, policy) in [
        ("{}", RetargetStrictness::Permissive),
        (r#"{"strict":false}"#, RetargetStrictness::Permissive),
        (r#"{"strict":true}"#, RetargetStrictness::Strict),
    ] {
        let settings: RetargetSettings = serde_json::from_str(input).unwrap();
        assert_eq!(settings.strict, policy);
        let serialized = serde_json::to_string(&settings).unwrap();
        let round_trip: RetargetSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(round_trip, settings);
        assert_eq!(
            serde_json::to_value(&settings).unwrap()["strict"],
            policy == RetargetStrictness::Strict
        );
        let profile = RetargetProfile::new(source(), target()).with_settings(round_trip);
        let profile_json = serde_json::to_string(&profile).unwrap();
        let decoded: RetargetProfile = serde_json::from_str(&profile_json).unwrap();
        assert_eq!(decoded, profile);
        let source_skeleton = skeleton(&["hips"]);
        let target_skeleton = skeleton(&["pelvis"]);
        assert_eq!(
            decoded
                .resolve(&source_skeleton, &target_skeleton)
                .unwrap()
                .settings,
            settings
        );
        assert_eq!(
            Retargeter::new(&source_skeleton, &target_skeleton, &decoded)
                .unwrap()
                .settings(),
            &settings
        );
    }
    let mut value = serde_json::to_value(RetargetProfile::new(source(), target())).unwrap();
    value.as_object_mut().unwrap().remove("settings");
    let omitted: RetargetProfile = serde_json::from_value(value).unwrap();
    assert_eq!(omitted.settings.strict, RetargetStrictness::Permissive);
    assert_eq!(
        RetargetProfile::default().settings.strict,
        RetargetStrictness::Permissive
    );
}

#[test]
fn invalid_boolean_payloads_keep_native_diagnostics_and_field_order() {
    for (input, diagnostic) in REJECTED_SETTINGS {
        assert_eq!(
            serde_json::from_str::<RetargetSettings>(input)
                .unwrap_err()
                .to_string(),
            *diagnostic
        );
    }
    let mut value = serde_json::to_value(RetargetProfile::new(source(), target())).unwrap();
    value["settings"]["strict"] = serde_json::json!("Strict");
    assert!(
        serde_json::from_value::<RetargetProfile>(value)
            .unwrap_err()
            .to_string()
            .contains("expected a boolean")
    );
}

#[test]
fn required_rig_errors_precede_strict_role_omissions() {
    let settings = strict_settings();
    let missing_source = RigProfile::new(RigProfileName::from("source-missing"))
        .with_required(HumanoidJoint::Pelvis, "hips");
    let missing_target = RigProfile::new(RigProfileName::from("target-missing"))
        .with_required(HumanoidJoint::Head, "head");
    let profile = RetargetProfile::new(missing_source.clone(), missing_target.clone())
        .with_settings(settings.clone());
    assert_eq!(
        profile
            .resolve(&skeleton(&[]), &skeleton(&[]))
            .unwrap_err()
            .to_string(),
        "rig profile \"source-missing\" requires joints the skeleton does not have: Pelvis (expected hips)"
    );
    let profile = RetargetProfile::new(missing_source, missing_target).with_settings(settings);
    assert_eq!(
        profile
            .resolve(&skeleton(&["hips"]), &skeleton(&[]))
            .unwrap_err()
            .to_string(),
        "rig profile \"target-missing\" requires joints the skeleton does not have: Head (expected head)"
    );
}

#[test]
fn resolved_source_roles_control_strictness_and_order() {
    let ordered = RigProfile::new(RigProfileName::from("ordered"))
        .with(HumanoidJoint::Head, "head")
        .with_required(HumanoidJoint::Pelvis, "hips")
        .with(HumanoidJoint::HandLeft, "hand");
    let source_skeleton = skeleton(&["hips", "head", "hand"]);
    let strict = RetargetProfile::new(
        ordered.clone(),
        RigProfile::new(RigProfileName::from("empty")),
    )
    .with_settings(strict_settings());
    assert_eq!(
        strict
            .resolve(&source_skeleton, &skeleton(&[]))
            .unwrap_err()
            .to_string(),
        "strict retargeting: the target rig has no joint for Head, Pelvis, HandLeft"
    );
    let permissive = RetargetProfile::new(
        ordered.clone(),
        RigProfile::new(RigProfileName::from("empty")),
    );
    assert!(
        permissive
            .resolve(&source_skeleton, &skeleton(&[]))
            .unwrap()
            .shared_roles()
            .is_empty()
    );
    let complete_target = RigProfile::new(RigProfileName::from("target"))
        .with(HumanoidJoint::HandLeft, "hand")
        .with(HumanoidJoint::Head, "head")
        .with_required(HumanoidJoint::Pelvis, "pelvis");
    let complete = RetargetProfile::new(ordered, complete_target).with_settings(strict_settings());
    assert_eq!(
        complete
            .resolve(&source_skeleton, &skeleton(&["head", "pelvis", "hand"]))
            .unwrap()
            .shared_roles(),
        [
            HumanoidJoint::Pelvis,
            HumanoidJoint::Head,
            HumanoidJoint::HandLeft
        ]
    );
    let optional = source().with(HumanoidJoint::Head, "absent-head");
    let extra_target = target().with(HumanoidJoint::HandLeft, "hand");
    let profile = RetargetProfile::new(optional, extra_target).with_settings(strict_settings());
    let resolved = profile
        .resolve(&skeleton(&["hips"]), &skeleton(&["pelvis", "hand"]))
        .unwrap();
    assert_eq!(resolved.source.missing, [HumanoidJoint::Head]);
    assert_eq!(resolved.shared_roles(), [HumanoidJoint::Pelvis]);
}

#[test]
fn strict_transfer_keeps_matching_chain_filtering_and_missing_root_allowance() {
    let source = RigProfile::new(RigProfileName::from("source"))
        .with_joint(
            HumanoidJoint::Pelvis,
            JointBinding::new("absent").required().with_alias("hips"),
        )
        .with_chain(
            HumanoidChain::Spine,
            ChainBinding {
                joints: vec!["absent-before".into(), "hips".into(), "absent-after".into()],
            },
        )
        .with_root(RootSource::Joint("absent-root".into()));
    let profile = RetargetProfile::new(source, target()).with_settings(strict_settings());
    let resolved = profile
        .resolve(
            &skeleton(&["HIP-S", "hips", "hips"]),
            &skeleton(&["pelvis"]),
        )
        .unwrap();
    assert_eq!(resolved.source.joint(HumanoidJoint::Pelvis), Some(1));
    assert_eq!(resolved.source.chains[&HumanoidChain::Spine], [1]);
    assert_eq!(resolved.source.root, None);
}

#[test]
fn unused_resolved_roles_are_rejected_without_changing_shared_clip_output() {
    let source_skeleton = skeleton(&["hips", "unused-head"]);
    let target_skeleton = skeleton(&["pelvis"]);
    let mut clip = Animation::new(AnimationClipName::from("strictness:walk"));
    let mut track = JointTrack::new("hips");
    track.translation = Some(Curve::new(
        vec![0.0, 0.5, 1.0],
        vec![
            Vec3::new(-0.0, 1.25, -2.5),
            Vec3::new(2.0, 1.5, -1.0),
            Vec3::new(4.0, 3.0, 0.5),
        ],
    ));
    clip.tracks.push(track);
    clip.recompute_duration();
    let settings = RetargetSettings::default()
        .with_scale(ScalePolicy::None)
        .with_translation(TranslationPolicy::Copy)
        .with_root_motion(RootMotionPolicy::Keep);
    let extra_source = source().with(HumanoidJoint::Head, "unused-head");
    let permissive =
        RetargetProfile::new(extra_source.clone(), target()).with_settings(settings.clone());
    let output = Retargeter::new(&source_skeleton, &target_skeleton, &permissive)
        .unwrap()
        .clip(&clip);
    assert_eq!(output.name, clip.name);
    assert_eq!(output.tracks[0].joint, "pelvis");
    let mut strict = settings;
    strict.strict = RetargetStrictness::Strict;
    let rejected = RetargetProfile::new(extra_source, target()).with_settings(strict.clone());
    assert_eq!(
        Retargeter::new(&source_skeleton, &target_skeleton, &rejected)
            .err()
            .unwrap()
            .to_string(),
        "strict retargeting: the target rig has no joint for Head"
    );
    let shared_only = RetargetProfile::new(source(), target()).with_settings(strict);
    let strict_output = Retargeter::new(&source_skeleton, &target_skeleton, &shared_only)
        .unwrap()
        .clip(&clip);
    assert_eq!(
        serde_json::to_string(&strict_output).unwrap(),
        serde_json::to_string(&output).unwrap()
    );
    let binding = output.bind(&target_skeleton);
    let strict_binding = strict_output.bind(&target_skeleton);
    for time in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let sample = output.sample(&binding, time);
        let strict_sample = strict_output.sample(&strict_binding, time);
        assert_eq!(
            sample[0].translation.x.to_bits(),
            strict_sample[0].translation.x.to_bits()
        );
        assert_eq!(
            sample[0].translation.y.to_bits(),
            strict_sample[0].translation.y.to_bits()
        );
        assert_eq!(
            sample[0].translation.z.to_bits(),
            strict_sample[0].translation.z.to_bits()
        );
    }
}

fn strict_settings() -> RetargetSettings {
    RetargetSettings {
        strict: RetargetStrictness::Strict,
        ..Default::default()
    }
}

fn source() -> RigProfile {
    RigProfile::new(RigProfileName::from("source")).with_required(HumanoidJoint::Pelvis, "hips")
}

fn target() -> RigProfile {
    RigProfile::new(RigProfileName::from("target")).with_required(HumanoidJoint::Pelvis, "pelvis")
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
