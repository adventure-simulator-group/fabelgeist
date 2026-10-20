//! Immutable boulder controls retain sampled downhill and noise dependence.
use super::super::{DetailNoise, TerrainShapeSample, detail_smoothstep, streams};
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
use bevy::math::Vec2;
use fabelgeist_determinism::Seed;

const MINIMUM_BOULDER_RELIEF_RADIUS_METRES: f32 = 0.35;

pub(super) struct TerrainRockInfluence {
    centre: ScenePlanPoint,
    radius: PositiveLength,
    // Native software sin/cos values must not be renormalized by a direction API.
    fallback_downhill: Vec2,
    tail_length: PositiveLength,
    debris_seed: Seed,
}

impl TerrainRockInfluence {
    pub(super) fn new(
        detail_seed: Seed,
        centre: ScenePlanPoint,
        radius: PositiveLength,
    ) -> Result<Self, GeometryError> {
        let radius =
            PositiveLength::from_metres(radius.metres().max(MINIMUM_BOULDER_RELIEF_RADIUS_METRES))?;
        let point = centre.metres();
        let rock_seed = streams::ROCK_INFLUENCE.seed(
            detail_seed,
            &[u64::from(point.x.to_bits()), u64::from(point.y.to_bits())],
        );
        let fallback_angle = streams::ROCK_DOWNHILL
            .rng(rock_seed, &[])
            .inclusive_unit_f32()
            * core::f32::consts::TAU;
        let fallback_downhill = Vec2::new(libm::cosf(fallback_angle), libm::sinf(fallback_angle));
        let tail_length = PositiveLength::from_metres(
            radius.metres()
                * (3.2 + streams::ROCK_TAIL.rng(rock_seed, &[]).inclusive_unit_f32() * 1.1),
        )?;
        let debris_seed = streams::ROCK_DEBRIS.seed(rock_seed, &[]);
        Ok(Self {
            centre,
            radius,
            fallback_downhill,
            tail_length,
            debris_seed,
        })
    }

    /// Native scene metres and unnormalized software directions are kernel ports.
    pub(super) fn relief(
        &self,
        point: Vec2,
        shape: Option<TerrainShapeSample>,
        noise: &DetailNoise,
    ) -> Option<f32> {
        let offset = point - self.centre.metres();
        let distance = offset.length();
        let radius = self.radius.metres();
        if distance > radius * 5.0 {
            return None;
        }
        let downhill = shape
            .map(|shape| shape.downhill)
            .unwrap_or(self.fallback_downhill);
        let across_axis = Vec2::new(-downhill.y, downhill.x);
        let socket = (1.0 - detail_smoothstep(radius * 0.48, radius * 1.08, distance)) * -0.042;
        let apron = detail_smoothstep(radius * 0.72, radius * 1.04, distance)
            * (1.0 - detail_smoothstep(radius * 1.04, radius * 1.72, distance))
            * 0.033;
        let downstream = offset.dot(downhill);
        let across = offset.dot(across_axis).abs();
        let tail_length = self.tail_length.metres();
        let longitudinal = detail_smoothstep(radius * 0.45, radius * 0.95, downstream)
            * (1.0 - detail_smoothstep(tail_length * 0.62, tail_length, downstream));
        let tail_width = radius * 0.42 + downstream.max(0.0) * 0.24;
        let lateral = 1.0 - detail_smoothstep(tail_width * 0.42, tail_width, across);
        let granular =
            0.72 + noise.sample(self.debris_seed, Vec2::new(downstream / 1.7, across / 0.8)) * 0.28;
        Some(socket + apron + longitudinal * lateral * granular * 0.034)
    }
}
