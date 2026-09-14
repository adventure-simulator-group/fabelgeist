//! Metric oak bark with narrow raised plates and broad shouldered fissures.
use super::*;

pub(super) const OAK_BARK_TILE_METRES: f32 = 0.5;
pub(super) const OAK_BARK_HEIGHT_RANGE_METRES: f32 = 0.032;
pub(super) const OAK_BARK_COLUMNS: i32 = 10;
pub(super) const OAK_BARK_ROWS: i32 = 6;
pub(super) const OAK_BARK_FISSURE_WIDTH_MIN: f32 = 0.007;
pub(super) const OAK_BARK_FISSURE_WIDTH_SPAN: f32 = 0.003;
pub(super) const OAK_BARK_VALLEY_WIDTH_MIN: f32 = 0.014;
pub(super) const OAK_BARK_VALLEY_WIDTH_SPAN: f32 = 0.010;
pub(super) const OAK_BARK_CROWN_HEIGHT_MIN: f32 = 0.10;
pub(super) const OAK_BARK_CROWN_HEIGHT_SPAN: f32 = 0.12;

pub(super) fn bark_random(
    params: &crate::TextureParameters,
    cell_x: i32,
    cell_y: i32,
    salt: u64,
) -> f32 {
    let hash = crate::parameters::seeded_hash(
        params,
        bark_cell_id(params, cell_x, cell_y) | salt.rotate_left(21),
    );
    unit_hash(hash)
}

fn bark_cell_id(params: &crate::TextureParameters, cell_x: i32, cell_y: i32) -> u64 {
    let wrapped_x = cell_x.rem_euclid(params.surface.columns) as u64;
    let wrapped_y = cell_y.rem_euclid(params.surface.rows) as u64;
    wrapped_x | (wrapped_y << 8)
}

pub(super) fn bark_edge_random(
    params: &crate::TextureParameters,
    first: (i32, i32),
    second: (i32, i32),
    salt: u64,
) -> f32 {
    let first = bark_cell_id(params, first.0, first.1);
    let second = bark_cell_id(params, second.0, second.1);
    let (lower, upper) = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    let hash = crate::parameters::seeded_hash(params, lower | (upper << 16) | salt.rotate_left(37));
    unit_hash(hash)
}

pub(super) fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(1.0e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(super) fn bark_segment_modulation(
    params: &crate::TextureParameters,
    point: Vec2,
    first: (i32, i32),
    second: (i32, i32),
) -> f32 {
    let tau = core::f32::consts::TAU;
    let frequency = 2.0 + (bark_edge_random(params, first, second, 0x8d31) * 3.0).floor();
    let phase = bark_edge_random(params, first, second, 0xc4b7);
    let meander =
        0.13 * (tau * (point.x * 3.0 + bark_edge_random(params, first, second, 0x724d))).sin();
    let wave = (tau * (point.y * frequency + phase + meander)).sin();
    0.48 + 0.52 * smoothstep(-0.55, 0.30, wave)
}

pub(super) fn oak_bark_major_profile(
    edge_distance: f32,
    core_width: f32,
    valley_width: f32,
    run_strength: f32,
    crown_height: f32,
    shoulder_height: f32,
) -> f32 {
    let core = (-0.5 * (edge_distance / core_width).powi(2)).exp();
    let valley = (-0.5 * (edge_distance / valley_width).powi(2)).exp();
    let shoulder_distance = (edge_distance - valley_width * 1.15) / (valley_width * 0.38);
    let shoulder = (-0.5 * shoulder_distance.powi(2)).exp();
    crown_height + shoulder_height * run_strength * shoulder
        - 0.24 * run_strength * valley
        - (0.035 + 0.24 * run_strength) * core
}

pub(super) fn oak_bark_crack_x(params: &crate::TextureParameters, crack: i32, v: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let phase = bark_random(params, crack, 0, 0xd32f);
    let secondary_phase = bark_random(params, crack, 0, 0x82b5);
    let offset =
        (bark_random(params, crack, 0, 0x4c19) - 0.5) * 0.16 / params.surface.columns as f32;
    crack as f32 / params.surface.columns as f32
        + offset
        + 0.0065 * (tau * (v * 2.0 + phase)).sin()
        + 0.0028 * (tau * (v * 5.0 + secondary_phase)).sin()
}

pub(super) fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let axis = end - start;
    let along = ((point - start).dot(axis) / axis.length_squared().max(1.0e-6)).clamp(0.0, 1.0);
    point.distance(start + axis * along)
}

mod height;
pub(crate) use height::oak_bark_height;
mod controls;
mod details;
pub use controls::Parameters;
mod baking;
pub(crate) use baking::{generate_oak_bark_texture, periodic_bilinear_sample, periodic_sample};
#[cfg(test)]
pub(crate) use baking::{oak_bark_horizon_ao, oak_bark_local_cavity};
#[cfg(test)]
mod tests;
