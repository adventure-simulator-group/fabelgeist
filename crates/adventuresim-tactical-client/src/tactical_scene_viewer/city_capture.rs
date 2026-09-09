use adventuresim_tactical_core::prelude::{
    BuildingOrientation, CityStreetPatch, DistantBuildingPlacement, GeneratedBuilding, SceneTerrain,
};
use bevy::prelude::*;

use super::capture_state::BuildingReviewCamera;

const STREET_EYE_HEIGHT_METRES: f32 = 1.65;
const STREET_CAMERA_CLEARANCE_METRES: f32 = 0.75;
const STREET_CONTEXT_OFFSET_METRES: f32 = 8.0;
const FACADE_STREET_SEARCH_METRES: f32 = 40.0;
const NEIGHBOURHOOD_RADIUS_METRES: f32 = 75.0;

pub(super) fn capture_cameras(
    buildings: &[GeneratedBuilding],
    distant_buildings: &[DistantBuildingPlacement],
    streets: &[CityStreetPatch],
    terrain: &SceneTerrain,
    profile: &str,
) -> Vec<BuildingReviewCamera> {
    if profile != super::CITY_REVIEW_PROFILE {
        return Vec::new();
    }
    let bounds = PlacementBounds::collect(buildings, distant_buildings);
    let clear = |point| {
        bounds.iter().all(|bounds| !bounds.contains(point)) && terrain.height_at(point).is_some()
    };
    let mut focus_candidates = buildings.iter().collect::<Vec<_>>();
    focus_candidates.sort_by(|left, right| {
        left.placement
            .centre_metres
            .length_squared()
            .total_cmp(&right.placement.centre_metres.length_squared())
    });
    let (focus, facade_target, street) = focus_candidates
        .into_iter()
        .find_map(|focus| {
            let transform = super::buildings::building_transform(focus);
            let width = f32::from(focus.placement.program.footprint.dimensions().0)
                * adventuresim_building_generator::CELL_SIZE_METRES;
            let target = transform.transform_point(
                Vec3::new(width * 0.5, 2.4, 0.0) - focus.collision.bounds.centre(),
            );
            let outward = focus.placement.orientation.local_to_world(Vec2::NEG_Y);
            StreetPosition::nearest(streets, target.xz(), outward, &clear)
                .map(|street| (focus, target, street))
        })
        .expect("city-review fixture has no clear street corridor facing a playable building");
    let eye = |point: Vec2| {
        Vec3::new(
            point.x,
            terrain
                .height_at(point)
                .expect("street review stays on playable terrain")
                + STREET_EYE_HEIGHT_METRES,
            point.y,
        )
    };
    let context = [-STREET_CONTEXT_OFFSET_METRES, STREET_CONTEXT_OFFSET_METRES]
        .into_iter()
        .map(|offset| street.shifted(offset))
        .find(|candidate| clear(candidate.point()))
        .unwrap_or(street);
    let street_target = if context.fraction < 0.5 {
        context.end
    } else {
        context.start
    };
    let centres = buildings
        .iter()
        .map(|building| building.placement.centre_metres)
        .chain(
            distant_buildings
                .iter()
                .map(|building| building.centre_metres),
        )
        .collect::<Vec<_>>();
    let neighbourhood_bounds = Bounds2::from_points(centres.iter().copied().filter(|point| {
        point.distance_squared(focus.placement.centre_metres) <= NEIGHBOURHOOD_RADIUS_METRES.powi(2)
    }));
    let playable_bounds = Bounds2::from_points(
        buildings
            .iter()
            .map(|building| building.placement.centre_metres),
    );
    let city_bounds = Bounds2::from_points(centres.into_iter());

    vec![
        BuildingReviewCamera {
            position: eye(street.point()),
            target: facade_target,
            plaster_raking_light: None,
        },
        BuildingReviewCamera {
            position: eye(context.point()),
            target: Vec3::new(street_target.x, eye(context.point()).y, street_target.y),
            plaster_raking_light: None,
        },
        playable_bounds.oblique_camera(0.95, 0.8),
        neighbourhood_bounds.oblique_camera(0.72, 0.42),
        city_bounds.edge_camera(),
        city_bounds.aerial_camera(),
        city_bounds.horizon_camera(),
    ]
}

#[derive(Clone, Copy)]
struct StreetPosition {
    start: Vec2,
    end: Vec2,
    fraction: f32,
}

impl StreetPosition {
    fn point(self) -> Vec2 {
        self.start.lerp(self.end, self.fraction)
    }

