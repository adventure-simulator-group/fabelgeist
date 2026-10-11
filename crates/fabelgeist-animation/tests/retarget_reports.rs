//! Public consumers of completed retarget mapping diagnostics.

use fabelgeist_animation::animation::retarget::{
    ChainBinding, HumanoidChain, HumanoidJoint, RetargetProfile, RetargetReport, RetargetSettings,
    Retargeter, RigProfile, RigProfileName, ScalePolicy,
};
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform};

#[test]
fn both_public_producers_return_typed_empty_reports() {
    let empty = Skeleton::new(Vec::new());
    let profile = RetargetProfile::new(
        RigProfile::new(RigProfileName::from("")),
        RigProfile::new(RigProfileName::from("")),
    );
    let resolved = profile.resolve(&empty, &empty).unwrap();
    let mapping: RetargetReport = resolved.report(&empty, &empty);
    let complete: RetargetReport = Retargeter::new(&empty, &empty, &profile).unwrap().report();
    assert_eq!(mapping, mapping.clone());
    assert_ne!(mapping, complete);
    assert_eq!(
        mapping.to_string(),
        "profile:  -> \nsource rig:  (0 joints)\ntarget rig:  (0 joints)\n\n"
    );
    assert_eq!(
        complete.to_string(),
        format!("{mapping}\nscale: 1.0000 target units per source unit\n")
    );
}

#[test]
fn arbitrary_labels_and_joint_spellings_keep_display_and_string_debug() {
    let source_name = "source \"\\ 雪\t\n\0";
    let target_name = "target 雪\0";
    let source = skeleton(&[source_name]);
    let target = skeleton(&[target_name]);
    for label in ["", "recipe \"\\ 雪\t\n\0"] {
        let profile = RetargetProfile::new(
            RigProfile::new(RigProfileName::from(label))
                .with_required(HumanoidJoint::Pelvis, source_name),
            RigProfile::new(RigProfileName::from(label))
                .with_required(HumanoidJoint::Pelvis, target_name),
        );
        let mapping = profile
            .resolve(&source, &target)
            .unwrap()
            .report(&source, &target);
        let expected = format!(
            "profile: {label} -> {label}\nsource rig: {label} (1 joints)\ntarget rig: {label} (1 joints)\n\nPelvis:\n  source = {source_name}\n  target = {target_name}\n"
        );
        assert_eq!(mapping.to_string(), expected);
        assert_eq!(format!("{mapping:?}"), format!("{expected:?}"));
        assert_eq!(format!("{mapping:#?}"), format!("{expected:#?}"));
    }
}

#[test]
fn role_and_missing_list_orders_keep_unmapped_counterparts() {
    let source = skeleton(&["hips", "head"]);
    let target = skeleton(&["pelvis"]);
    let profile = RetargetProfile::new(
        RigProfile::new(RigProfileName::from("source"))
            .with(HumanoidJoint::HandLeft, "missing hand")
            .with(HumanoidJoint::Head, "head")
            .with_required(HumanoidJoint::Pelvis, "hips")
            .with(HumanoidJoint::Chest, "missing chest"),
        RigProfile::new(RigProfileName::from("target"))
            .with(HumanoidJoint::FootRight, "missing foot")
            .with_required(HumanoidJoint::Pelvis, "pelvis"),
    );
    let report = profile
        .resolve(&source, &target)
        .unwrap()
        .report(&source, &target);
    assert_eq!(
        report.to_string(),
        "profile: source -> target\nsource rig: source (2 joints)\ntarget rig: target (1 joints)\n\nPelvis:\n  source = hips\n  target = pelvis\nHead:\n  source = head\n  target = <unmapped>\n\nsource rig is missing: HandLeft, Chest\ntarget rig is missing: FootRight\n"
    );
}

#[test]
fn only_shared_unequal_chains_render_original_joint_order() {
    let source = skeleton(&["source:a", "source:b"]);
    let target = skeleton(&["target:a", "target:b", "target:c"]);
    let source_profile = RigProfile::new(RigProfileName::from("source")).with_chain(
        HumanoidChain::Spine,
        ChainBinding::new(["source:a", "source:b"]),
    );
    for (names, suffix) in [
        (
            vec!["target:a", "target:b", "target:c"],
            "\nchain Spine (2 source joints, 3 target joints, motion distributed)\n  source = source:a -> source:b\n  target = target:a -> target:b -> target:c\n",
        ),
        (vec!["target:a", "target:b"], ""),
        (Vec::new(), ""),
    ] {
        let profile = RetargetProfile::new(
            source_profile.clone(),
            RigProfile::new(RigProfileName::from("target"))
                .with_chain(HumanoidChain::Spine, ChainBinding::new(names)),
        );
        let report = profile
            .resolve(&source, &target)
            .unwrap()
            .report(&source, &target);
        assert_eq!(
            report.to_string(),
            format!(
                "profile: source -> target\nsource rig: source (2 joints)\ntarget rig: target (3 joints)\n\n{suffix}"
            )
        );
    }
    let profile = RetargetProfile::new(
        source_profile,
        RigProfile::new(RigProfileName::from("target")),
    );
    assert_eq!(
        profile
            .resolve(&source, &target)
            .unwrap()
            .report(&source, &target)
            .to_string(),
        "profile: source -> target\nsource rig: source (2 joints)\ntarget rig: target (3 joints)\n\n"
    );
}

#[test]
fn complete_report_preserves_end_effector_insertion_order() {
    let source = skeleton(&["hips", "left", "right"]);
    let target = skeleton(&["pelvis", "left target", "right target"]);
    let profile = RetargetProfile::new(
        RigProfile::new(RigProfileName::from("source"))
            .with_required(HumanoidJoint::Pelvis, "hips")
            .with(HumanoidJoint::HandLeft, "left")
            .with(HumanoidJoint::HandRight, "right"),
        RigProfile::new(RigProfileName::from("target"))
            .with(HumanoidJoint::HandRight, "right target")
            .with(HumanoidJoint::HandLeft, "left target")
            .with_required(HumanoidJoint::Pelvis, "pelvis"),
    );
    let retargeter = Retargeter::new(&source, &target, &profile).unwrap();
    let mapping = retargeter.resolved().report(&source, &target);
    assert_eq!(
        retargeter.report().to_string(),
        format!(
            "{mapping}\nscale: 1.0000 target units per source unit\nend effectors: HandRight=right target, HandLeft=left target\n"
        )
    );
}

#[test]
fn footer_formats_scale_without_changing_typed_mapping_artifact() {
    let rig = skeleton(&["hips"]);
    let profile = RetargetProfile {
        settings: RetargetSettings {
            scale: ScalePolicy::Fixed(1.23456),
            ..RetargetSettings::default()
        },
        ..RetargetProfile::new(
            RigProfile::new(RigProfileName::from("source"))
                .with_required(HumanoidJoint::Pelvis, "hips"),
            RigProfile::new(RigProfileName::from("target"))
                .with_required(HumanoidJoint::Pelvis, "hips"),
        )
    };
    let retargeter = Retargeter::new(&rig, &rig, &profile).unwrap();
    let mapping = retargeter.resolved().report(&rig, &rig);
    let retained = mapping.clone();
    let complete = retargeter.report();
    assert_eq!(mapping, retained);
    assert_eq!(
        complete.to_string(),
        format!("{mapping}\nscale: 1.2346 target units per source unit\n")
    );
    assert_eq!(
        format!("{complete:?}"),
        format!("{:?}", complete.to_string())
    );
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
