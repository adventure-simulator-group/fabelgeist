use fabelgeist_animation::animation::retarget::{RetargetProfile, Retargeter, RigProfile};
use fabelgeist_animation::skeleton::{ShapeType, auto_rigger::AutoRigger, build_skinning_matrices};
use fabelgeist_animation::{Joint, JointTransform, Skeleton, model_pose, rest_pose};
use fabelgeist_animation::{LocalPose, skeleton::SkinJointOrdinal};
use fabelgeist_math::{matrix::Mat4, transform::Transform, vector::Vec3};
use fabelgeist_rig::RigJointOrdinal;

fn sparse_rig() -> Skeleton {
    let mut root = Transform::identity();
    root.position = Vec3::new(4.0, -2.0, 1.0);
    let mut child = Transform::identity();
    child.position = Vec3::new(0.0, 1.0, 0.0);
    let mut skeleton = Skeleton::new(vec![
        Joint::new(
            "root".into(),
            RigJointOrdinal::from(0_usize),
            None,
            Mat4::identity(),
            root,
            Some(SkinJointOrdinal::from(2_usize)),
        ),
        Joint::new(
            "child".into(),
            RigJointOrdinal::from(1_usize),
            Some(RigJointOrdinal::from(0_usize)),
            Mat4::identity(),
            child,
            Some(SkinJointOrdinal::from(5_usize)),
        ),
        Joint::new(
            "late-parent".into(),
            RigJointOrdinal::from(2_usize),
            Some(RigJointOrdinal::from(2_usize)),
            Mat4::identity(),
            child,
            None,
        ),
    ]);
    skeleton.transform.position = Vec3::new(1.0, 2.0, -3.0);
    for joint in skeleton.joints.iter_mut() {
        joint.shape_type = ShapeType::Sphere;
        joint.radius = 0.25;
        joint.smoothstep_end = 0.75;
    }
    skeleton
}

#[test]
fn frozen_pose_and_skin_payload() {
    let skeleton = sparse_rig();
    let matrices = build_skinning_matrices(&skeleton, &skeleton.world_transforms()).unwrap();
    let matrices: Vec<_> = matrices.iter().cloned().collect();
    let locals = rest_pose(&skeleton);
    let model = model_pose(&skeleton, &locals);
    let positions = [
        Vec3::new(4.0, -2.0, 1.0),
        Vec3::new(4.0, -1.5, 1.0),
        Vec3::new(20.0, 0.0, 0.0),
    ];
    let bindings = AutoRigger::rig_positions(&positions, &skeleton, &skeleton.world_positions());
    let mut joint_words = Vec::new();
    let mut weight_bits = Vec::new();
    for binding in bindings.iter() {
        joint_words.push(binding.joints.map(u32::from));
        weight_bits.push(binding.weights.map(f32::from).map(f32::to_bits));
    }
    let profile: RigProfile = serde_json::from_value(serde_json::json!({
        "name": "root-only", "joints": { "Pelvis": {"names": ["root"]}},
        "root": "Pelvis", "reference": "Bind"
    }))
    .unwrap();
    let recipe = RetargetProfile::new(profile.clone(), profile);
    let default = Retargeter::new(&skeleton, &skeleton, &recipe).unwrap();
    let wrong_rest = LocalPose::from(vec![JointTransform::identity()]);
    let fallback = Retargeter::with_rest_poses(
        &skeleton,
        Some(&wrong_rest),
        &skeleton,
        Some(&wrong_rest),
        &recipe,
    )
    .unwrap();
    let expected_pose = default.pose(&locals);
    assert_eq!(expected_pose, fallback.pose(&locals));
    let empty = Skeleton::new(vec![]);
    let empty_skin = build_skinning_matrices(&empty, &empty.world_transforms()).unwrap();
    let empty_skin: Vec<_> = empty_skin.iter().cloned().collect();
    let output = serde_json::json!({
        "matrices": matrices, "locals": locals, "model": model,
        "joint_words": joint_words, "weight_bits": weight_bits,
        "fallback_pose": expected_pose, "empty_matrices": empty_skin
    });
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("animation-slots-fixture.json")).unwrap();
    assert_eq!(output, expected);
}
