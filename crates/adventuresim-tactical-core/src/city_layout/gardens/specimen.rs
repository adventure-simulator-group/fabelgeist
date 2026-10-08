//! Measured production geometry shared by garden siting and shrub rendering.
use bevy::{
    math::{Vec2, Vec3},
    prelude::Reflect,
};
use fabelgeist_determinism::Seed;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Conservative world-space reserve for the leaf shader's displacement.
/// Its current 0.035m strength permits less than 0.036m horizontal movement.
pub const GARDEN_LEAF_WIND_CLEARANCE_METRES: f32 = 0.04;
pub const GARDEN_LEAF_WIND_STRENGTH_METRES: f32 = 0.035;
const ENVELOPE_TOLERANCE_METRES: f32 = 0.00001;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenSpecimenEnvelope {
    pub geometry_revision: u16,
    pub seed: Seed,
    pub hull_metres: Vec<Vec2>,
    pub min_height_metres: f32,
    pub max_height_metres: f32,
    pub vertex_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GardenSpecimen {
    CommonHazel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, thiserror::Error)]
pub enum GardenSpecimenError {
    #[error("garden specimen envelope cannot be decoded")]
    MalformedEnvelope,
    #[error("garden specimen envelope has invalid native geometry")]
    InvalidGeometry,
}

impl GardenSpecimen {
    pub fn envelope(self) -> Result<&'static GardenSpecimenEnvelope, GardenSpecimenError> {
        static HAZEL: LazyLock<Result<GardenSpecimenEnvelope, GardenSpecimenError>> =
            LazyLock::new(|| {
                let envelope: GardenSpecimenEnvelope = serde_json::from_str(include_str!(
                    "../../../../../content/tactical/garden-hazel-envelope.json"
                ))
                .map_err(|_| GardenSpecimenError::MalformedEnvelope)?;
                envelope.validate()?;
                Ok(envelope)
            });
        match self {
            Self::CommonHazel => HAZEL.as_ref().map_err(|issue| *issue),
        }
    }
}

impl GardenSpecimenEnvelope {
    /// Asset-native metre vectors retain the renderer's local coordinate
    /// representation. They enter scene planning only through world_hull.
    fn validate(&self) -> Result<(), GardenSpecimenError> {
        if !self.min_height_metres.is_finite()
            || !self.max_height_metres.is_finite()
            || self.min_height_metres > self.max_height_metres
            || self.hull_metres.len() < 3
            || self.hull_metres.iter().any(|point| !point.is_finite())
        {
            return Err(GardenSpecimenError::InvalidGeometry);
        }
        for index in 0..self.hull_metres.len() {
            let a = self.hull_metres[index];
            let b = self.hull_metres[(index + 1) % self.hull_metres.len()];
            if a == b
                || self
                    .hull_metres
                    .iter()
                    .any(|point| (b - a).perp_dot(*point - a) < 0.0)
            {
                return Err(GardenSpecimenError::InvalidGeometry);
            }
        }
        Ok(())
    }

    pub fn contains_local_vertex(&self, point: Vec3) -> bool {
        point.is_finite()
            && point.y >= self.min_height_metres - ENVELOPE_TOLERANCE_METRES
            && point.y <= self.max_height_metres + ENVELOPE_TOLERANCE_METRES
            && self
                .hull_metres
                .iter()
                .zip(self.hull_metres.iter().cycle().skip(1))
                .all(|(a, b)| {
                    (*b - *a).perp_dot(Vec2::new(point.x, point.z) - *a) / b.distance(*a)
                        >= -ENVELOPE_TOLERANCE_METRES
                })
    }
}
