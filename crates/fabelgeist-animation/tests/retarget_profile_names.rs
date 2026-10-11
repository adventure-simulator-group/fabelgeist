//! Public recipe labels, native text boundaries and retained transfer behavior.

use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, RetargetProfile, RetargetProfileName, RetargetSettings, RetargetStrictness,
    Retargeter, RigProfile, RigProfileName, RootMotionPolicy, ScalePolicy, TranslationPolicy,
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
    "  ",
];

const REJECTED_PROFILES: &[RejectedProfile] = &[
    RejectedProfile {
        input: "{\"name\":null,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: null, expected a string at line 1 column 12",
    },
    RejectedProfile {
        input: "{\"name\":false,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: boolean `false`, expected a string at line 1 column 13",
    },
    RejectedProfile {
        input: "{\"name\":true,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: boolean `true`, expected a string at line 1 column 12",
    },
    RejectedProfile {
        input: "{\"name\":0,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"name\":1.5,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: floating point `1.5`, expected a string at line 1 column 11",
    },
    RejectedProfile {
        input: "{\"name\":[],\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: sequence, expected a string at line 1 column 8",
    },
    RejectedProfile {
        input: "{\"name\":{},\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: map, expected a string at line 1 column 8",
    },
    RejectedProfile {
        input: "{\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "missing field `name` at line 1 column 105",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"name\":\"b\",\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "duplicate field `name` at line 1 column 18",
    },
    RejectedProfile {
        input: "{\"source\":null,\"name\":0,\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: null, expected struct RigProfile at line 1 column 14",
    },
    RejectedProfile {
        input: "{\"name\":0,\"source\":null,\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"target\":null,\"name\":0,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: null, expected struct RigProfile at line 1 column 14",
    },
    RejectedProfile {
        input: "{\"name\":0,\"target\":null,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"settings\":null,\"name\":0,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: null, expected struct RetargetSettings at line 1 column 16",
    },
    RejectedProfile {
        input: "{\"name\":0,\"settings\":null,\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "invalid type: integer `0`, expected a string at line 1 column 9",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "missing field `source` at line 1 column 64",
    },
    RejectedProfile {
        input: "{\"name\":\"a\",\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
        diagnostic: "missing field `target` at line 1 column 64",
    },
    RejectedProfile {
        input: "[\"short\",{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}]",
        diagnostic: "invalid length 2, expected struct RetargetProfile with 4 elements at line 1 column 52",
    },
    RejectedProfile {
        input: "[\"extra\",{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},{\"scale\":\"None\",\"translation\":\"Copy\",\"root_motion\":\"Keep\",\"strict\":false},42]",
        diagnostic: "trailing characters at line 1 column 170",
    },
    RejectedProfile {
        input: "null",
        diagnostic: "invalid type: null, expected struct RetargetProfile at line 1 column 4",
    },
];

const VALID_PROFILES: &[&str] = &[
    "{\"name\":\"\",\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
    "{\"name\":\"line\\ncarriage\\rtab\\tNUL\\u0000\\u9aa8\\u67b6\",\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}}",
    "{\"name\":\"authored\",\"source\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"target\":{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},\"unknown\":42}",
    "[\"sequence\",{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},{\"scale\":\"None\",\"translation\":\"Copy\",\"root_motion\":\"Keep\",\"strict\":false}]",
    "[\"sequence-default\",{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"},{\"name\":\"\",\"joints\":{},\"reference\":\"Bind\"}]",
];

struct RejectedProfile {
    input: &'static str,
    diagnostic: &'static str,
}

#[test]
fn authored_labels_round_trip_and_remain_typed_through_resolution() {
    let source = skeleton(&["hips"]);
    let target = skeleton(&["pelvis"]);
    for &spelling in SPELLINGS {
        let label = RetargetProfileName::from(spelling.to_owned());
        let profile = recipe(label.clone());
        let serialized = serde_json::to_string(&profile).unwrap();
        let native = serde_json::to_string(spelling).unwrap();
        assert!(serialized.starts_with(&format!("{{\"name\":{native},")));
        let decoded: RetargetProfile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded, profile);
        let leaf: RetargetProfileName = serde_json::from_str(&native).unwrap();
        assert_eq!(leaf, label);
        assert_eq!(serde_json::to_string(&leaf).unwrap(), native);
        assert_eq!(format!("{}", decoded.name), spelling);
        assert_eq!(format!("{:?}", decoded.name), format!("{spelling:?}"));
        assert!(
            format!("{decoded:?}").starts_with(&format!("RetargetProfile {{ name: {spelling:?},"))
        );
        let resolved = decoded.resolve(&source, &target).unwrap();
        let retained: &RetargetProfileName = &resolved.name;
        assert_eq!(retained, &label);
        assert!(
            format!("{resolved:?}").starts_with(&format!("ResolvedProfile {{ name: {spelling:?},"))
        );
        assert_eq!(resolved.source.profile, RigProfileName::from("source"));
        assert_eq!(resolved.target.profile, RigProfileName::from("target"));
    }
}

