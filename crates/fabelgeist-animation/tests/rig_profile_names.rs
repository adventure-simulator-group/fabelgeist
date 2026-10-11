//! Public profile labels, string JSON, diagnostics and retained motion.

use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, JointRequirement, RetargetProfile, RetargetSettings, RetargetStrictness,
    Retargeter, RigProfile, RigProfileName, RootMotionPolicy, ScalePolicy, TranslationPolicy,
    profiles,
};
use fabelgeist_animation::animation::{Animation, AnimationClipName, Curve, JointTrack};
use fabelgeist_animation::skeleton::mixamo::MixamoRig;
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform, vector::Vec3};

const SPELLINGS: &[&str] = &[
    "",
    "Mixamo",
    "inferred",
    "same",
    "source -> target",
    "namespace:Rig|Name",
    "  spaced  ",
    "élève 骨架🙂",
    "quote\"slash\\",
    "line\ncarriage\rtab\tNUL\0",
    "\u{2028}\u{2029}",
];

const REJECTED_PROFILES: &[RejectedProfile] = &[
    RejectedProfile {
        input: "{\"name\":null,\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: null, expected a string at line 1 column 12",
    },
    RejectedProfile {
        input: "{\"name\":false,\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: boolean `false`, expected a string at line 1 column 13",
    },
    RejectedProfile {
        input: "{\"name\":true,\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: boolean `true`, expected a string at line 1 column 12",
    },
    RejectedProfile {
        input: "{\"name\":0,\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"name\":1.5,\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: floating point `1.5`, expected a string at line 1 column 11",
    },
    RejectedProfile {
        input: "{\"name\":[],\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: sequence, expected a string at line 1 column 8",
    },
    RejectedProfile {
        input: "{\"name\":{},\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "invalid type: map, expected a string at line 1 column 8",
    },
    RejectedProfile {
        input: "{\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "missing field `name` at line 1 column 32",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"name\":\"b\",\"joints\":{},\"reference\":\"Bind\"}",
        diagnostic: "duplicate field `name` at line 1 column 18",
    },
    RejectedProfile {
        input: "{\"joints\":null,\"name\":0,\"reference\":\"Bind\"}",
        diagnostic: "invalid type: null, expected a map at line 1 column 14",
    },
    RejectedProfile {
        input: "{\"name\":0,\"joints\":null,\"reference\":\"Bind\"}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"reference\":\"Bind\"}",
        diagnostic: "missing field `joints` at line 1 column 31",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"joints\":{}}",
        diagnostic: "missing field `reference` at line 1 column 24",
    },
];

struct RejectedProfile {
    input: &'static str,
    diagnostic: &'static str,
}

#[test]
fn serialized_profiles_retain_authored_labels_through_resolution() {
    let skeleton = skeleton(&["hips"]);
    for &spelling in SPELLINGS {
        let label = RigProfileName::from(spelling.to_owned());
        let profile = rig(label.clone(), "hips");
        let assigned = RigProfile {
            name: RigProfileName::from(spelling),
            ..RigProfile::default()
        };
        let assigned = assigned.with_required(HumanoidJoint::Pelvis, "hips");
        assert_eq!(profile, assigned);
        let serialized = serde_json::to_string(&profile).unwrap();
        let native_name = serde_json::to_string(spelling).unwrap();
        assert!(serialized.starts_with(&format!("{{\"name\":{native_name},")));
        let restored: RigProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored, profile);
        let resolved = restored.resolve(&skeleton).unwrap();
        let retained: &RigProfileName = &resolved.profile;
        assert_eq!(retained, &label);
        assert_eq!(resolved.joint(HumanoidJoint::Pelvis), Some(0));
        assert_eq!(format!("{}", restored.name), spelling);
        assert_eq!(format!("{:?}", restored.name), format!("{spelling:?}"));
        assert!(
            format!("{resolved:?}").starts_with(&format!("ResolvedRig {{ profile: {spelling:?},"))
        );
    }
    let default = RigProfile::default();
    let text = serde_json::to_string(&default).unwrap();
    let restored: RigProfile = serde_json::from_str(&text).unwrap();
    assert_eq!(restored, default);
    assert_eq!(restored.name, RigProfileName::from(""));
    assert_eq!(
        restored
            .resolve(&Skeleton::new(Vec::new()))
            .unwrap()
            .profile,
        RigProfileName::from("")
    );
}

#[test]
fn profile_decoding_preserves_name_diagnostics_and_field_order() {
    for case in REJECTED_PROFILES {
        let error = serde_json::from_str::<RigProfile>(case.input).unwrap_err();
        assert_eq!(error.to_string(), case.diagnostic, "{}", case.input);
    }
    for input in [
        r#"{"name":"","joints":{},"reference":"Bind"}"#,
        r#"{"name":"line\nNUL\u0000","joints":{},"reference":"Bind"}"#,
        r#"{"name":"a","joints":{},"reference":"Bind","unknown":42}"#,
    ] {
        let profile: RigProfile = serde_json::from_str(input).unwrap();
        let text = serde_json::to_string(&profile).unwrap();
        assert_eq!(serde_json::from_str::<RigProfile>(&text).unwrap(), profile);
    }
}

