//! Native road-profile probes preserve the deterministic source relief.
use super::{
    DetailNoise, GroundSubstrate, SceneGround, ScenePlanPoint, detail_smoothstep,
    signed_ground_noise,
};
use bevy::math::Vec2;

pub(super) fn road_surface_relief(
    point: ScenePlanPoint,
    ground: &SceneGround,
    noise: &DetailNoise,
) -> f32 {
    let point = point.metres();
    let is_road = |sample: Vec2| {
        ScenePlanPoint::from_metres(sample)
            .and_then(|point| ground.ground_at(point))
            .is_some_and(|surface| surface.substrate == GroundSubstrate::Road)
    };
    let mut tangent = Vec2::X;
    let mut best_score: Option<i32> = None;
    for direction_index in 0..8 {
        let angle = direction_index as f32 * core::f32::consts::PI / 8.0;
        let candidate = Vec2::new(libm::cosf(angle), libm::sinf(angle));
        let score = [0.75_f32, 1.5, 2.5]
            .into_iter()
            .map(|distance| {
                i32::from(is_road(point + candidate * distance))
                    + i32::from(is_road(point - candidate * distance))
            })
            .sum();
        if best_score.is_none_or(|best| score > best) {
            best_score = Some(score);
            tangent = candidate;
        }
    }
    let normal = Vec2::new(-tangent.y, tangent.x);
    let edge_distance = |direction: f32| {
        (1..=24)
            .map(|step| step as f32 * 0.25)
            .find(|distance| !is_road(point + normal * direction * *distance))
            .unwrap_or(6.0)
    };
    let positive_edge = edge_distance(1.0);
    let negative_edge = edge_distance(-1.0);
    let half_width = ((positive_edge + negative_edge) * 0.5).max(0.6);
    let across = (negative_edge - positive_edge) * 0.5;
    let rut_offset = (half_width * 0.46).clamp(0.38, 0.82);
    let rut_width = (half_width * 0.12).clamp(0.14, 0.27);
    // Accepted terrain hashes require software transcendental math on all targets.
    let gaussian = |distance: f32| libm::expf(-(distance / rut_width).powi(2) * 1.7);
    let ruts = gaussian(across - rut_offset) + gaussian(across + rut_offset);
    let crown = (1.0 - (across / half_width).powi(2)).max(0.0) * 0.026;
    let travelled = point.dot(tangent);
    let irregularity = signed_ground_noise(
        noise.fields.road_rut,
        Vec2::new(travelled / 2.4, across / 1.1),
        noise,
    ) * 0.004;
    (crown - ruts * 0.038 + irregularity).clamp(-0.048, 0.032)
}

#[cfg(test)]
mod tests;
