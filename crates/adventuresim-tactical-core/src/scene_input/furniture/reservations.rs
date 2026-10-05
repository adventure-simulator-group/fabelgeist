use super::*;
use crate::city_layout::CityStreetPatch;

pub(super) const DOOR_APPROACH_METRES: f32 = 4.0;
pub(super) const DOOR_SHOULDER_METRES: f32 = 0.5;
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
    buildings: &[sites::FurnitureSite],
) -> Result<Vec<FurnitureFootprint>, adventuresim_building_generator::DoorError> {
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
        routes.extend(building.routes.iter().copied());
    }
    if let Some(projection) = &input.grounding {
        // The accepted approach can include a property setback as well as the
        // bounded street apron. Reserve the actual grading contract, including
        // entrances expressed as workplace passages without a door assembly.
        routes.extend(projection.surfaces().iter().flat_map(|surface| {
            surface
                .doorway_approaches()
                .iter()
                .copied()
                .map(FurnitureFootprint::accepted_approach)
        }));
    }
    for compound in &input.compounds {
        routes.extend(compound.access.iter().map(|access| {
            route(
                access.start_metres,
                access.end_metres,
                access.half_width_metres,
            )
        }));
        routes.push(FurnitureFootprint::gate_sweep(compound)?);
    }
    for garden in &input.gardens {
        routes.extend(
            garden
                .access
                .iter()
                .map(|a| route(a.start_metres, a.end_metres, a.half_width_metres)),
        );
        routes.push(FurnitureFootprint {
            centre_metres: garden.cultivated_bounds.centre_metres,
            half_extents_metres: garden.cultivated_bounds.dimensions_metres * 0.5,
            orientation: garden.cultivated_bounds.orientation,
        });
    }
    Ok(routes)
}

pub(super) fn obstacles(
    input: &TacticalSceneInput,
    terrain: &SceneTerrain,
    buildings: &[sites::FurnitureSite],
    obstacles: &[GeneratedObstacle],
) -> Result<Vec<FurnitureFootprint>, super::super::SceneInputError> {
    let mut footprints = buildings
        .iter()
        .map(|building| FurnitureFootprint {
            centre_metres: building.placement.centre_metres,
            half_extents_metres: building.half_extents,
            orientation: building.placement.orientation,
        })
        .collect::<Vec<_>>();
    for compound in &input.compounds {
        footprints.extend(compound.boundary.fixed_members()?.iter().map(|member| {
            FurnitureFootprint {
                centre_metres: member.pose.plan_metres(),
                half_extents_metres: Vec2::new(
                    member.size_metres.metres().x,
                    member.size_metres.metres().z,
                ) * 0.5,
                orientation: member.orientation,
            }
        }));
    }
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
    Ok(footprints)
}

impl FurnitureFootprint {
    pub(super) fn accepted_approach(region: crate::city_layout::CityPlotBounds) -> Self {
        Self {
            centre_metres: region.centre_metres,
            half_extents_metres: region.dimensions_metres * 0.5 + Vec2::splat(DOOR_SHOULDER_METRES),
            orientation: region.orientation,
        }
    }

    fn gate_sweep(
        compound: &crate::city_layout::CityCompound,
    ) -> Result<Self, adventuresim_building_generator::DoorError> {
        let gate = compound.boundary.gate.door(compound.id)?;
        let hinge = Vec2::new(gate.hinge_centre.metres().x, gate.hinge_centre.metres().z);
        // Reserve the inward quarter of the hinge's enclosing square for the
        // complete leaf sweep, independently of its current dynamic state.
        Ok(Self {
            centre_metres: hinge
                + compound
                    .boundary
                    .gate
                    .orientation
                    .local_to_world(Vec2::new(-compound.boundary.gate.hinge.sign(), 1.0))
                    * gate.size_metres.metres().x
                    * 0.5,
            half_extents_metres: Vec2::splat(
                gate.size_metres.metres().x * 0.5 + gate.size_metres.metres().z,
            ),
            orientation: compound.boundary.gate.orientation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city_layout::PropertySide;
    use bevy::math::{Quat, Vec3};

    #[test]
    fn furniture_reservation_contains_both_complete_gate_sweeps() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../../assets/tactical-scenes/compound-review.json"
        ))
        .unwrap();
        let mut property = input.compounds[0].clone();
        for yaw in [0.0, 0.71, core::f32::consts::FRAC_PI_2] {
            for hinge in [PropertySide::Left, PropertySide::Right] {
                property.boundary.gate.orientation =
                    BuildingOrientation::from_radians(yaw).unwrap();
                property.boundary.gate.hinge = hinge;
                let reservation = FurnitureFootprint::gate_sweep(&property).unwrap();
                let door = property.boundary.gate.door(property.id).unwrap();
                for step in 0..=90 {
                    let rotation = Quat::from_rotation_y(
                        door.open_angle_radians.radians() * step as f32 / 90.0,
                    );
                    for corner in [
                        Vec2::new(-1.0, -1.0),
                        Vec2::new(-1.0, 1.0),
                        Vec2::new(1.0, -1.0),
                        Vec2::ONE,
                    ] {
                        let closed = door.closed_centre.metres()
                            + Quat::from_rotation_y(door.closed_yaw_radians.radians())
                                * Vec3::new(
                                    corner.x * door.size_metres.metres().x * 0.5,
                                    0.0,
                                    corner.y * door.size_metres.metres().z * 0.5,
                                );
                        let point = door.hinge_centre.metres()
                            + rotation * (closed - door.hinge_centre.metres());
                        assert!(
                            reservation.contains(Vec2::new(point.x, point.z)),
                            "yaw={yaw}, hinge={hinge:?}, step={step}"
                        );
                    }
                }
            }
        }
    }
}
