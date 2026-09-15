//! Semantic ground patches derived from the developed city blocks.

use std::collections::BTreeSet;

use bevy::{math::Vec2, prelude::Reflect};
use fabelgeist_determinism::mix64;
use serde::{Deserialize, Serialize};

use super::*;

const SURFACE_EDGE_TOLERANCE_METRES: f32 = 0.001;
const YARD_SURFACE_DOMAIN: u64 = 0x7961_7264_5f73_7572;
pub const MAX_CITY_STREET_PATCHES: usize = 12_000;
pub const MAX_CITY_YARD_PATCHES: usize = MAX_CITY_LOTS * 2;

/// Historically plausible surface treatment for one part of the urban street network.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityStreetSurface {
    CompactedEarth,
    Gravel,
    Fieldstone,
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

/// One bounded surface patch in the connected street network.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "shape")]
pub enum CityStreetPatch {
    Corridor {
        start_metres: Vec2,
        end_metres: Vec2,
        half_width_metres: f32,
        surface: CityStreetSurface,
    },
    Market {
        corners_metres: [Vec2; 4],
        surface: CityStreetSurface,
    },
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
                let displacement = end_metres - start_metres;
                let fraction = ((point - start_metres).dot(displacement)
                    / displacement.length_squared())
                .clamp(0.0, 1.0);
                point.distance_squared(start_metres + displacement * fraction)
                    <= (half_width_metres + SURFACE_EDGE_TOLERANCE_METRES).powi(2)
            }
            Self::Market { corners_metres, .. } => convex_quad_contains(corners_metres, point),
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
                start_metres.is_finite()
                    && end_metres.is_finite()
                    && start_metres.distance_squared(end_metres).is_finite()
                    && start_metres.distance_squared(end_metres) > 0.0
                    && half_width_metres.is_finite()
                    && (1.0..=20.0).contains(&half_width_metres)
            }
            Self::Market { corners_metres, .. } => corners_metres.into_iter().all(Vec2::is_finite),
        }
    }
}

/// Surface treatment inside one developed urban block.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityYardSurface {
    PackedEarth,
    KitchenGarden,
}

/// One developed block interior beneath its buildings and rear courts.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityYardPatch {
    pub corners_metres: [Vec2; 4],
    pub surface: CityYardSurface,
}

impl CityYardPatch {
    pub fn contains(self, point: Vec2) -> bool {
        convex_quad_contains(self.corners_metres, point)
    }

    pub fn is_valid(self) -> bool {
        self.corners_metres.into_iter().all(Vec2::is_finite)
            && polygon_area(self.corners_metres).abs() > 1.0
    }
}

pub(super) fn city_yard_patches(seed: u64, lots: &[CandidateLot]) -> Vec<CityYardPatch> {
    let mut patches = Vec::new();
    for candidate in lots {
        let lot = candidate.lot;
        patches.push(CityYardPatch {
            corners_metres: plots::corners(plots::reservation(lot)),
            surface: CityYardSurface::PackedEarth,
        });
        if lot.service.is_none()
            && !lot.has_rear_range()
            && mix64(seed ^ YARD_SURFACE_DOMAIN ^ lot.id).is_multiple_of(3)
        {
            let garden = CityBuildingLot {
                centre_metres: lot.centre_metres
                    + lot.orientation.local_to_world(
                        Vec2::Y * (lot.footprint_metres.y + plots::REAR_COURT_METRES) * 0.5,
                    ),
                footprint_metres: Vec2::new(lot.footprint_metres.x, plots::REAR_COURT_METRES),
                ..lot
            };
            patches.push(CityYardPatch {
                corners_metres: plots::corners(garden),
                surface: CityYardSurface::KitchenGarden,
            });
        }
    }
    patches
}

pub(super) fn city_street_patches(
    graph: &StreetGraph,
    developed_blocks: &BTreeSet<BlockId>,
) -> Vec<CityStreetPatch> {
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
