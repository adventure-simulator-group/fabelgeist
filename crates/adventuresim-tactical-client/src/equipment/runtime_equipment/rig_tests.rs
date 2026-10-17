use super::*;
use adventuresim_core::character_proportions::{BODY_PROPORTION_COUNT, BodyProportion};

fn body() -> RuntimeBody {
    RuntimeBody {
        domain: "test".into(),
        positions: vec![[0.2, 1.1, 0.0], [0.4, 1.1, 0.0], [0.2, 1.3, 0.0]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        faces: vec![[0, 1, 2]],
        texcoords: vec![[0.0, 0.0]; 3],
        texcoord_faces: vec![[0, 1, 2]],
        joint_indices: vec![[0; 8]; 3],
        joint_weights: vec![[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; 3],
        joint_names: vec!["arm".into()],
        global_joint_states: vec![[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]],
        device: Default::default(),
    }
}

#[test]
fn leg_height_correction_moves_pelvis_descendants_but_not_body_world() {
    let mut proportions = CharacterProportions::default();
    proportions.set(BodyProportion::HipWidth, 0.5).unwrap();
    let mut basis = JointProportionBasis {
        reference: Default::default(),
        translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
    };
    basis.translation_metres[BodyProportion::HipWidth.index()] = [0.0, -0.2, 0.0];
    let rig = FittingRig {
        reference: Default::default(),
        joints: vec![0, 1, 2, 3],
        nodes: vec![
            RigNode {
                parent: None,
                local: Transform::IDENTITY,
                basis: None,
            },
            RigNode {
                parent: Some(0),
                local: Transform::IDENTITY,
                basis: None,
            },
            RigNode {
                parent: Some(1),
                local: Transform::IDENTITY,
                basis: Some(basis.clone()),
            },
            RigNode {
                parent: Some(1),
                local: Transform::IDENTITY,
                basis: Some(basis),
            },
        ],
    };
    let mut body = body();
    body.joint_names = ["body_world", "root", "l_foot", "r_foot"]
        .map(fabelgeist_rig::RigJointName::from)
        .to_vec();
    body.global_joint_states = vec![[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]; 4];
    rig.fit(&mut body, proportions).unwrap();
    assert_eq!(body.global_joint_states[0][1], 0.0);
    assert!((body.global_joint_states[1][1] - 0.1).abs() < 1e-6);
    assert!(body.global_joint_states[2][1].abs() < 1e-6);
    assert!(body.global_joint_states[3][1].abs() < 1e-6);
}

#[test]
fn coupled_proportions_match_morph_then_skin_and_do_not_deform_twice() {
    let mut reference = CharacterProportions::default();
    reference.set(BodyProportion::HipWidth, 0.2).unwrap();
    let mut desired = reference;
    desired.set(BodyProportion::HipWidth, -0.1).unwrap();
    desired.set(BodyProportion::UpperArmLength, 0.4).unwrap();
    let mut basis = JointProportionBasis {
        reference,
        translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
    };
    basis.translation_metres[BodyProportion::HipWidth.index()] = [0.1, 0.0, 0.0];
    basis.translation_metres[BodyProportion::UpperArmLength.index()] = [0.0, 0.2, 0.0];
    let parent = Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
    let rig = FittingRig {
        reference,
        joints: vec![1],
        nodes: vec![
            RigNode {
                parent: None,
                local: parent,
                basis: None,
            },
            RigNode {
                parent: Some(0),
                local: Transform::from_xyz(1.0, 0.0, 0.0),
                basis: Some(basis),
            },
        ],
    };
    let mut body = body();
    let names = vec!["mhr_identity_00".into()];
    let key = BodyShapeKey::new(&names, Some(42), desired, reference);
    let target = MorphAttributes::new(Vec3::X * 0.1, Vec3::ZERO, Vec3::ZERO);
    let targets = vec![target; 3];
    let weight = adventuresim_core::character_morph::CharacterMorphWeights::from_character_id(42)
        .named_weight(&names[0])
        .unwrap();
    let expected_offset = parent.rotation * Vec3::new(-0.03, 0.08, 0.0);
    let original = body.positions.clone();
    key.apply(&mut body, &targets).unwrap();
    let inverse = rig.fit(&mut body, desired).unwrap();
    let joint = Mat4::from_translation(Vec3::Y + expected_offset);
    for (&source, &fitted) in original.iter().zip(&body.positions) {
        let shader_position = Vec3::from_array(source) + Vec3::X * (0.1 * weight) + expected_offset;
        assert!(Vec3::from_array(fitted).abs_diff_eq(shader_position, 1e-6));
        assert!(
            (joint * inverse[0])
                .transform_point3(Vec3::from_array(fitted))
                .abs_diff_eq(shader_position, 1e-6)
        );
    }
    let pose = joint * Mat4::from_quat(Quat::from_rotation_y(0.7)) * inverse[0];
    let a = Vec3::from_array(body.positions[0]);
    let b = Vec3::from_array(body.positions[1]);
    assert!(
        (pose.transform_point3(a).distance(pose.transform_point3(b)) - a.distance(b)).abs() < 1e-6
    );
}

#[test]
fn cache_identity_changes_with_body_controls_and_rejects_missing_targets() {
    let names = vec!["mhr_identity_00".into()];
    let reference = CharacterProportions::default();
    let first = BodyShapeKey::new(&names, Some(1), reference, reference);
    let second = BodyShapeKey::new(&names, Some(2), reference, reference);
    assert_ne!(first, second);
    let mut desired = reference;
    desired.set(BodyProportion::HipWidth, 0.1).unwrap();
    assert_ne!(
        first,
        BodyShapeKey::new(&names, Some(1), desired, reference)
    );
    assert!(first.apply(&mut body(), &[]).is_err());
}
