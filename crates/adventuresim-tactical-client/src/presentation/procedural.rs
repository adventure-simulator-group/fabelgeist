use super::*;
use fabelgeist_determinism::StreamId;

pub(super) fn obstacle_seed(position: Vec3) -> u64 {
    StreamId::new("visual.obstacle.position")
        .seed(
            0,
            &[
                u64::from(position.x.to_bits()),
                u64::from(position.z.to_bits()),
            ],
        )
        .to_u64()
}

pub(super) fn stable_text_seed(value: &str) -> u64 {
    fabelgeist_determinism::Seed::derive(
        value.as_bytes(),
        StreamId::new("visual.text-identity"),
        &[],
    )
    .to_u64()
}

pub(super) fn bps(value: u16) -> f32 {
    adventuresim_world_schema::UnitBasisPoints::saturating(value).as_unit_f32()
}

pub(super) fn color_vec4(color: Color) -> Vec4 {
    Vec4::from_array(color.to_linear().to_f32_array())
}
