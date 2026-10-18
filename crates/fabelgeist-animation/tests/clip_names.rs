//! Public clip identity, serialization, and retargeting regressions.

use fabelgeist_animation::animation::retarget::{
    HumanoidJoint, RetargetProfile, RetargetSettings, Retargeter, RigProfile, RootMotionPolicy,
    ScalePolicy, TranslationPolicy,
};
use fabelgeist_animation::animation::{Animation, AnimationClipName, Curve, JointTrack};
use fabelgeist_animation::{Joint, Skeleton};
use fabelgeist_math::{matrix::Mat4, transform::Transform, vector::Vec3};

#[test]
fn authored_spellings_round_trip_through_scalar_and_clip_json_and_formatting() {
    for spelling in [
        "",
        "walk",
        "Walk",
        "namespace:walk",
        "β歩行😀",
        "  walk \t\n",
        "a\0b",
        "\u{0001}\r\u{001f}",
        "e\u{0301}",
        "é",
        "a\\\"b",
    ] {
        let name = AnimationClipName::from(spelling.to_owned());
        let scalar = serde_json::to_string(&name).unwrap();
        assert_eq!(scalar, serde_json::to_string(spelling).unwrap());
        assert_eq!(
            serde_json::from_str::<AnimationClipName>(&scalar).unwrap(),
            name
        );
        assert_eq!(format!("{name}"), spelling);
        assert_eq!(format!("{name:?}"), format!("{spelling:?}"));
        let clip = Animation::new(name.clone());
        let serialized = serde_json::to_string(&clip).unwrap();
        assert_eq!(
            serialized,
            format!("{{\"name\":{scalar},\"duration\":0.0,\"tracks\":[]}}")
        );
        let restored: Animation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored, clip);
        assert_eq!(restored.name, name);
    }
    assert_eq!(Animation::default().name, AnimationClipName::from(""));
    assert_ne!(
        AnimationClipName::from("e\u{0301}"),
        AnimationClipName::from("é")
    );
}

#[test]
fn invalid_native_json_preserves_string_deserialization_errors() {
    for json in ["null", "42", "[]", "{}", "true"] {
        let error = serde_json::from_str::<AnimationClipName>(json).unwrap_err();
        let native_error = serde_json::from_str::<String>(json).unwrap_err();
        assert_eq!(error.to_string(), native_error.to_string());
    }
    let error = serde_json::from_str::<Animation>("{\"duration\":0,\"tracks\":[]}").unwrap_err();
    assert_eq!(
        error.to_string(),
        "missing field `name` at line 1 column 26"
    );
}

#[test]
fn retargeting_keeps_clip_identity_and_sampled_motion() {
    let skeleton = Skeleton::new(vec![Joint::new(
        "hips".to_owned(),
        0,
        None,
        Mat4::identity(),
        Transform::identity(),
        Some(0),
    )]);
    let rig = RigProfile::new("one-pelvis").with_required(HumanoidJoint::Pelvis, "hips");
    let profile = RetargetProfile::new(rig.clone(), rig).with_settings(
        RetargetSettings::default()
            .with_scale(ScalePolicy::None)
            .with_root_motion(RootMotionPolicy::Keep)
            .with_translation(TranslationPolicy::Copy),
    );
    let mut clip = Animation::new(AnimationClipName::from("namespace:β\0 walk\t"));
    clip.tracks.push(JointTrack {
        joint: "hips".to_owned(),
        translation: Some(Curve::new(
            vec![0.0, 0.5, 1.5],
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.25, 0.5, -0.5),
                Vec3::new(1.0, 2.0, -3.0),
            ],
        )),
        ..Default::default()
    });
    clip.recompute_duration();
    let retargeted = Retargeter::new(&skeleton, &skeleton, &profile)
        .unwrap()
        .clip(&clip);
    assert_eq!(retargeted.name, clip.name);
    assert_eq!(retargeted.duration, clip.duration);
    assert_eq!(retargeted.key_times(), clip.key_times());
    let source_binding = clip.bind(&skeleton);
    let target_binding = retargeted.bind(&skeleton);
    for time in [0.0, 0.25, 0.5, 1.0, 1.5] {
        assert_eq!(
            clip.sample(&source_binding, time),
            retargeted.sample(&target_binding, time)
        );
    }
    let source_name = clip.name.clone();
    clip.name = AnimationClipName::from("renamed");
    assert_eq!(retargeted.name, source_name);
}