    fn shifted(self, metres: f32) -> Self {
        Self {
            fraction: (self.fraction + metres / self.start.distance(self.end)).clamp(0.05, 0.95),
            ..self
        }
    }

    fn nearest(
        streets: &[CityStreetPatch],
        target: Vec2,
        outward: Vec2,
        clear: &impl Fn(Vec2) -> bool,
    ) -> Option<Self> {
        streets
            .iter()
            .filter_map(|street| {
                let CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    ..
                } = *street
                else {
                    return None;
                };
                let displacement = end_metres - start_metres;
                Some(Self {
                    start: start_metres,
                    end: end_metres,
                    fraction: ((target - start_metres).dot(displacement)
                        / displacement.length_squared())
                    .clamp(0.05, 0.95),
                })
            })
            .flat_map(|street| {
                [
                    street,
                    street.shifted(-STREET_CONTEXT_OFFSET_METRES),
                    street.shifted(STREET_CONTEXT_OFFSET_METRES),
                ]
            })
            .filter(|street| {
                let offset = street.point() - target;
                offset.dot(outward) > STREET_CAMERA_CLEARANCE_METRES
                    && offset.length_squared() < FACADE_STREET_SEARCH_METRES.powi(2)
                    && clear(street.point())
            })
            .min_by(|a, b| {
                a.point()
                    .distance_squared(target)
                    .total_cmp(&b.point().distance_squared(target))
            })
    }
}

struct PlacementBounds {
    centre: Vec2,
    half_extents: Vec2,
    orientation: BuildingOrientation,
}

impl PlacementBounds {
    fn collect(buildings: &[GeneratedBuilding], distant: &[DistantBuildingPlacement]) -> Vec<Self> {
        buildings
            .iter()
            .map(|building| Self {
                centre: building.placement.centre_metres,
                half_extents: building.collision.bounds.plan_half_extents(),
                orientation: building.placement.orientation,
            })
            .chain(distant.iter().map(|building| Self {
                centre: building.centre_metres,
                half_extents: building.program().plot_dimensions_metres() * 0.5,
                orientation: building.orientation,
            }))
            .collect()
    }

    fn contains(&self, point: Vec2) -> bool {
        self.orientation
            .world_to_local(point - self.centre)
            .abs()
            .cmple(self.half_extents + Vec2::splat(STREET_CAMERA_CLEARANCE_METRES))
            .all()
    }
}

#[derive(Clone, Copy)]
struct Bounds2 {
    min: Vec2,
    max: Vec2,
}

impl Bounds2 {
    fn from_points(points: impl Iterator<Item = Vec2>) -> Self {
        let bounds = points.fold(None::<Self>, |bounds, point| {
            Some(match bounds {
                None => Self {
                    min: point,
                    max: point,
                },
                Some(bounds) => Self {
                    min: bounds.min.min(point),
                    max: bounds.max.max(point),
                },
            })
        });
        bounds.expect("city-review fixture lacks building placements")
    }

    fn centre(self) -> Vec3 {
        let centre = (self.min + self.max) * 0.5;
        Vec3::new(centre.x, 4.0, centre.y)
    }

    fn radius(self) -> f32 {
        ((self.max - self.min).max_element() * 0.5).max(20.0)
    }

    fn oblique_camera(self, distance_scale: f32, height_scale: f32) -> BuildingReviewCamera {
        let target = self.centre();
        let radius = self.radius();
        BuildingReviewCamera {
            position: target + Vec3::new(radius * distance_scale, radius * height_scale, radius),
            target,
            plaster_raking_light: None,
        }
    }

    fn edge_camera(self) -> BuildingReviewCamera {
        let target = self.centre();
        let radius = self.radius();
        BuildingReviewCamera {
            position: target + Vec3::new(radius * 1.45, radius * 0.18, radius * 0.35),
            target,
            plaster_raking_light: None,
        }
    }

    fn aerial_camera(self) -> BuildingReviewCamera {
        let target = self.centre();
        let radius = self.radius();
        BuildingReviewCamera {
            position: target + Vec3::new(0.0, radius * 1.7, radius * 0.18),
            target,
            plaster_raking_light: None,
        }
    }

