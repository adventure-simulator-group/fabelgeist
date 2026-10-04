use fabelgeist_animation::animation::{
    Animation,
    retarget::{HumanoidJoint, RetargetProfile, RigProfile},
};
use fabelgeist_animation::skeleton::SkinJointOrdinal;
use fabelgeist_animation::skeleton::{Joint, Skeleton};
use fabelgeist_math::matrix::Mat4;
use fabelgeist_rig::RigJointOrdinal;

#[test]
fn frozen_animation_boundary_fixture() {
    let skeleton = Skeleton::new(vec![
        Joint::new(
            "ns:Left_Arm".into(),
            RigJointOrdinal::from(0_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(0_usize)),
        ),
        Joint::new(
            "LeftArm".into(),
            RigJointOrdinal::from(1_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(1_usize)),
        ),
        Joint::new(
            "ns:Left_Arm".into(),
            RigJointOrdinal::from(2_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(2_usize)),
        ),
    ]);
    let profile: RigProfile = serde_json::from_value(serde_json::json!({
        "name": " profile:α \0",
        "joints": {
            "UpperArmLeft": {"names": ["LeftArm"], "required": true},
            "LowerArmLeft": {"names": ["missing|LEFT.ARM"]},
            "HandLeft": {"names": ["ns:Left_Arm"]}
        }, "root": "Pelvis", "reference": "Bind"
    }))
    .unwrap();
    let recipe = RetargetProfile::new(profile.clone(), profile.clone());
    let clip: Animation = serde_json::from_value(serde_json::json!({
        "name": "clip:α \0", "duration": 1.0, "tracks": [{"joint": "joint:β \0"}]
    }))
    .unwrap();
    let resolved = recipe.resolve(&skeleton, &skeleton).unwrap();
    let missing: RigProfile = serde_json::from_value(serde_json::json!({
        "name": "failure", "joints": { "Head": {"names": ["cranium", "skull"], "required": true} },
        "root": "Pelvis", "reference": "Bind"
    }))
    .unwrap();
    let output = serde_json::json!({
        "skeleton": skeleton,
        "recipe": recipe,
        "clip": clip,
        "slots": [resolved.source.joint(HumanoidJoint::UpperArmLeft),
                  resolved.source.joint(HumanoidJoint::LowerArmLeft),
                  resolved.source.joint(HumanoidJoint::HandLeft)],
        "report": resolved.report(&skeleton, &skeleton).to_string(),
        "failure": missing.resolve(&skeleton).unwrap_err().to_string()
    });
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("animation-boundary-fixture.json")).unwrap();
    assert_eq!(output, expected);
}
