use fabelgeist_animation::animation::retarget::{ChainPosition, RigJointChain};
use fabelgeist_animation::{JointTransform, ModelPose};
use fabelgeist_math::vector::{Vec3, Vec4};
use fabelgeist_rig::RigJointOrdinal;

#[test]
fn frozen_chain_interpolation_fixture() {
    let mut model = ModelPose::from(vec![JointTransform::identity(); 7]);
    model[RigJointOrdinal::from(2_usize)].rotation =
        Vec4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.8);
    model[RigJointOrdinal::from(4_usize)].rotation =
        Vec4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), -0.4);
    let mut rows = Vec::new();
    for chain in [vec![], vec![4_usize], vec![6, 2, 4]] {
        let chain = RigJointChain::from(
            chain
                .into_iter()
                .map(RigJointOrdinal::from)
                .collect::<Vec<_>>(),
        );
        for position in [
            -1.0_f32,
            -0.0,
            0.249,
            0.25,
            0.5,
            0.75,
            1.0,
            2.0,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ]
        .map(ChainPosition::from)
        {
            let rotation = chain.rotation_in(&model, position);
            rows.push(serde_json::json!({
                "nearest": usize::from(chain.nearest_joint(position)),
                "rotation_bits": ([rotation.x, rotation.y, rotation.z, rotation.w].map(f32::to_bits))
            }));
        }
    }
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("animation-chain-fixture.json")).unwrap();
    assert_eq!(serde_json::to_value(rows).unwrap(), expected);
}
