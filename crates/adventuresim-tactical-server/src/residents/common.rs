//! Common residents occupy the actual bounded market square, never invented
//! city-wide coordinates. Collision and route admission still apply at entry.

use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_core::{
    combat::HUMANOID_COLLISION_RADIUS_METRES, tactical_session::MAX_LOCAL_SCENE_RESIDENTS,
};
use adventuresim_stdb_client::SettlementVenueKind;
use adventuresim_tactical_core::{
    city_layout::CityStreetPatch,
    scene::SceneTerrain,
    scene_coordinates::{Scene, ScenePlanPoint},
};
use bevy::prelude::*;

/// Two actor diameters leave a standing gap between neighbouring residents.
const COMMON_STATION_SPACING_METRES: f32 = 4.0 * HUMANOID_COLLISION_RADIUS_METRES;

#[derive(Resource, Default)]
pub(crate) struct CommonResidentStations(pub(super) Vec<Position<Scene>>);

impl CommonResidentStations {
    pub(crate) fn prepare(streets: &[CityStreetPatch], terrain: &SceneTerrain) -> Self {
        let mut positions = Vec::new();
        for street in streets {
            let CityStreetPatch::Market { corners_metres, .. } = street else {
                continue;
            };
            let minimum = corners_metres
                .iter()
                .copied()
                .fold(Vec2::splat(f32::INFINITY), Vec2::min);
            let maximum = corners_metres
                .iter()
                .copied()
                .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
            let mut x = minimum.x + COMMON_STATION_SPACING_METRES;
            while x < maximum.x - COMMON_STATION_SPACING_METRES {
                let mut z = minimum.y + COMMON_STATION_SPACING_METRES;
                while z < maximum.y - COMMON_STATION_SPACING_METRES {
                    let horizontal = Vec2::new(x, z);
                    if street.contains(horizontal)
                        && let Ok(point) = ScenePlanPoint::try_from(horizontal)
                        && let Some(height) = terrain.height_at(point)
                        && let Ok(position) =
                            Position::from_metres(Vec3::new(x, height.metres(), z))
                    {
                        positions.push(position);
                        if positions.len() == MAX_LOCAL_SCENE_RESIDENTS {
                            return Self(positions);
                        }
                    }
                    z += COMMON_STATION_SPACING_METRES;
                }
                x += COMMON_STATION_SPACING_METRES;
            }
        }
        Self(positions)
    }

    pub(super) fn venues(&self) -> Vec<SettlementVenueKind> {
        if self.0.is_empty() {
            Vec::new()
        } else {
            vec![SettlementVenueKind::PublicSquare]
        }
    }
}
