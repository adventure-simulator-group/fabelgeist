//! Exact managed-ground ownership, independent of terrain grading and sampling.
use super::*;
use crate::{
    city_layout::{CityPlotBounds, CityStreetPatch, CityStreetSurface, CityYardPatch},
    scene_input::GeneratedBuilding,
};

#[derive(Default, Serialize, Deserialize, Debug, Reflect, Clone, PartialEq)]
pub struct UrbanGroundSurfaces {
    streets: Vec<CityStreetPatch>,
    yards: Vec<CityYardPatch>,
    buildings: Vec<CityPlotBounds>,
}

impl UrbanGroundSurfaces {
    pub(crate) fn new(
        streets: &[CityStreetPatch],
        yards: &[CityYardPatch],
        buildings: &[GeneratedBuilding],
    ) -> Self {
        Self {
            streets: streets.to_vec(),
            yards: yards.to_vec(),
            buildings: buildings
                .iter()
                .map(|building| CityPlotBounds {
                    centre_metres: building.placement.centre_metres,
                    dimensions_metres: building.collision.bounds.plan_half_extents() * 2.0,
                    orientation: building.placement.orientation,
                })
                .collect(),
        }
    }

    pub fn surface_at(&self, position: Vec2) -> Option<GroundSurface> {
        let substrate = if let Some(street) = self
            .streets
            .iter()
            .filter(|street| street.contains(position))
            .max_by_key(|street| street.surface().priority())
        {
            match street.surface() {
                CityStreetSurface::CompactedEarth => GroundSubstrate::Soil,
                CityStreetSurface::Gravel => GroundSubstrate::Gravel,
                CityStreetSurface::Fieldstone => GroundSubstrate::Road,
            }
        } else if self
            .buildings
            .iter()
            .any(|bounds| bounds.contains(position))
        {
            GroundSubstrate::Stone
        } else if self.yards.iter().any(|yard| yard.contains(position)) {
            GroundSubstrate::Soil
        } else {
            return None;
        };
        Some(GroundSurface {
            substrate,
            cover: GroundCover::Bare,
            cover_density_bps: 0,
            cover_height_cm: 0,
        })
    }

    /// Conservative exclusion for a rotating wild-scatter patch's full envelope.
    pub fn overlaps_disk(&self, centre: Vec2, radius_metres: f32) -> bool {
        self.streets.iter().any(|street| match *street {
            CityStreetPatch::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => {
                segment_distance(centre, start_metres, end_metres)
                    <= radius_metres + half_width_metres
            }
            CityStreetPatch::Market { corners_metres, .. } => {
                street.contains(centre) || touches_edges(centre, radius_metres, corners_metres)
            }
        }) || self.yards.iter().any(|yard| {
            yard.contains(centre) || touches_edges(centre, radius_metres, yard.corners_metres)
        }) || self.buildings.iter().any(|bounds| {
            bounds.contains(centre) || touches_edges(centre, radius_metres, bounds.corners())
        })
    }

    #[cfg(test)]
    pub(super) fn garden_test() -> Self {
        Self {
            yards: vec![CityYardPatch {
                corners_metres: [
                    Vec2::new(0.3, 0.3),
                    Vec2::new(1.7, 0.3),
                    Vec2::new(1.7, 1.7),
                    Vec2::new(0.3, 1.7),
                ],
                surface: crate::city_layout::CityYardSurface::KitchenGarden,
            }],
            ..Default::default()
        }
    }
}

fn segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    let fraction =
        ((point - start).dot(edge) / edge.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
    point.distance(start + edge * fraction)
}
fn touches_edges(centre: Vec2, radius: f32, corners: [Vec2; 4]) -> bool {
    (0..4).any(|side| segment_distance(centre, corners[side], corners[(side + 1) % 4]) <= radius)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subcell_garden_ownership_survives_transport_without_clearing_neighbouring_grass() {
        let terrain = SceneTerrain::new(4, 4, 2.0, |_| 0.0);
        let mut ground = SceneGround::uniform_for_terrain(
            &terrain,
            GroundSurface {
                cover_density_bps: 9600,
                ..Default::default()
            },
        );
        ground.urban = UrbanGroundSurfaces::garden_test();
        assert!(ground.urban.overlaps_disk(Vec2::new(0.1, 1.0), 0.3));
        assert!(!ground.urban.overlaps_disk(Vec2::new(0.1, 1.0), 0.1));
        let transported: SceneGround =
            serde_json::from_slice(&serde_json::to_vec(&ground).unwrap()).unwrap();
        for point in [Vec2::new(0.31, 0.31), Vec2::new(1.69, 1.69)] {
            assert_eq!(transported.ground_at(point).unwrap().cover_density_bps, 0);
        }
        assert_eq!(
            transported
                .ground_at(Vec2::new(0.29, 0.31))
                .unwrap()
                .cover_density_bps,
            9600
        );
    }
}
