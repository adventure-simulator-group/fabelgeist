use fabelgeist_animation::skeleton::{
    Joint, Skeleton, SkeletonModelMatrices, SkinBlendWeight, SkinJointCount, SkinJointOrdinal,
    SkinningError, build_skinning_matrices,
};
use fabelgeist_math::matrix::Mat4;
use fabelgeist_rig::{RigJointCount, RigJointOrdinal};

#[test]
fn sparse_skin_slots_are_independent_of_skeleton_order() {
    let skeleton = Skeleton::new(vec![
        Joint::new(
            "first".into(),
            RigJointOrdinal::from(0_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(5_usize)),
        ),
        Joint::new(
            "second".into(),
            RigJointOrdinal::from(1_usize),
            None,
            Mat4::identity(),
            Default::default(),
            Some(SkinJointOrdinal::from(2_usize)),
        ),
    ]);
    let mut posed = skeleton.world_transforms();
    posed[RigJointOrdinal::from(0_usize)].columns[3][0] = 7.0;
    posed[RigJointOrdinal::from(1_usize)].columns[3][0] = 9.0;
    let skin = build_skinning_matrices(&skeleton, &posed).unwrap();
    assert_eq!(skin.count(), SkinJointCount::from(6_usize));
    assert_eq!(skin[SkinJointOrdinal::from(5_usize)].columns[3][0], 7.0);
    assert_eq!(skin[SkinJointOrdinal::from(2_usize)].columns[3][0], 9.0);
    for hole in [0_usize, 1, 3, 4].map(SkinJointOrdinal::from) {
        assert_eq!(skin[hole], Mat4::identity());
    }
    let mismatch =
        build_skinning_matrices(&skeleton, &SkeletonModelMatrices::from(vec![])).unwrap_err();
    assert_eq!(
        mismatch,
        SkinningError::JointCountMismatch {
            expected: RigJointCount::from(2_usize),
            actual: RigJointCount::default(),
        }
    );
    assert_eq!(
        mismatch.to_string(),
        "Pose joint count mismatch with skeleton"
    );
}

#[test]
fn weight_admission_preserves_ieee_bits_and_sum_semantics() {
    for raw in [
        0.0_f32,
        -0.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::from_bits(0x7fc0_0123),
    ] {
        assert_eq!(
            f32::from(SkinBlendWeight::from(raw)).to_bits(),
            raw.to_bits()
        );
    }
    for raw in [
        vec![],
        vec![-0.0_f32],
        vec![1.0, 1.0e-8, -1.0],
        vec![f32::INFINITY],
    ] {
        let expected: f32 = raw.iter().copied().sum();
        let actual: SkinBlendWeight = raw.into_iter().map(SkinBlendWeight::from).sum();
        assert_eq!(f32::from(actual).to_bits(), expected.to_bits());
    }
}
