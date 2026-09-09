use super::*;
use crate::city_layout::CityStreetPatch;

const DOOR_APPROACH_METRES: f32 = 4.0;
const DOOR_SHOULDER_METRES: f32 = 0.5;
pub(super) const MARKET_AISLE_HALF_WIDTH_METRES: f32 = 2.0;
const EDGE_ROAD_MIN_ALIGNMENT: f32 = 0.95;

/// Adjacent streets can be wider than the market's pedestrian perimeter aisle.
/// Start vendor rows behind their full reserved width, including angled caps.
pub(super) fn market_edge_clearance(
    input: &TacticalSceneInput,
    start: Vec2,
    end: Vec2,
    inward: Vec2,
) -> f32 {
    let tangent = (end - start).normalize_or_zero();
    let edge_length = start.distance(end);
    let mut clearance = MARKET_AISLE_HALF_WIDTH_METRES;
    for patch in &input.streets {
        let CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            half_width_metres,
            ..
        } = *patch
        else {
            continue;
        };
        if (end_metres - start_metres)
            .normalize_or_zero()
            .dot(tangent)
            .abs()
            < EDGE_ROAD_MIN_ALIGNMENT
        {
            continue;
        }
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for corner in route(start_metres, end_metres, half_width_metres).corners() {
            let offset = corner - start;
            let projected = Vec2::new(offset.dot(tangent), offset.dot(inward));
            minimum = minimum.min(projected);
            maximum = maximum.max(projected);
        }
        if maximum.x >= 0.0
            && minimum.x <= edge_length
            && maximum.y >= 0.0
            && minimum.y <= MARKET_AISLE_HALF_WIDTH_METRES
        {
            clearance = clearance.max(maximum.y);
        }
    }
    clearance
}

pub(super) fn route(start: Vec2, end: Vec2, half_width: f32) -> FurnitureFootprint {
    FurnitureFootprint {
        centre_metres: (start + end) * 0.5,
        half_extents_metres: Vec2::new(start.distance(end) * 0.5 + half_width, half_width),
        orientation: BuildingOrientation::from_frontage_tangent(end - start)
            .unwrap_or(BuildingOrientation::IDENTITY),
    }
}

pub(super) fn routes(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
) -> Vec<FurnitureFootprint> {
    let mut routes = Vec::new();
    for patch in &input.streets {
        match *patch {
            CityStreetPatch::Corridor {
                start_metres,
                end_metres,
                half_width_metres,
                ..
            } => {
                routes.push(route(start_metres, end_metres, half_width_metres));
            }
            CityStreetPatch::Market {
                corners_metres: corners,
                ..
            } => {
                // Preserve both crossings and a perimeter circuit before any
                // stalls are proposed in the remaining quadrants.
                for edge in 0..4 {
                    routes.push(route(
                        corners[edge],
                        corners[(edge + 1) % 4],
                        MARKET_AISLE_HALF_WIDTH_METRES,
                    ));
                }
                for edge in 0..2 {
                    let start = (corners[edge] + corners[edge + 1]) * 0.5;
                    let end = (corners[(edge + 2) % 4] + corners[(edge + 3) % 4]) * 0.5;
                    routes.push(route(start, end, MARKET_AISLE_HALF_WIDTH_METRES));
                }
            }
        }
    }
    for building in buildings {
        for opening in building.plan.opening_assemblies.iter().filter(|opening| {
            opening.use_kind == adventuresim_building_generator::OpeningUse::Door
                && opening.frame.outside_room.is_none()
        }) {
            let start = building_point(building, opening.frame.origin);
            let outward = building
                .placement
                .orientation
                .local_to_world(opening.frame.outward);
            routes.push(route(
                start,
                start + outward * DOOR_APPROACH_METRES,
                opening.profile.exterior_width_metres() * 0.5 + DOOR_SHOULDER_METRES,
            ));
        }
        if let Some(workplace) = &building.plan.workplace {
            for passage in &workplace.passages {
                let min = Vec2::new(passage.min.x, passage.min.z);
                let max = Vec2::new(passage.max.x, passage.max.z);
                routes.push(FurnitureFootprint {
                    centre_metres: building_point(building, (min + max) * 0.5),
                    half_extents_metres: (max - min) * 0.5,
                    orientation: building.placement.orientation,
                });
            }
        }
    }
    routes
}

pub(super) fn building_point(building: &GeneratedBuilding, local: Vec2) -> Vec2 {
    let origin = building.collision.bounds.centre();
    building.placement.centre_metres
        + building
            .placement
            .orientation
            .local_to_world(local - Vec2::new(origin.x, origin.z))
}

pub(super) fn obstacles(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    buildings: &[GeneratedBuilding],
    obstacles: &[GeneratedObstacle],
) -> Vec<FurnitureFootprint> {
    let mut footprints = buildings
        .iter()
        .map(|building| FurnitureFootprint {
            centre_metres: building.placement.centre_metres,
            half_extents_metres: building.collision.bounds.plan_half_extents(),
            orientation: building.placement.orientation,
        })
        .collect::<Vec<_>>();
    for obstacle in obstacles {
        let (x, z, radius) = match *obstacle {
            GeneratedObstacle::Tree { x, z } => (x, z, super::super::TREE_TRUNK_RADIUS_METRES),
            GeneratedObstacle::Rock { x, z, recipe } => (x, z, recipe.collision_radius_metres()),
        };
        footprints.push(FurnitureFootprint {
            centre_metres: Vec2::new(f32::from(x), f32::from(z)) * input.playable.spacing_metres
                - Vec2::new(terrain.width(), terrain.depth()) * 0.5,
            half_extents_metres: Vec2::splat(radius),
            orientation: BuildingOrientation::IDENTITY,
        });
    }
    footprints
}
