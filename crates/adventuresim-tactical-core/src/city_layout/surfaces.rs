//! Semantic ground patches derived from the developed city blocks.

use std::collections::BTreeSet;

use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use bevy::{math::Vec2, prelude::Reflect};
use serde::{Deserialize, Serialize};

use super::*;

const SURFACE_EDGE_TOLERANCE_METRES: f32 = 0.001;
pub const MAX_CITY_STREET_PATCHES: usize = 12_000;
pub const MAX_CITY_YARD_PATCHES: usize = MAX_CITY_LOTS * 3;

/// One developed block interior beneath its buildings and rear courts.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityYardPatch {
    pub corners_metres: [ScenePlanPoint; 4],
    pub surface: CityYardSurface,
}

/// Historically plausible surface treatment for one part of the urban street network.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityStreetSurface {
    CompactedEarth,
    Gravel,
    Fieldstone,
}

/// One bounded surface patch in the connected street network.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "shape")]
pub enum CityStreetPatch {
    Corridor {
        start_metres: ScenePlanPoint,
        end_metres: ScenePlanPoint,
        half_width_metres: PositiveLength,
        surface: CityStreetSurface,
    },
    Market {
        corners_metres: [ScenePlanPoint; 4],
        surface: CityStreetSurface,
    },
}

/// Surface treatment inside one developed urban block.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityYardSurface {
    PackedEarth,
    KitchenGarden,
}

impl CityStreetSurface {
    pub const fn priority(self) -> u8 {
        match self {
            Self::CompactedEarth => 0,
            Self::Gravel => 1,
            Self::Fieldstone => 2,
        }
    }
}

impl CityStreetPatch {
    pub fn surface(self) -> CityStreetSurface {
        match self {
            Self::Corridor { surface, .. } | Self::Market { surface, .. } => surface,
        }
    }

    pub fn contains(self, point: Vec2) -> bool {
        match self {
            Self::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => {
                let start_metres = start_metres.metres();
                let end_metres = end_metres.metres();
                let half_width_metres = half_width_metres.metres();
                let displacement = end_metres - start_metres;
                let fraction = ((point - start_metres).dot(displacement)
                    / displacement.length_squared())
                .clamp(0.0, 1.0);
                point.distance_squared(start_metres + displacement * fraction)
                    <= (half_width_metres + SURFACE_EDGE_TOLERANCE_METRES).powi(2)
            }
            Self::Market { corners_metres, .. } => {
                convex_quad_contains(corners_metres.map(ScenePlanPoint::metres), point)
            }
        }
    }

    pub fn is_valid(self) -> bool {
        match self {
            Self::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => {
                start_metres
                    .metres()
                    .distance_squared(end_metres.metres())
                    .is_finite()
                    && start_metres.metres().distance_squared(end_metres.metres()) > 0.0
                    && (1.0..=20.0).contains(&half_width_metres.metres())
            }
            Self::Market { corners_metres, .. } => corners_metres
                .into_iter()
                .all(|point| point.metres().is_finite()),
        }
    }
}

impl CityYardPatch {
    pub fn from_bounds(bounds: CityPlotBounds, surface: CityYardSurface) -> GeometryResult<Self> {
        let corners = bounds.corners();
        Ok(Self {
            corners_metres: [
                ScenePlanPoint::try_from(corners[0])?,
                ScenePlanPoint::try_from(corners[1])?,
                ScenePlanPoint::try_from(corners[2])?,
                ScenePlanPoint::try_from(corners[3])?,
            ],
            surface,
        })
    }
    pub fn contains(self, point: Vec2) -> bool {
        convex_quad_contains(self.corners_metres.map(ScenePlanPoint::metres), point)
    }

    pub fn is_valid(self) -> bool {
        self.corners_metres
            .into_iter()
            .all(|point| point.metres().is_finite())
            && polygon_area(self.corners_metres.map(ScenePlanPoint::metres)).abs() > 1.0
    }
}

pub(super) fn city_yard_patches(lots: &[CandidateLot]) -> GeometryResult<Vec<CityYardPatch>> {
    let mut patches = Vec::new();
    for candidate in lots {
        let lot = candidate.lot;
        patches.push(CityYardPatch::from_bounds(
            plots::reservation(lot)?,
            CityYardSurface::PackedEarth,
        )?);
    }
    Ok(patches)
}

pub(super) fn city_street_patches(
    graph: &StreetGraph,
    developed_blocks: &BTreeSet<BlockId>,
) -> CityCompileResult<Vec<CityStreetPatch>> {
    graph.developed_streets(developed_blocks)
}

fn convex_quad_contains(corners: [Vec2; 4], point: Vec2) -> bool {
    let mut sign = 0.0_f32;
    for index in 0..4 {
        let start = corners[index];
        let end = corners[(index + 1) % 4];
        let cross = (end - start).perp_dot(point - start);
        if cross.abs() <= f32::EPSILON {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

fn polygon_area(corners: [Vec2; 4]) -> f32 {
    (0..4)
        .map(|index| corners[index].perp_dot(corners[(index + 1) % 4]))
        .sum::<f32>()
        * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn street_decoding_rejects_nonpositive_width_and_keeps_native_coverage() {
        let patch = CityStreetPatch::Corridor {
            start_metres: ScenePlanPoint::ORIGIN,
            end_metres: ScenePlanPoint::try_from(Vec2::new(10.0, 0.0)).unwrap(),
            half_width_metres: PositiveLength::from_metres(2.0).unwrap(),
            surface: CityStreetSurface::Gravel,
        };
        assert!(patch.contains(Vec2::new(5.0, 2.0)));
        assert!(!patch.contains(Vec2::new(5.0, 2.01)));
        let mut wire = serde_json::to_value(patch).unwrap();
        assert_eq!(
            serde_json::from_value::<CityStreetPatch>(wire.clone()).unwrap(),
            patch
        );
        wire["half_width_metres"] = serde_json::json!(-1.0);
        assert!(serde_json::from_value::<CityStreetPatch>(wire).is_err());
    }
}
