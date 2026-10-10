//! Public construction, Boolean JSON ports and rig-resolution behavior.

use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, JointBinding, JointRequirement, RetargetProfile, RetargetSettings,
    RetargetStrictness, RigProfile,
};
use fabelgeist_animation::skeleton::mixamo::MixamoRig;
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform};

const REJECTED_BINDINGS: [RejectedBinding; 9] = [
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":null}",
        diagnostic: "invalid type: null, expected a boolean at line 1 column 33",
    },
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":\"true\"}",
        diagnostic: "invalid type: string \"true\", expected a boolean at line 1 column 35",
    },
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":1}",
        diagnostic: "invalid type: integer `1`, expected a boolean at line 1 column 30",
    },
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":[]}",
        diagnostic: "invalid type: sequence, expected a boolean at line 1 column 29",
    },
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":{}}",
        diagnostic: "invalid type: map, expected a boolean at line 1 column 29",
    },
    RejectedBinding {
        input: "{\"required\":\"bad\",\"names\":3}",
        diagnostic: "invalid type: string \"bad\", expected a boolean at line 1 column 17",
    },
    RejectedBinding {
        input: "{\"names\":3,\"required\":\"bad\"}",
        diagnostic: "invalid type: integer `3`, expected a sequence at line 1 column 10",
    },
    RejectedBinding {
        input: "{\"required\":true}",
        diagnostic: "missing field `names` at line 1 column 17",
    },
    RejectedBinding {
        input: "{\"names\":[\"hips\"],\"required\":false,\"required\":true}",
        diagnostic: "duplicate field `required` at line 1 column 45",
    },
];

struct RejectedBinding {
    input: &'static str,
    diagnostic: &'static str,
}

struct LookupCase {
    aliases: &'static [&'static str],
    skeleton_names: &'static [&'static str],
    expected_index: usize,
}

#[test]
fn optional_and_required_bindings_control_missing_joint_admission() {
    let empty = skeleton(&[]);
    let mut binding = JointBinding::new("head");
    let optional = RigProfile::new("optional").with_joint(HumanoidJoint::Head, binding.clone());
    let resolved = optional.resolve(&empty).unwrap();
    assert_eq!(resolved.missing, [HumanoidJoint::Head]);
    assert!(resolved.joints.is_empty());

    binding.required = JointRequirement::Required;
    let required = RigProfile::new("required").with_joint(HumanoidJoint::Head, binding.clone());
    let constructed = RigProfile::new("required").with_required(HumanoidJoint::Head, "head");
    assert_eq!(required, constructed);
    assert_eq!(required.clone(), required);
    assert_eq!(
        required.resolve(&empty).unwrap_err().to_string(),
        constructed.resolve(&empty).unwrap_err().to_string(),
    );

    binding.required = JointRequirement::Optional;
    assert_eq!(
        RigProfile::new("optional").with_joint(HumanoidJoint::Head, binding),
        optional,
    );
}

