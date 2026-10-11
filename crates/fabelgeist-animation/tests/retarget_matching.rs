//! Public consumers of the private normalized retarget lookup key.

use fabelgeist_animation::animation::retarget::{
    ChainBinding, HumanoidChain, HumanoidJoint, JointBinding, RetargetProfile, RetargetSettings,
    RetargetStrictness, Retargeter, RigProfile, RigProfileName, RootMotionPolicy, RootSource,
    ScalePolicy, TranslationPolicy, detect,
};
use fabelgeist_animation::animation::{Animation, AnimationClipName, Curve, JointTrack};
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform, vector::Vec3};

struct LookupCase {
    skeleton_names: &'static [&'static str],
    aliases: &'static [&'static str],
    expected_index: Option<usize>,
}

#[test]
fn namespaces_ascii_filtering_digits_and_side_markers_keep_lookup_policy() {
    for case in [
        LookupCase {
            skeleton_names: &["exporter:hips"],
            aliases: &["hips"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["Armature|Hips"],
            aliases: &["hips"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["outer:wrong|HIP-S"],
            aliases: &["hips"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["H_i.P-S"],
            aliases: &["hips"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["HIP-S2"],
            aliases: &["hips2"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["hips2"],
            aliases: &["hips"],
            expected_index: None,
        },
        LookupCase {
            skeleton_names: &["LeftArm", "RightArm"],
            aliases: &["arm"],
            expected_index: None,
        },
        LookupCase {
            skeleton_names: &["Left_Arm"],
            aliases: &["leftarm"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["l_arm", "r_arm"],
            aliases: &["arm"],
            expected_index: None,
        },
        LookupCase {
            skeleton_names: &["l_arm"],
            aliases: &["larm"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["éhead"],
            aliases: &["HEAD"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["β"],
            aliases: &["é"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["he\0a\td\n"],
            aliases: &["HEAD"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["β", "é"],
            aliases: &["é"],
            expected_index: Some(1),
        },
        LookupCase {
            skeleton_names: &["β", ""],
            aliases: &[""],
            expected_index: Some(1),
        },
        LookupCase {
            skeleton_names: &["β"],
            aliases: &["ns:"],
            expected_index: Some(0),
        },
    ] {
        let rig = skeleton(case.skeleton_names);
        let profile = lookup_profile(RigProfileName::from("lookup"), case.aliases);
        let resolved = profile
            .resolve(&rig)
            .expect("optional lookup admits missing role");
        assert_eq!(resolved.joint(HumanoidJoint::Pelvis), case.expected_index);
        assert_eq!(resolved.root, case.expected_index);
        assert_eq!(resolved.missing.is_empty(), case.expected_index.is_some());
        assert_eq!(
            resolved.unmapped,
            (0..rig.joints.len())
                .filter(|index| Some(*index) != case.expected_index)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn exact_duplicate_and_ordered_alias_precedence_remain_distinct() {
    for case in [
        LookupCase {
            skeleton_names: &["HIP-S", "hips"],
            aliases: &["hips"],
            expected_index: Some(1),
        },
        LookupCase {
            skeleton_names: &["hips", "hips"],
            aliases: &["hips"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["HIP-S", "hi_ps"],
            aliases: &["hip_s"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["hips", "pelvis"],
            aliases: &["HIP-S", "pelvis"],
            expected_index: Some(0),
        },
        LookupCase {
            skeleton_names: &["HIP-S", "hips"],
            aliases: &["absent", "hips"],
            expected_index: Some(1),
        },
    ] {
        let rig = skeleton(case.skeleton_names);
        let resolved = lookup_profile(RigProfileName::from("ordered"), case.aliases)
            .resolve(&rig)
            .expect("ordered alias resolves");
        assert_eq!(resolved.joint(HumanoidJoint::Pelvis), case.expected_index);
        assert_eq!(
            resolved.unmapped,
            (0..rig.joints.len())
                .filter(|index| Some(*index) != case.expected_index)
                .collect::<Vec<_>>()
        );
    }
    let profile = RigProfile::new(RigProfileName::from("ordered"))
        .with_joint(
            HumanoidJoint::Head,
            JointBinding::new("absent-head")
                .with_alias("other-head")
                .required(),
        )
        .with_required(HumanoidJoint::Pelvis, "absent-hips");
    assert_eq!(
        profile
            .resolve(&skeleton(&["hips"]))
            .unwrap_err()
            .to_string(),
        "rig profile \"ordered\" requires joints the skeleton does not have: Head (expected absent-head | other-head), Pelvis (expected absent-hips)"
    );
}

#[test]
fn chains_keep_duplicates_missing_entries_and_root_allowance() {
    let rig = skeleton(&["hips", "ns:SPINE", "head", "unused"]);
    let profile = RigProfile::new(RigProfileName::from("chain"))
        .with_required(HumanoidJoint::Pelvis, "hips")
        .with_chain(
            HumanoidChain::Spine,
            ChainBinding::new(["spine", "missing", "head", "spine"]),
        )
        .with_root(RootSource::Joint("SP_INE".to_owned()));
    let resolved = profile.resolve(&rig).expect("loose chain/root resolution");
    assert_eq!(resolved.chains[&HumanoidChain::Spine], [1, 2, 1]);
    assert_eq!(resolved.root, Some(1));
    assert_eq!(resolved.unmapped, [3]);
    let unknown_root = profile.with_root(RootSource::Joint("unknown".to_owned()));
    assert_eq!(
        unknown_root
            .resolve(&rig)
            .expect("unknown root is allowed")
            .root,
        None
    );
}

#[test]
fn markers_require_all_queries_and_builtin_detection_uses_loose_lookup() {
    let rig = skeleton(&["hips", "Left_Arm"]);
    let profile = RigProfile::new(RigProfileName::from("markers"));
    assert!(!profile.matches(&rig));
    assert!(
        profile
            .clone()
            .with_markers(["HIP-S", "ns:leftarm"])
            .matches(&rig)
    );
    assert!(!profile.with_markers(["HIP-S", "arm"]).matches(&rig));
    let detected =
        detect(&skeleton(&["HIPS", "SP_INE", "left-up-leg"])).expect("all loose Mixamo markers");
    assert_eq!(detected.name, RigProfileName::from("Mixamo"));
    assert!(detect(&skeleton(&["HIPS", "SP_INE"])).is_none());
}

#[test]
fn required_source_then_target_errors_precede_strict_transfer_errors() {
    let source_rig = skeleton(&["HIP-S", "HEAD"]);
    let target_rig = skeleton(&["hips"]);
    let source = RigProfile::new(RigProfileName::from("source"))
        .with_required(HumanoidJoint::Head, "head")
        .with_required(HumanoidJoint::Pelvis, "hips");
    let target = RigProfile::new(RigProfileName::from("target"))
        .with_required(HumanoidJoint::Pelvis, "hips");
    let strict = RetargetSettings {
        strict: RetargetStrictness::Strict,
        ..Default::default()
    };
    let source_missing = source
        .clone()
        .with_required(HumanoidJoint::Head, "absent-source");
    let target_missing = target
        .clone()
        .with_required(HumanoidJoint::Head, "absent-target");
    let source_first =
        RetargetProfile::new(source_missing, target_missing.clone()).with_settings(strict.clone());
    assert_eq!(
        source_first
            .resolve(&source_rig, &target_rig)
            .unwrap_err()
            .to_string(),
        "rig profile \"source\" requires joints the skeleton does not have: Head (expected absent-source)"
    );
    let target_next =
        RetargetProfile::new(source.clone(), target_missing).with_settings(strict.clone());
    assert_eq!(
        target_next
            .resolve(&source_rig, &target_rig)
            .unwrap_err()
            .to_string(),
        "rig profile \"target\" requires joints the skeleton does not have: Head (expected absent-target)"
    );
    let profile = RetargetProfile::new(source, target);
    assert_eq!(
        profile
            .clone()
            .with_settings(strict)
            .resolve(&source_rig, &target_rig)
            .unwrap_err()
            .to_string(),
        "strict retargeting: the target rig has no joint for Head"
    );
    assert_eq!(
        profile
            .resolve(&source_rig, &target_rig)
            .expect("permissive transfer")
            .shared_roles(),
        [HumanoidJoint::Pelvis]
    );
}

#[test]
fn stored_spellings_reports_and_real_clip_samples_survive_loose_resolution() {
    let source = skeleton(&["ns:HIP-S"]);
    let target = skeleton(&["H_iPs"]);
    let source_profile = RigProfile::new(RigProfileName::from("source"))
        .with_required(HumanoidJoint::Pelvis, "hips");
    let target_profile = RigProfile::new(RigProfileName::from("target"))
        .with_required(HumanoidJoint::Pelvis, "hips");
    let settings = RetargetSettings::default()
        .with_scale(ScalePolicy::None)
        .with_translation(TranslationPolicy::Copy)
        .with_root_motion(RootMotionPolicy::Keep);
    let profile = RetargetProfile::new(source_profile, target_profile).with_settings(settings);
    let serialized = serde_json::to_string(&profile).expect("serialize authored spelling");
    let decoded: RetargetProfile =
        serde_json::from_str(&serialized).expect("decode authored spelling");
    assert_eq!(decoded, profile);
    assert_eq!(
        decoded.source.joints[&HumanoidJoint::Pelvis].names,
        ["hips"]
    );
    let resolved = decoded
        .resolve(&source, &target)
        .expect("loose source/target resolve");
    let report = "profile: source -> target\nsource rig: source (1 joints)\ntarget rig: target (1 joints)\n\nPelvis:\n  source = ns:HIP-S\n  target = H_iPs\n";
    assert_eq!(resolved.report(&source, &target), report);
    let retargeter =
        Retargeter::new(&source, &target, &decoded).expect("public retarget construction");
    assert_eq!(
        retargeter.report(),
        format!("{report}\nscale: 1.0000 target units per source unit\n")
    );
    let mut clip = Animation::new(AnimationClipName::from("walk"));
    let mut track = JointTrack::new("ns:HIP-S");
    track.translation = Some(Curve::new(
        vec![0.0, 1.0],
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 2.0, 3.0)],
    ));
    clip.tracks.push(track);
    clip.recompute_duration();
    let output = retargeter.clip(&clip);
    assert_eq!(output.name, clip.name);
    assert_eq!(output.tracks.len(), 1);
    assert_eq!(output.tracks[0].joint, "H_iPs");
    let sample = output.sample(&output.bind(&target), 1.0);
    let translation = sample[0].translation;
    assert_eq!(
        [
            translation.x.to_bits(),
            translation.y.to_bits(),
            translation.z.to_bits()
        ],
        [1.0_f32.to_bits(), 2.0_f32.to_bits(), 3.0_f32.to_bits()]
    );
    let wire = serde_json::to_string(&output).expect("serialize transferred clip");
    let round_trip: Animation = serde_json::from_str(&wire).expect("decode transferred clip");
    assert_eq!(round_trip, output);
}

fn lookup_profile(label: RigProfileName, aliases: &[&str]) -> RigProfile {
    let binding = JointBinding {
        names: aliases.iter().map(|name| (*name).to_owned()).collect(),
        ..JointBinding::new("")
    };
    RigProfile::new(label).with_joint(HumanoidJoint::Pelvis, binding)
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