#[test]
fn generated_labels_snapshot_rigs_and_keep_default_distinction() {
    let empty = Skeleton::new(Vec::new());
    let default = RetargetProfile::default();
    assert_eq!(default.name, RetargetProfileName::from(""));
    assert_eq!(default.resolve(&empty, &empty).unwrap().name, default.name);
    let generated = RetargetProfile::new(RigProfile::default(), RigProfile::default());
    assert_eq!(generated.name, RetargetProfileName::from(" -> "));
    assert_eq!(
        generated.resolve(&empty, &empty).unwrap().name,
        generated.name
    );
    let source_label = RigProfileName::from("source -> first\n骨架");
    let target_label = RigProfileName::from("target\"\\\t");
    let label = RetargetProfileName::from_rig_labels(&source_label, &target_label);
    assert_eq!(
        label,
        RetargetProfileName::from("source -> first\n骨架 -> target\"\\\t")
    );
    let mut profile =
        RetargetProfile::new(RigProfile::new(source_label), RigProfile::new(target_label));
    profile.source.name = RigProfileName::from("renamed source");
    profile.target.name = RigProfileName::from("renamed target\n");
    assert_eq!(profile.name, label);
    let serialized = serde_json::to_string(&profile).unwrap();
    let restored: RetargetProfile = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored, profile);
    let resolved = restored.resolve(&empty, &empty).unwrap();
    assert_eq!(resolved.name, label);
    assert_eq!(resolved.source.profile, profile.source.name);
    assert_eq!(resolved.target.profile, profile.target.name);
    assert!(
        resolved
            .report(&empty, &empty)
            .starts_with(&format!("profile: {label}\n"))
    );
}

#[test]
fn decoding_retains_required_text_diagnostics_field_order_and_settings() {
    for case in REJECTED_PROFILES {
        let error = serde_json::from_str::<RetargetProfile>(case.input).unwrap_err();
        assert_eq!(error.to_string(), case.diagnostic, "{}", case.input);
    }
    for &input in VALID_PROFILES {
        let profile: RetargetProfile = serde_json::from_str(input).unwrap();
        let text = serde_json::to_string(&profile).unwrap();
        assert_eq!(
            serde_json::from_str::<RetargetProfile>(&text).unwrap(),
            profile
        );
        let empty = Skeleton::new(Vec::new());
        assert_eq!(profile.resolve(&empty, &empty).unwrap().name, profile.name);
        if input.starts_with("[\"sequence\"") {
            assert_eq!(profile.settings.translation, TranslationPolicy::Copy);
            assert_eq!(profile.settings.scale, ScalePolicy::None);
            assert_eq!(profile.settings.root_motion, RootMotionPolicy::Keep);
        } else {
            assert_eq!(profile.settings, RetargetSettings::default());
        }
    }
}

#[test]
fn authored_recipe_labels_preserve_required_errors_role_order_and_strictness() {
    let source = skeleton(&["hips", "head"]);
    let target = skeleton(&["pelvis"]);
    let rig_label = RigProfileName::from("rig\" 骨架\n\t\0");
    let source_rig = rig(rig_label.clone(), "hips")
        .with_required(HumanoidJoint::Head, "head")
        .with_required(HumanoidJoint::Chest, "absent-chest");
    let target_rig =
        rig(rig_label.clone(), "pelvis").with_required(HumanoidJoint::Head, "absent-head");
    for &spelling in SPELLINGS {
        let strict = RetargetSettings {
            strict: RetargetStrictness::Strict,
            ..Default::default()
        };
        let profile = RetargetProfile {
            name: RetargetProfileName::from(spelling),
            settings: strict.clone(),
            ..RetargetProfile::new(source_rig.clone(), target_rig.clone())
        };
        assert_eq!(
            profile.resolve(&source, &target).unwrap_err().to_string(),
            format!(
                "rig profile {rig_label:?} requires joints the skeleton does not have: Chest (expected absent-chest)"
            )
        );
        let source_rig = rig(rig_label.clone(), "hips").with_required(HumanoidJoint::Head, "head");
        let profile = RetargetProfile {
            source: source_rig,
            ..profile
        };
        assert_eq!(
            profile.resolve(&source, &target).unwrap_err().to_string(),
            format!(
                "rig profile {rig_label:?} requires joints the skeleton does not have: Head (expected absent-head)"
            )
        );
        let profile = RetargetProfile {
            target: rig(rig_label.clone(), "pelvis"),
            ..profile
        };
        assert_eq!(
            profile.resolve(&source, &target).unwrap_err().to_string(),
            "strict retargeting: the target rig has no joint for Head"
        );
        let profile = profile.with_settings(RetargetSettings::default());
        let resolved = profile.resolve(&source, &target).unwrap();
        assert_eq!(resolved.name, RetargetProfileName::from(spelling));
        assert_eq!(
            resolved.source.joints.keys().copied().collect::<Vec<_>>(),
            [HumanoidJoint::Pelvis, HumanoidJoint::Head]
        );
        assert_eq!(
            resolved.target.joints.keys().copied().collect::<Vec<_>>(),
            [HumanoidJoint::Pelvis]
        );
    }
}

