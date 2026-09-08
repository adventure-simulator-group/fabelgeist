//! Clay faces and lime joints after the running-bond layout has selected a unit.
use super::*;

pub(super) fn finish(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    best: (f32, u64, f32, f32, f32),
) -> BrickSample {
    let (edge_distance, id, local_x, local_y, minimum_half_extent) = best;
    let antialias = params.handmade_brick.sample_brickwork_antialias
        / params.size(HANDMADE_BRICK_TEXTURE_SIZE) as f32
        / minimum_half_extent;
    let brick_coverage = ((antialias - edge_distance) / (antialias * 2.0)).clamp(0.0, 1.0);
    let face_noise = face_noise(params, local_x, local_y, id);
    let cup_strength = (hash_unit(params, id ^ 0xa59d) - 0.5) * params.handmade_brick.cupping;
    let twist_strength = (hash_unit(params, id ^ 0x66c3) - 0.5) * params.handmade_brick.twist;
    let broad_cup = ((local_x * local_x - params.handmade_brick.sample_brickwork_broad_cup_1)
        + (local_y * local_y - params.handmade_brick.sample_brickwork_broad_cup_2)
            * params.handmade_brick.cup_aspect)
        * cup_strength;
    let twist = local_x * local_y * twist_strength;
    let uv = bevy::math::Vec2::new(u, v);
    let pores = params.handmade_brick.pores.sample(params, uv, 0x5491);
    let grit = params.handmade_brick.mortar_grit.sample(params, uv, 0x8117);
    let face_height = params.handmade_brick.face_height - pores.bowl
        + broad_cup
        + twist
        + face_noise * params.handmade_brick.face_noise_relief;
    let mortar_noise = (std::f32::consts::TAU
        * (u * params.handmade_brick.sample_brickwork_mortar_noise_1
            + v * params.handmade_brick.sample_brickwork_mortar_noise_2))
        .sin()
        * params.handmade_brick.mortar_noise_relief;
    let mortar_height = params.handmade_brick.mortar_height + mortar_noise + grit.facet;
    BrickSample {
        height: mortar_height + (face_height - mortar_height) * brick_coverage,
        brick: brick_coverage >= 0.5,
        brick_id: id,
        brick_coverage,
    }
}
