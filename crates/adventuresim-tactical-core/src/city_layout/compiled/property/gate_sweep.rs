use super::*;
use adventuresim_building_generator::{CollisionCuboid, DoorSpec};
use bevy::math::{Quat, Vec3};

const SWEEP_STEP_RADIANS: f32 = core::f32::consts::PI / 180.0;

/// Each enlarged sample contains the complete angular interval around it.
/// Candidate filtering uses bounding spheres; the final test uses oriented solids.
pub(super) fn clear(door: DoorSpec, solids: &[CollisionCuboid]) -> bool {
    let radius = door.size_metres.x.hypot(door.size_metres.z * 0.5);
    let candidates = solids
        .iter()
        .filter(|solid| {
            solid.centre.distance(door.hinge_centre)
                <= radius.hypot(door.size_metres.y * 0.5) + solid.size.length() * 0.5
        })
        .collect::<Vec<_>>();
    let steps = (door.open_angle_radians.abs() / SWEEP_STEP_RADIANS).ceil() as usize;
    let step = door.open_angle_radians / steps as f32;
    let padding = radius * step.abs() * 0.5;
    (0..steps).all(|index| {
        let angle = step * (index as f32 + 0.5);
        let leaf = CollisionCuboid {
            source: door.source,
            centre: door.hinge_centre
                + Quat::from_rotation_y(angle) * (door.closed_centre - door.hinge_centre),
            size: door.size_metres + Vec3::new(padding, 0.0, padding) * 2.0,
            yaw_radians: door.closed_yaw_radians + angle,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        };
        candidates.iter().all(|solid| !leaf.intersects(**solid))
    })
}

pub(super) fn building_solids(
    placement: &TacticalBuildingPlacement,
    recipe: &Recipe,
) -> Vec<CollisionCuboid> {
    let origin = recipe.collision.bounds.centre();
    let rotation = Quat::from_rotation_y(placement.orientation.yaw_radians());
    let translation = Vec3::new(placement.centre_metres.x, 0.0, placement.centre_metres.y);
    recipe
        .collision
        .cuboids
        .iter()
        .map(|solid| CollisionCuboid {
            centre: rotation * (solid.centre - Vec3::new(origin.x, 0.0, origin.z)) + translation,
            yaw_radians: solid.yaw_radians + placement.orientation.yaw_radians(),
            ..*solid
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inward_gate_clears_posts_but_mid_swing_obstacle_is_rejected() {
        for yaw in [0.0, 0.71, core::f32::consts::FRAC_PI_2] {
            let boundary = CityBoundary {
                walls: vec![],
                gate: CityGate {
                    centre_metres: Vec2::new(10.0, -4.0),
                    orientation: BuildingOrientation::from_radians(yaw).unwrap(),
                    width_metres: 1.6,
                    height_metres: 1.8,
                },
            };
            let door = boundary.gate.door(CityPropertyId(1));
            let mut solids = boundary
                .fixed_members()
                .iter()
                .map(|member| CollisionCuboid {
                    source: door.source,
                    centre: member.centre_metres,
                    size: member.size_metres,
                    yaw_radians: member.yaw_radians,
                    crossfall_radians: 0.0,
                    longfall_radians: 0.0,
                })
                .collect::<Vec<_>>();
            assert!(clear(door, &solids));
            solids.push(CollisionCuboid {
                source: door.source,
                centre: door.hinge_centre
                    + Quat::from_rotation_y(door.open_angle_radians * 0.5)
                        * (door.closed_centre - door.hinge_centre),
                size: Vec3::splat(0.01),
                yaw_radians: 0.0,
                crossfall_radians: 0.0,
                longfall_radians: 0.0,
            });
            assert!(!clear(door, &solids));
        }
    }
}
