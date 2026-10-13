use super::*;
use crate::scene_coordinates::{ArchitecturalGateDatum, ArchitecturalPlanProjection, GateRelative};
use adventuresim_building_generator::spatial_geometry::{Elevation, Radians};
use adventuresim_building_generator::{CollisionCuboid, CollisionError, DoorSpec, GenerationError};
use bevy::math::{Quat, Vec3};

const SWEEP_STEP_RADIANS: f32 = core::f32::consts::PI / 180.0;

/// Each enlarged sample contains the complete angular interval around it.
/// Candidate filtering uses bounding spheres; the final test uses oriented solids.
pub(super) fn clear(
    door: DoorSpec<GateRelative>,
    solids: &[CollisionCuboid<GateRelative>],
) -> Result<bool, GenerationError> {
    let radius = door.horizontal_sweep_radius_metres()?;
    let candidates = solids
        .iter()
        .filter(|solid| {
            solid.centre.metres().distance(door.hinge_centre.metres())
                <= radius.hypot(door.size_metres.metres().y * 0.5)
                    + solid.size.metres().length() * 0.5
        })
        .collect::<Vec<_>>();
    let steps = (door.open_angle_radians.radians().abs() / SWEEP_STEP_RADIANS).ceil() as usize;
    let step = door.open_angle_radians.radians() / steps as f32;
    let padding = radius * step.abs() * 0.5;
    for index in 0..steps {
        let angle = step * (index as f32 + 0.5);
        let leaf = CollisionCuboid::from_metres(
            door.source,
            door.hinge_centre.metres()
                + Quat::from_rotation_y(angle)
                    * (door.closed_centre.metres() - door.hinge_centre.metres()),
            door.size_metres.metres() + Vec3::new(padding, 0.0, padding) * 2.0,
            door.closed_yaw_radians.radians() + angle,
            0.0,
            0.0,
        )?;
        if candidates.iter().any(|solid| leaf.intersects(**solid)) {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn building_solids(
    placement: &TacticalBuildingPlacement,
    recipe: &Recipe,
) -> Result<Vec<CollisionCuboid<GateRelative>>, GenerationError> {
    let datum = ArchitecturalGateDatum {
        plan: ArchitecturalPlanProjection::from_placement(placement, recipe.collision.bounds)
            .map_err(|_| {
                adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection
            })?,
        floor: Elevation::ZERO,
    };
    recipe
        .collision
        .cuboids
        .iter()
        .map(|solid| {
            let construct = || {
                Ok(CollisionCuboid {
                    source: solid.source,
                    centre: datum.point(solid.centre)?,
                    size: solid.size,
                    yaw_radians: Radians::new(
                        solid.yaw_radians.radians() + placement.orientation.yaw_radians(),
                    )?,
                    crossfall_radians: solid.crossfall_radians,
                    longfall_radians: solid.longfall_radians,
                })
            };
            construct().map_err(|cause| {
                GenerationError::Collision(CollisionError {
                    source_id: solid.source,
                    cause,
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inward_gate_clears_posts_but_mid_swing_obstacle_is_rejected() {
        for (yaw, hinge) in [0.0, 0.71, core::f32::consts::FRAC_PI_2]
            .into_iter()
            .flat_map(|yaw| [PropertySide::Left, PropertySide::Right].map(|hinge| (yaw, hinge)))
        {
            let boundary = CityBoundary {
                walls: vec![],
                gate: CityGate {
                    hinge,
                    centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::new(10.0, -4.0)).unwrap(),
                    orientation: BuildingOrientation::from_radians(yaw).unwrap(),
                    width_metres: adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.6).unwrap(),
                    height_metres: adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(1.8).unwrap(),
                },
            };
            let door = boundary.gate.door(CityPropertyId(1)).unwrap();
            let mut solids = boundary
                .fixed_members()
                .unwrap()
                .into_iter()
                .map(|member| member.packing_cuboid(door.source).unwrap())
                .collect::<Vec<_>>();
            assert!(clear(door, &solids).unwrap());
            solids.push(
                CollisionCuboid::<crate::scene_coordinates::GateRelative>::from_metres(
                    door.source,
                    door.hinge_centre.metres()
                        + Quat::from_rotation_y(door.open_angle_radians.radians() * 0.5)
                            * (door.closed_centre.metres() - door.hinge_centre.metres()),
                    Vec3::splat(0.01),
                    0.0,
                    0.0,
                    0.0,
                )
                .unwrap(),
            );
            assert!(!clear(door, &solids).unwrap());
        }
    }
}