#[test]
fn boolean_json_defaults_and_round_trips_keep_binding_roles() {
    for input in [
        r#"{"names":["hips"]}"#,
        r#"{"names":["hips"],"required":false}"#,
    ] {
        let binding: JointBinding = serde_json::from_str(input).unwrap();
        assert_eq!(binding.required, JointRequirement::Optional);
        let serialized = serde_json::to_string(&binding).unwrap();
        assert_eq!(serialized, r#"{"names":["hips"],"required":false}"#);
        let decoded: JointBinding = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded, binding);
        let resolved = RigProfile::new("optional-json")
            .with_joint(HumanoidJoint::Pelvis, decoded)
            .resolve(&skeleton(&[]))
            .unwrap();
        assert_eq!(resolved.missing, [HumanoidJoint::Pelvis]);
    }

    let binding: JointBinding =
        serde_json::from_str(r#"{"names":["hips"],"required":true}"#).unwrap();
    assert_eq!(binding, JointBinding::new("hips").required());
    let serialized = serde_json::to_string(&binding).unwrap();
    assert_eq!(serialized, r#"{"names":["hips"],"required":true}"#);
    let decoded: JointBinding = serde_json::from_str(&serialized).unwrap();
    assert_eq!(decoded, binding);
    assert!(
        RigProfile::new("required-json")
            .with_joint(HumanoidJoint::Pelvis, decoded)
            .resolve(&skeleton(&[]))
            .is_err()
    );
}

#[test]
fn invalid_boolean_json_retains_diagnostics_and_field_precedence() {
    for rejected in REJECTED_BINDINGS {
        let error = serde_json::from_str::<JointBinding>(rejected.input).unwrap_err();
        assert_eq!(error.to_string(), rejected.diagnostic);
    }
    for input in ["null", "1", "\"Required\"", "[]", "{}"] {
        let error = serde_json::from_str::<JointRequirement>(input).unwrap_err();
        let native = serde_json::from_str::<bool>(input).unwrap_err();
        assert_eq!(error.to_string(), native.to_string());
    }
}

#[test]
fn required_joint_diagnostics_follow_profile_insertion_order() {
    let profile = RigProfile::new("ordered")
        .with_joint(
            HumanoidJoint::Head,
            JointBinding::new("head").required().with_alias("Head2"),
        )
        .with_required(HumanoidJoint::Pelvis, "hips")
        .with(HumanoidJoint::Chest, "chest");
    assert_eq!(
        profile.resolve(&skeleton(&[])).unwrap_err().to_string(),
        "rig profile \"ordered\" requires joints the skeleton does not have: Head (expected head | Head2), Pelvis (expected hips)",
    );
}

#[test]
fn matching_keeps_exact_name_alias_and_duplicate_priority() {
    for case in [
        LookupCase {
            aliases: &["hips"],
            skeleton_names: &["HIP-S", "hips"],
            expected_index: 1,
        },
        LookupCase {
            aliases: &["absent", "hips"],
            skeleton_names: &["HIP-S", "hips"],
            expected_index: 1,
        },
        LookupCase {
            aliases: &["HIP-S", "pelvis"],
            skeleton_names: &["hips", "pelvis"],
            expected_index: 0,
        },
        LookupCase {
            aliases: &["hips"],
            skeleton_names: &["hips", "hips"],
            expected_index: 0,
        },
    ] {
        let mut binding = JointBinding::new("").required();
        binding.names = case.aliases.iter().map(|name| (*name).to_owned()).collect();
        let resolved = RigProfile::new("lookup")
            .with_joint(HumanoidJoint::Pelvis, binding)
            .resolve(&skeleton(case.skeleton_names))
            .unwrap();
        assert_eq!(
            resolved.joint(HumanoidJoint::Pelvis),
            Some(case.expected_index)
        );
    }
}

#[test]
fn built_in_and_inferred_profiles_require_their_pelvis() {
    let mixamo = MixamoRig::profile();
    let required_names: Vec<_> = mixamo
        .joints
        .values()
        .filter(|binding| binding.required == JointRequirement::Required)
        .map(|binding| binding.names[0].as_str())
        .collect();
    let resolved = mixamo.resolve(&skeleton(&required_names)).unwrap();
    assert!(resolved.missing.contains(&HumanoidJoint::IndexDistalLeft));
    assert!(resolved.missing.contains(&HumanoidJoint::Head));
    let no_pelvis: Vec<_> = required_names
        .into_iter()
        .filter(|name| *name != "mixamorig:Hips")
        .collect();
    assert_eq!(
        mixamo
            .resolve(&skeleton(&no_pelvis))
            .unwrap_err()
            .to_string(),
        "rig profile \"Mixamo\" requires joints the skeleton does not have: Pelvis (expected mixamorig:Hips)",
    );

    let inferred_skeleton = skeleton(&["hips", "spine", "head"]);
    let inferred = RigProfile::infer(&inferred_skeleton);
    assert_eq!(
        inferred.binding(HumanoidJoint::Pelvis).unwrap().required,
        JointRequirement::Required,
    );
    assert_eq!(
        inferred
            .resolve(&inferred_skeleton)
            .unwrap()
            .joint(HumanoidJoint::Pelvis),
        Some(0)
    );
}

#[test]
fn named_joint_requirements_resolve_before_distinct_strictness_policy() {
    let source = RigProfile::new("source-good").with_required(HumanoidJoint::Pelvis, "hips");
    let target = RigProfile::new("target-missing").with_required(HumanoidJoint::Head, "head");
    let settings = RetargetSettings {
        strict: RetargetStrictness::Strict,
        ..Default::default()
    };
    let profile = RetargetProfile::new(source.clone(), target).with_settings(settings.clone());
    assert_eq!(
        profile
            .resolve(&skeleton(&["hips"]), &skeleton(&[]))
            .unwrap_err()
            .to_string(),
        "rig profile \"target-missing\" requires joints the skeleton does not have: Head (expected head)",
    );
    let strict = RetargetProfile::new(source.clone(), RigProfile::new("target-empty"))
        .with_settings(settings);
    assert_eq!(
        strict
            .resolve(&skeleton(&["hips"]), &skeleton(&[]))
            .unwrap_err()
            .to_string(),
        "strict retargeting: the target rig has no joint for Pelvis",
    );
    let permissive = RetargetProfile::new(source, RigProfile::new("target-empty"));
    assert!(
        permissive
            .resolve(&skeleton(&["hips"]), &skeleton(&[]))
            .unwrap()
            .shared_roles()
            .is_empty()
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
