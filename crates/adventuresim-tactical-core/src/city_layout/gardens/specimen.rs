//! Measured production geometry shared by garden siting and shrub rendering.
use bevy::{
    math::{Vec2, Vec3},
    prelude::Reflect,
};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Conservative world-space reserve for the leaf shader's displacement.
/// Its current 0.035m strength permits less than 0.036m horizontal movement.
pub const GARDEN_LEAF_WIND_CLEARANCE_METRES: f32 = 0.04;
pub const GARDEN_LEAF_WIND_STRENGTH_METRES: f32 = 0.035;
const ENVELOPE_TOLERANCE_METRES: f32 = 0.00001;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GardenSpecimen {
    CommonHazel,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenSpecimenEnvelope {
    pub geometry_revision: u16,
    pub seed: u64,
    pub hull_metres: Vec<Vec2>,
    pub min_height_metres: f32,
    pub max_height_metres: f32,
    pub vertex_count: usize,
}

impl GardenSpecimen {
    pub fn envelope(self) -> &'static GardenSpecimenEnvelope {
        static HAZEL: LazyLock<GardenSpecimenEnvelope> = LazyLock::new(|| {
            serde_json::from_str(include_str!(
                "../../../../../content/tactical/garden-hazel-envelope.json"
            ))
            .expect("checked common-hazel geometry envelope")
        });
        match self {
            Self::CommonHazel => &HAZEL,
        }
    }
}

impl GardenSpecimenEnvelope {
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
