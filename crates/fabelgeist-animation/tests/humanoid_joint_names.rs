//! Public role spelling, classification and native formatting contracts.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Debug, Display};

use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, HumanoidRoleName, JointBinding, RetargetProfile, RigProfile, RigProfileName,
    UnknownHumanoidJoint,
};
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform};

const REJECTED: &[&str] = &[
    "",
    " Pelvis",
    "Pelvis ",
    "pelvis",
    "ns:Pelvis",
    "ns|Pelvis",
    "Pelvis0",
    "Pel-vis",
    "未知🦴",
    "\"Pelvis\"",
    "Pelvis\0",
    "Pelvis\n\t",
];

#[test]
fn role_spellings_classify_after_native_decoding_and_preserve_enum_wire() {
    for &role in HumanoidJoint::ALL {
        let serialized = serde_json::to_string(&role).unwrap();
        let decoded_text: String = serde_json::from_str(&serialized).unwrap();
        let decoded_role: HumanoidJoint = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decoded_role, role);
        assert_eq!(
            HumanoidJoint::try_from(HumanoidRoleName::from(decoded_text)),
            Ok(role)
        );
        assert_eq!(HumanoidJoint::try_from(role.name()), Ok(role));
        let map = BTreeMap::from([(role, String::from("bound"))]);
        let wire = serde_json::to_string(&map).unwrap();
        assert_eq!(wire, format!("{{{serialized}:\"bound\"}}"));
        assert_eq!(
            serde_json::from_str::<BTreeMap<HumanoidJoint, String>>(&wire).unwrap(),
            map
        );
    }
}

#[test]
fn potential_names_preserve_arbitrary_text_and_native_formatting() {
    for &text in REJECTED {
        let borrowed = HumanoidRoleName::from(text);
        let owned = HumanoidRoleName::from(text.to_owned());
        assert_eq!(borrowed, owned);
        assert_eq!(formatting(&borrowed), formatting(&text.to_owned()));
    }
    for &role in HumanoidJoint::ALL {
        let decoded = HumanoidRoleName::from(role.to_string());
        assert_eq!(role.name(), decoded);
        assert_eq!(formatting(&role.name()), formatting(&role.to_string()));
        assert_eq!(format!("{role:>64.8}"), role.to_string());
    }
}

#[test]
fn rejected_spelling_survives_typed_and_native_parsers_with_string_error_format() {
    for &text in REJECTED {
        let name = HumanoidRoleName::from(text);
        let typed = HumanoidJoint::try_from(name.clone()).unwrap_err();
        let native: UnknownHumanoidJoint = text.parse::<HumanoidJoint>().unwrap_err();
        let expected = format!("unknown humanoid joint {text:?}");
        assert_eq!(typed.name(), &name);
        assert_eq!(typed, native);
        assert_eq!(typed.clone(), typed);
        assert_eq!(formatting(&typed), formatting(&expected));
        assert!(typed.source().is_none());
    }
}

#[test]
fn parsed_role_reaches_public_rig_binding_and_complete_report() {
    let skeleton = Skeleton::new(vec![Joint::new(
        "hips".into(),
        0,
        None,
        Mat4::identity(),
        Transform::identity(),
        None,
    )]);
    let role = HumanoidJoint::try_from(HumanoidRoleName::from("Pelvis")).unwrap();
    let profile =
        RigProfile::new(RigProfileName::from("source")).with_joint(role, JointBinding::new("hips"));
    let resolved = profile.resolve(&skeleton).unwrap();
    assert_eq!(
        resolved.joints.keys().copied().collect::<Vec<_>>(),
        vec![HumanoidJoint::Pelvis]
    );
    assert_eq!(resolved.profile, RigProfileName::from("source"));
    let recipe = RetargetProfile::new(profile.clone(), profile);
    let mapping = recipe.resolve(&skeleton, &skeleton).unwrap();
    assert_eq!(
        mapping.report(&skeleton, &skeleton).to_string(),
        "profile: source -> source\nsource rig: source (1 joints)\ntarget rig: source (1 joints)\n\nPelvis:\n  source = hips\n  target = hips\n"
    );
}

#[test]
fn structured_parse_errors_propagate_through_standard_error_boundary() {
    let error = parse_role(HumanoidRoleName::from("pelvis")).unwrap_err();
    assert_eq!(error.to_string(), "unknown humanoid joint \"pelvis\"");
    assert!(error.source().is_none());
    assert_eq!(
        parse_role(HumanoidRoleName::from("Pelvis")).unwrap(),
        HumanoidJoint::Pelvis
    );
}

#[test]
fn enum_serde_rejects_invalid_payloads_and_keeps_exact_spelling() {
    for input in ["null", "true", "1", "[]", "{}", "\"Unknown\""] {
        assert!(serde_json::from_str::<HumanoidJoint>(input).is_err());
    }
    for &text in REJECTED {
        let wire = serde_json::to_string(text).unwrap();
        assert!(serde_json::from_str::<HumanoidJoint>(&wire).is_err());
    }
    assert_eq!(
        serde_json::from_str::<HumanoidJoint>("\"UpperArmLeft\"").unwrap(),
        HumanoidJoint::UpperArmLeft
    );
}

fn formatting(value: &(impl Debug + Display)) -> Vec<String> {
    vec![
        format!("{value}"),
        format!("{value:>64}"),
        format!("{value:<64}"),
        format!("{value:^64}"),
        format!("{value:*^64}"),
        format!("{value:.8}"),
        format!("{value:>64.8}"),
        format!("{value:?}"),
        format!("{value:#?}"),
        format!("{value:>64?}"),
        format!("{value:>64.8?}"),
        format!("{value:#x?}"),
    ]
}

fn parse_role(name: HumanoidRoleName) -> Result<HumanoidJoint, Box<dyn Error>> {
    Ok(HumanoidJoint::try_from(name)?)
}