    fn horizon_camera(self) -> BuildingReviewCamera {
        let target = self.centre();
        let radius = self.radius();
        BuildingReviewCamera {
            position: target + Vec3::new(0.0, radius * 0.12, radius * 2.35),
            target,
            plaster_raking_light: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::{
        BuildingArchetype, BuildingProgram, compile_building_collision, generate,
    };
    use adventuresim_tactical_core::prelude::{CityStreetSurface, TacticalBuildingPlacement};

    fn house(id: u64, centre: Vec2, orientation: BuildingOrientation) -> GeneratedBuilding {
        let placement = TacticalBuildingPlacement {
            id,
            program: BuildingProgram::fixture(BuildingArchetype::TownHouse, 42),
            centre_metres: centre,
            orientation,
        };
        let plan = generate(&placement.program).unwrap();
        GeneratedBuilding {
            placement,
            collision: compile_building_collision(&plan),
            plan,
            pad_elevation_metres: 3.0,
        }
    }

    fn corridor(orientation: BuildingOrientation) -> CityStreetPatch {
        CityStreetPatch::Corridor {
            start_metres: orientation.local_to_world(Vec2::new(-60.0, 0.0)),
            end_metres: orientation.local_to_world(Vec2::new(60.0, 0.0)),
            half_width_metres: 3.0,
            surface: CityStreetSurface::Fieldstone,
        }
    }

    #[test]
    fn city_packet_spans_close_and_whole_settlement_views() {
        let buildings = [house(
            1,
            Vec2::new(0.0, 12.0),
            BuildingOrientation::IDENTITY,
        )];
        let distant_city = DistantBuildingPlacement {
            usage: None,
            service_size: None,
            id: 2,
            archetype: BuildingArchetype::FachwerkCottage,
            seed: 7,
            centre_metres: Vec2::new(300.0, 240.0),
            base_elevation_metres: 0.0,
            orientation: BuildingOrientation::IDENTITY,
        };
        let distant = [
            distant_city,
            DistantBuildingPlacement {
                id: 3,
                centre_metres: Vec2::new(70.0, 12.0),
                ..distant_city
            },
        ];

        let cameras = capture_cameras(
            &buildings,
            &distant,
            &[corridor(BuildingOrientation::IDENTITY)],
            &SceneTerrain::new(65, 65, 4.0, |_| 3.0),
            crate::tactical_scene_viewer::CITY_REVIEW_PROFILE,
        );

        assert_eq!(cameras.len(), 7);
        assert!(cameras.iter().all(|camera| {
            camera.position.is_finite()
                && camera.target.is_finite()
                && camera.position.distance(camera.target) > 0.5
        }));
        assert!(cameras[5].position.y > cameras[0].position.y + 100.0);
        assert_eq!(cameras[2].target.xz(), buildings[0].placement.centre_metres);
        assert_eq!(cameras[3].target.xz(), Vec2::new(35.0, 12.0));
        assert!(cameras[3].position.distance(cameras[3].target) < 75.0);
        assert!(cameras[5].position.distance(cameras[5].target) > 150.0);
    }

    #[test]
    fn eye_level_views_stay_in_rotated_street_outside_both_rows_of_buildings() {
        let orientation = BuildingOrientation::from_radians(0.63).unwrap();
        let mut focus = house(1, Vec2::ZERO, orientation);
        let local_centre = Vec2::new(0.0, focus.collision.bounds.plan_half_extents().y + 4.0);
        focus.placement.centre_metres = orientation.local_to_world(local_centre);
        let opposite = house(
            2,
            orientation.local_to_world(-local_centre),
            BuildingOrientation::from_radians(0.63 + std::f32::consts::PI).unwrap(),
        );
        let buildings = [focus, opposite];
        let street = corridor(orientation);
        let terrain = SceneTerrain::new(65, 65, 4.0, |_| 3.0);
        let cameras = capture_cameras(
            &buildings,
            &[],
            &[street],
            &terrain,
            crate::tactical_scene_viewer::CITY_REVIEW_PROFILE,
        );

        for camera in &cameras[..2] {
            let point = camera.position.xz();
            assert!(
                street.contains(point),
                "eye must be in the actual street corridor"
            );
            for building in &buildings {
                let local = building
                    .placement
                    .orientation
                    .world_to_local(point - building.placement.centre_metres);
                assert!(
                    local
                        .abs()
                        .cmpgt(building.collision.bounds.plan_half_extents())
                        .any(),
                    "eye must be outside every placed building"
                );
            }
            assert!((camera.position.y - 4.65).abs() < 0.001);
        }
        let front_eye = orientation
            .world_to_local(cameras[0].position.xz() - buildings[0].placement.centre_metres);
        assert!(front_eye.y < 0.0, "the front is local negative Z");
    }
}
