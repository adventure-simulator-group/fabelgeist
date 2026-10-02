//! Chips, terminating cracks, fractured lips, and grain on raised bark plates.
use super::*;

pub(super) fn transverse_closure(
    params: &crate::TextureParameters,
    column: i32,
    row: i32,
    within_row: f32,
    sample_x: f32,
    crown: f32,
) -> f32 {
    let tau = core::f32::consts::TAU;
    let distance = within_row.min(1.0 - within_row) / params.surface.rows as f32;
    let width = 0.012 + 0.008 * bark_random(params, column, row, 0xc713);
    let core = (-0.5 * (distance / width).powi(2)).exp();
    let gate = 0.42
        + 0.58
            * smoothstep(
                -0.58,
                0.20,
                (tau * (sample_x * 7.0 + bark_random(params, column, row, 0x5a71))).sin(),
            );
    -(0.12 + 0.10 * bark_random(params, column, row, 0x731c)) * core * gate * crown
}

pub(super) fn chipped_face(
    params: &crate::TextureParameters,
    column: i32,
    row: i32,
    within_column: f32,
    within_row: f32,
    crown: f32,
) -> f32 {
    let chip_x = 0.16 + 0.68 * bark_random(params, column, row, 0xe417);
    let chip_y = 0.10 + 0.80 * bark_random(params, column, row, 0xb529);
    let chip_distance = Vec2::new(
        (within_column - chip_x) / 0.13,
        (within_row - chip_y) / 0.10,
    )
    .length_squared();
    -(0.050 + 0.080 * bark_random(params, column, row, 0xf81d))
        * (-0.5 * chip_distance).exp()
        * crown
}

pub(super) fn terminating_branch(
    params: &crate::TextureParameters,
    column: i32,
    row: i32,
    within_column: f32,
    within_row: f32,
    crown: f32,
) -> f32 {
    let branch_roll = bark_random(params, column, row, 0x64ab);
    let branch_side = bark_random(params, column, row, 0x917d) >= 0.5;
    let branch_start_y = 0.18 + 0.64 * bark_random(params, column, row, 0x2f43);
    let branch_end_y =
        (branch_start_y + bark_random(params, column, row, 0xd815) * 0.54 - 0.27).clamp(0.08, 0.92);
    let branch_start_x = if branch_side { 0.98 } else { 0.02 };
    let branch_end_x = if branch_side {
        0.42 + 0.20 * bark_random(params, column, row, 0x3e29)
    } else {
        0.38 - 0.20 * bark_random(params, column, row, 0x3e29)
    };
    let plate_point = Vec2::new(
        within_column / params.surface.columns as f32,
        within_row / params.surface.rows as f32,
    );
    let branch_start = Vec2::new(
        branch_start_x / params.surface.columns as f32,
        branch_start_y / params.surface.rows as f32,
    );
    let branch_end = Vec2::new(
        branch_end_x / params.surface.columns as f32,
        branch_end_y / params.surface.rows as f32,
    );
    let branch_distance = distance_to_segment(plate_point, branch_start, branch_end);
    let branch_width = 0.005 + 0.003 * bark_random(params, column, row, 0xa53f);
    let branch_enabled = smoothstep(0.54, 0.68, branch_roll);
    -(0.070 + 0.090 * bark_random(params, column, row, 0x781b))
        * (-0.5 * (branch_distance / branch_width).powi(2)).exp()
        * branch_enabled
        * crown
}

pub(super) fn fractured_notch(
    params: &crate::TextureParameters,
    nearest_crack: i32,
    row: i32,
    edge_distance: f32,
    valley_width: f32,
    within_row: f32,
) -> f32 {
    let notch_y = bark_random(params, nearest_crack, row, 0x48c1);
    let notch_distance = Vec2::new(
        (edge_distance - valley_width * 1.08) / (valley_width * 0.34),
        (within_row - notch_y) / 0.11,
    )
    .length_squared();
    -(0.035 + 0.055 * bark_random(params, nearest_crack, row, 0xbb27))
        * (-0.5 * notch_distance).exp()
}

pub(super) fn grain(sample: Vec2, crown: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let secondary_phase = sample.x * 31.0
        + 0.82 * (tau * (sample.y * 3.0 + 0.17)).sin()
        + 0.21 * (tau * (sample.x * 2.0 - sample.y * 4.0 + 0.31)).sin();
    let secondary_distance = (core::f32::consts::PI * secondary_phase).sin().abs();
    let secondary_fissure = (-0.5 * (secondary_distance / 0.14).powi(2)).exp();
    let secondary_gate = smoothstep(
        -0.24,
        0.46,
        (tau * (sample.y * 5.0 + 0.16 * (tau * sample.x * 3.0).sin())).sin(),
    );
    let secondary_relief = -0.032 * secondary_fissure * secondary_gate * crown;
    let plate_grain = (0.018
        * (tau * (sample.x * 19.0 + 0.24 * (tau * sample.y * 4.0).sin())).sin()
        + 0.008 * (tau * (sample.x * 43.0 - sample.y * 11.0 + 0.37)).sin())
        * crown;
    secondary_relief + plate_grain
}