#[test]
fn quoted_labels_retain_required_source_target_and_strict_error_precedence() {
    let label = RigProfileName::from("same\" 骨架\n\t\0");
    let source_skeleton = skeleton(&["hips", "head"]);
    let target_skeleton = skeleton(&["pelvis"]);
    let source = rig(label.clone(), "hips")
        .with_required(HumanoidJoint::Head, "head")
        .with_required(HumanoidJoint::Chest, "absent-source-chest");
    let target =
        rig(label.clone(), "pelvis").with_required(HumanoidJoint::Head, "absent-target-head");
    let strict = RetargetSettings {
        strict: RetargetStrictness::Strict,
        ..Default::default()
    };
    let error = RetargetProfile::new(source, target.clone())
        .with_settings(strict.clone())
        .resolve(&source_skeleton, &target_skeleton)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        format!(
            "rig profile {label:?} requires joints the skeleton does not have: Chest (expected absent-source-chest)"
        )
    );
    let source = rig(label.clone(), "hips").with_required(HumanoidJoint::Head, "head");
    let error = RetargetProfile::new(source.clone(), target)
        .with_settings(strict.clone())
        .resolve(&source_skeleton, &target_skeleton)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        format!(
            "rig profile {label:?} requires joints the skeleton does not have: Head (expected absent-target-head)"
        )
    );
    let transfer = RetargetProfile::new(source, rig(label, "pelvis"));
    let error = transfer
        .clone()
        .with_settings(strict)
        .resolve(&source_skeleton, &target_skeleton)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "strict retargeting: the target rig has no joint for Head"
    );
    assert!(transfer.resolve(&source_skeleton, &target_skeleton).is_ok());
}

#[test]
fn builtin_and_inferred_profiles_keep_their_labels_and_joint_requirements() {
    let mixamo = MixamoRig::skeleton();
    let detected = profiles::detect(&mixamo).unwrap();
    assert_eq!(detected.name, RigProfileName::from("Mixamo"));
    assert_eq!(
        detected.resolve(&mixamo).unwrap().profile,
        RigProfileName::from("Mixamo")
    );
    let skeleton = skeleton(&["hips", "head"]);
    let inferred = RigProfile::infer(&skeleton);
    assert_eq!(inferred.name, RigProfileName::from("inferred"));
    assert_eq!(
        inferred.joints[&HumanoidJoint::Pelvis].required,
        JointRequirement::Required
    );
    let resolved = inferred.resolve(&skeleton).unwrap();
    assert_eq!(resolved.profile, RigProfileName::from("inferred"));
    assert_eq!(resolved.joint(HumanoidJoint::Pelvis), Some(0));
}

#[test]
fn reports_display_labels_while_retargeted_clip_and_float_words_stay_equal() {
    let source = skeleton(&["hips"]);
    let target = skeleton(&["pelvis"]);
    let settings = RetargetSettings::default()
        .with_scale(ScalePolicy::None)
        .with_translation(TranslationPolicy::Copy)
        .with_root_motion(RootMotionPolicy::Keep);
    let mut clip = Animation::new(AnimationClipName::from("walk\0 namespace"));
    let mut track = JointTrack::new("hips");
    track.translation = Some(Curve::new(
        vec![0.0, 0.5, 1.0],
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.25, 0.5, -0.5),
            Vec3::ones(),
        ],
    ));
    clip.tracks.push(track);
    clip.recompute_duration();
    let control = RetargetProfile::new(
        rig(RigProfileName::from("source"), "hips"),
        rig(RigProfileName::from("target"), "pelvis"),
    )
    .with_settings(settings.clone());
    let control = Retargeter::new(&source, &target, &control)
        .unwrap()
        .clip(&clip);
    let control_binding = control.bind(&target);
    for &spelling in SPELLINGS {
        let label = RigProfileName::from(spelling);
        let transfer =
            RetargetProfile::new(rig(label.clone(), "hips"), rig(label.clone(), "pelvis"))
                .with_settings(settings.clone());
        assert_eq!(transfer.name, format!("{spelling} -> {spelling}"));
        let retargeter = Retargeter::new(&source, &target, &transfer).unwrap();
        assert_eq!(retargeter.resolved().source.profile, label);
        assert_eq!(retargeter.resolved().target.profile, label);
        let prefix = format!(
            "profile: {spelling} -> {spelling}\nsource rig: {spelling} (1 joints)\ntarget rig: {spelling} (1 joints)\n\n"
        );
        assert!(retargeter.report().starts_with(&prefix));
        assert!(
            retargeter
                .resolved()
                .report(&source, &target)
                .starts_with(&prefix)
        );
        let output = retargeter.clip(&clip);
        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            serde_json::to_string(&control).unwrap()
        );
        assert_eq!(output.name, clip.name);
        assert_eq!(output.duration.to_bits(), control.duration.to_bits());
        let binding = output.bind(&target);
        for time in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let sample = output.sample(&binding, time);
            let reference = control.sample(&control_binding, time);
            assert_eq!(sample, reference);
            assert_eq!(
                [
                    sample[0].translation.x.to_bits(),
                    sample[0].translation.y.to_bits(),
                    sample[0].translation.z.to_bits()
                ],
                [
                    reference[0].translation.x.to_bits(),
                    reference[0].translation.y.to_bits(),
                    reference[0].translation.z.to_bits()
                ]
            );
        }
    }
}

fn rig(label: RigProfileName, joint: &str) -> RigProfile {
    RigProfile::new(label).with_required(HumanoidJoint::Pelvis, joint)
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
