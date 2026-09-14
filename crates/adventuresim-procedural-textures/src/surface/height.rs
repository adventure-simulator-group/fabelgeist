use super::*;

/// Periodic oak relief built from meandering longitudinal crack lines. The
/// intervening strips receive staggered, interrupted transverse closures so
/// they read as raised bark masses rather than a complete Voronoi mosaic.
pub(crate) fn oak_bark_height(params: &crate::TextureParameters, u: f32, v: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let point = Vec2::new(u, v);
    let sample = warped_point(point);
    let approximate_crack = (sample.x * params.surface.columns as f32).round() as i32;
    let mut nearest_crack = approximate_crack;
    let mut signed_edge_distance = f32::INFINITY;
    for crack in (approximate_crack - 2)..=(approximate_crack + 2) {
        let signed_distance = sample.x - oak_bark_crack_x(params, crack, sample.y);
        if signed_distance.abs() < signed_edge_distance.abs() {
            signed_edge_distance = signed_distance;
            nearest_crack = crack;
        }
    }
    let edge_distance = signed_edge_distance.abs();
    let nearest_cell = (nearest_crack - 1, 0);
    let second_cell = (nearest_crack, 0);
    let primary_run =
        0.72 + 0.28 * bark_segment_modulation(params, point, nearest_cell, second_cell);
    let core_width = params.surface.fissure_width_min
        + params.surface.fissure_width_span
            * bark_edge_random(params, nearest_cell, second_cell, 0x1337);
    let valley_width = params.surface.valley_width_min
        + params.surface.valley_width_span
            * bark_edge_random(params, nearest_cell, second_cell, 0x4f29);
    let column = (sample.x * params.surface.columns as f32).floor() as i32;
    let row_offset = bark_random(params, column, 0, 0x8bd1);
    let row_coordinate = sample.y * params.surface.rows as f32
        + row_offset
        + 0.13 * (tau * (sample.x * 3.0 + row_offset)).sin();
    let row = row_coordinate.floor() as i32;
    let within_row = row_coordinate - row_coordinate.floor();
    let plate_variation = bark_random(params, column, row, 0x2d91) - 0.5;
    let crown_height = params.surface.crown_height_min
        + params.surface.crown_height_span * bark_random(params, column, row, 0x61e3);
    let shoulder_bias = 0.72
        + 0.56
            * bark_random(
                params,
                nearest_crack,
                row + i32::from(signed_edge_distance >= 0.0),
                0xa91f,
            );
    let shoulder_height = (0.025
        + 0.055 * bark_edge_random(params, nearest_cell, second_cell, 0xa91f))
        * shoulder_bias;
    let macro_relief = oak_bark_major_profile(
        edge_distance,
        core_width,
        valley_width,
        primary_run,
        crown_height,
        shoulder_height,
    );
    let crown = smoothstep(core_width * 0.8, valley_width * 1.6, edge_distance);
    let transverse_relief =
        details::transverse_closure(params, column, row, within_row, sample.x, crown);
    let column_coordinate = sample.x * params.surface.columns as f32;
    let within_column = column_coordinate - column_coordinate.floor();
    let plate_tilt =
        ((within_column - 0.5) * 0.15 + (within_row - 0.5) * 0.055) * plate_variation * crown;
    let vertical_bulge = (1.0 - ((within_row - 0.5) * 2.0).powi(2)).max(0.0);
    let plate_bulge =
        (0.035 + 0.055 * bark_random(params, column, row, 0x19d7)) * vertical_bulge * crown;
    let chipped_face = details::chipped_face(params, column, row, within_column, within_row, crown);
    let terminating_branch =
        details::terminating_branch(params, column, row, within_column, within_row, crown);
    let fractured_notch = details::fractured_notch(
        params,
        nearest_crack,
        row,
        edge_distance,
        valley_width,
        within_row,
    );
    let grain = details::grain(sample, crown);
    (macro_relief
        + transverse_relief
        + chipped_face
        + terminating_branch
        + fractured_notch
        + plate_bulge
        + 0.045 * plate_variation * crown
        + plate_tilt
        + grain)
        .clamp(-0.5, 0.38)
}

fn warped_point(point: Vec2) -> Vec2 {
    let tau = core::f32::consts::TAU;
    let Vec2 { x: u, y: v } = point;
    let warp = Vec2::new(
        0.022 * (tau * (v * 2.0 + 0.17)).sin() + 0.009 * (tau * (u * 2.0 - v * 3.0 + 0.41)).sin(),
        0.008 * (tau * (u * 3.0 + 0.63)).sin() + 0.004 * (tau * (u * 5.0 + v * 2.0)).sin(),
    );
    point + warp
}