#[test]
fn complete_reports_and_clip_float_words_ignore_authored_recipe_labels() {
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
    let control_profile =
        recipe(RetargetProfileName::from("control")).with_settings(settings.clone());
    let control = Retargeter::new(&source, &target, &control_profile).unwrap();
    let control_report = control.report();
    let control_mapping_report = control.resolved().report(&source, &target);
    let output_control = control.clip(&clip);
    let control_binding = output_control.bind(&target);
    for &spelling in SPELLINGS {
        let profile = recipe(RetargetProfileName::from(spelling)).with_settings(settings.clone());
        let retargeter = Retargeter::new(&source, &target, &profile).unwrap();
        assert_eq!(
            retargeter.report(),
            format!(
                "profile: {spelling}\n{}",
                control_report.split_once('\n').unwrap().1
            )
        );
        assert_eq!(
            retargeter.resolved().report(&source, &target),
            format!(
                "profile: {spelling}\n{}",
                control_mapping_report.split_once('\n').unwrap().1
            )
        );
        assert_eq!(retargeter.resolved().name, profile.name);
        let output = retargeter.clip(&clip);
        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            serde_json::to_string(&output_control).unwrap()
        );
        assert_eq!(output.name, clip.name);
        assert_eq!(output.duration.to_bits(), output_control.duration.to_bits());
        assert_eq!(
            output
                .key_times()
                .iter()
                .map(|time| time.to_bits())
                .collect::<Vec<_>>(),
            output_control
                .key_times()
                .iter()
                .map(|time| time.to_bits())
                .collect::<Vec<_>>()
        );
        let binding = output.bind(&target);
        assert_eq!(binding.tracks, control_binding.tracks);
        for time in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let sample = output.sample(&binding, time);
            let reference = output_control.sample(&control_binding, time);
            assert_eq!(sample, reference);
            assert_eq!(
                sample.iter().flat_map(transform_words).collect::<Vec<_>>(),
                reference
                    .iter()
                    .flat_map(transform_words)
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn builtin_to_inferred_transfer_keeps_the_generated_label() {
    let source = MixamoRig::skeleton();
    let target = skeleton(&["hips", "head"]);
    let profile = RetargetProfile::new(MixamoRig::profile(), RigProfile::infer(&target));
    assert_eq!(
        profile.name,
        RetargetProfileName::from("Mixamo -> inferred")
    );
    let resolved = profile.resolve(&source, &target).unwrap();
    assert_eq!(resolved.name, profile.name);
    assert_eq!(resolved.source.profile, RigProfileName::from("Mixamo"));
    assert_eq!(resolved.target.profile, RigProfileName::from("inferred"));
    assert_eq!(resolved.target.joint(HumanoidJoint::Pelvis), Some(0));
}

fn recipe(label: RetargetProfileName) -> RetargetProfile {
    RetargetProfile {
        name: label,
        ..RetargetProfile::new(
            rig(RigProfileName::from("source"), "hips"),
            rig(RigProfileName::from("target"), "pelvis"),
        )
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

fn transform_words(transform: &fabelgeist_animation::animation::JointTransform) -> [u32; 10] {
    [
        transform.translation.x.to_bits(),
        transform.translation.y.to_bits(),
        transform.translation.z.to_bits(),
        transform.rotation.x.to_bits(),
        transform.rotation.y.to_bits(),
        transform.rotation.z.to_bits(),
        transform.rotation.w.to_bits(),
        transform.scale.x.to_bits(),
        transform.scale.y.to_bits(),
        transform.scale.z.to_bits(),
    ]
}
