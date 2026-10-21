//! Immutable root shapes retain the existing per-sample relief arithmetic.
use super::super::{detail_smoothstep, rock_recipe::wrapped_angle_difference, streams};
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength, Radians};
use bevy::math::Vec2;
use fabelgeist_determinism::Seed;

const ROOT_RIDGE_COUNT: u64 = 7;

pub(super) struct TerrainTreeInfluence {
    centre: ScenePlanPoint,
    ridges: Vec<TerrainRootRidge>,
}

struct TerrainRootRidge {
    origin: Radians,
    phase: Radians,
    length: PositiveLength,
}

impl TerrainTreeInfluence {
    pub(super) fn new(detail_seed: Seed, centre: ScenePlanPoint) -> Result<Self, GeometryError> {
        let tree = centre.metres();
        let tree_seed = streams::TREE_ROOTS.seed(
            detail_seed,
            &[u64::from(tree.x.to_bits()), u64::from(tree.y.to_bits())],
        );
        let ridges = (0..ROOT_RIDGE_COUNT)
            .map(|root| {
                let mut random = streams::ROOT_SHAPE.rng(tree_seed, &[root]);
                Ok(TerrainRootRidge {
                    origin: Radians::new(random.inclusive_unit_f32() * core::f32::consts::TAU)?,
                    phase: Radians::new(random.inclusive_unit_f32() * core::f32::consts::TAU)?,
                    length: PositiveLength::from_metres(4.8 + random.inclusive_unit_f32() * 2.9)?,
                })
            })
            .collect::<Result<Vec<_>, GeometryError>>()?;
        Ok(Self { centre, ridges })
    }

    /// Native scene east/north metres feed the terrain sampler's numeric port.
    pub(super) fn relief(&self, point: Vec2) -> Option<f32> {
        let offset = point - self.centre.metres();
        let radius = offset.length();
        if radius > 8.0 {
            return None;
        }
        let mound = libm::expf(-(radius / 1.35).powi(2)) * 0.045;
        let basin = detail_smoothstep(0.9, 1.8, radius)
            * (1.0 - detail_smoothstep(5.2, 7.7, radius))
            * -0.012;
        let angle = libm::atan2f(offset.y, offset.x);
        let mut ridges = 0.0_f32;
        for ridge in &self.ridges {
            // Checked leaves preserve the original f32 radians/metres exactly.
            let origin = ridge.origin.radians();
            let phase = ridge.phase.radians();
            let length = ridge.length.metres();
            if radius > length || radius < 0.28 {
                continue;
            }
            let curved_angle = origin + libm::sinf(radius * 0.9 + phase) * 0.11;
            let angular_distance = wrapped_angle_difference(angle, curved_angle).abs() * radius;
            let width = 0.16 + radius * 0.045;
            let ridge = 1.0 - detail_smoothstep(width * 0.3, width, angular_distance);
            let taper = 1.0 - detail_smoothstep(length * 0.58, length, radius);
            ridges = ridges.max(ridge * taper * (0.082 - radius * 0.007).max(0.025));
        }
        Some(mound + basin + ridges)
    }
}
