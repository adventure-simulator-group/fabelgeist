//! Dynamic door leaves compiled from accepted opening assemblies.

use std::collections::BTreeMap;

use bevy::math::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{
    BuildingPlan, ClosureKind, ClosureState, OpeningAssemblyId, OpeningUse, ResolvedItemId,
    ResolvedSolid,
};

const EXTERIOR_DOOR_OPEN_ANGLE_RADIANS: f32 = 100.0 * core::f32::consts::PI / 180.0;

/// A single operable exterior leaf in building-local coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DoorSpec {
    pub opening: OpeningAssemblyId,
    pub source: ResolvedItemId,
    pub closed_centre: Vec3,
    pub hinge_centre: Vec3,
    /// Collider and render dimensions in leaf-local X/Y/Z axes.
    pub size_metres: Vec3,
    pub closed_yaw_radians: f32,
    pub tangent: Vec2,
    pub outward: Vec2,
    /// Signed relative yaw. Its sign is selected so the leaf always enters the room.
    pub open_angle_radians: f32,
}

impl DoorSpec {
    /// Furthest horizontal leaf corner from the hinge. Rotation preserves this
    /// radius, including a hinge offset from the edge or centre of the leaf.
    pub fn horizontal_sweep_radius_metres(self) -> f32 {
        let rotation = Quat::from_rotation_y(self.closed_yaw_radians);
        let arm = self.closed_centre - self.hinge_centre;
        [-1.0, 1.0]
            .into_iter()
            .flat_map(|x| [-1.0, 1.0].map(|z| (x, z)))
            .map(|(x, z)| {
                let point = arm
                    + rotation
                        * Vec3::new(
                            x * self.size_metres.x * 0.5,
                            0.0,
                            z * self.size_metres.z * 0.5,
                        );
                Vec2::new(point.x, point.z).length()
            })
            .fold(0.0, f32::max)
    }
}

/// Compiles independently simulated leaves for operable exterior doors.
pub fn compile_operable_doors(plan: &BuildingPlan) -> Vec<DoorSpec> {
    let solids = plan
        .resolved_geometry
        .solids
        .iter()
        .map(|solid| (solid.id, solid))
        .collect::<BTreeMap<_, _>>();

    plan.opening_assemblies
        .iter()
        .filter(|opening| {
            opening.use_kind == OpeningUse::Door
                && opening.closure.state == ClosureState::Operable
                && opening.closure.layers.contains(&ClosureKind::DoorLeaf)
                && opening.frame.inside_room.is_some()
                && opening.frame.outside_room.is_none()
        })
        .filter_map(|opening| {
            let solid = opening
                .closure_solids
                .iter()
                .find_map(|id| solids.get(id).copied())?;
            door_from_solid(
                opening.id,
                opening.frame.tangent,
                opening.frame.outward,
                solid,
            )
        })
        .collect()
}

fn door_from_solid(
    opening: OpeningAssemblyId,
    tangent: Vec2,
    outward: Vec2,
    solid: &ResolvedSolid,
) -> Option<DoorSpec> {
    let tangent = tangent.normalize_or_zero();
    let outward = outward.normalize_or_zero();
    if tangent == Vec2::ZERO || outward == Vec2::ZERO {
        return None;
    }

    let width = tangent.x.abs() * solid.size.x + tangent.y.abs() * solid.size.z;
    let thickness = outward.x.abs() * solid.size.x + outward.y.abs() * solid.size.z;
    let hinge_centre = solid.centre - Vec3::new(tangent.x, 0.0, tangent.y) * width * 0.5;
    let positive_swing = Quat::from_rotation_y(0.01) * Vec3::new(tangent.x, 0.0, tangent.y);
    let enters_room = Vec2::new(positive_swing.x, positive_swing.z).dot(-outward) > 0.0;
    Some(DoorSpec {
        opening,
        source: solid.id,
        closed_centre: solid.centre,
        hinge_centre,
        size_metres: Vec3::new(width, solid.size.y, thickness),
        closed_yaw_radians: -tangent.y.atan2(tangent.x),
        tangent,
        outward,
        open_angle_radians: if enters_room {
            EXTERIOR_DOOR_OPEN_ANGLE_RADIANS
        } else {
            -EXTERIOR_DOOR_OPEN_ANGLE_RADIANS
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, generate};

    #[test]
    fn town_house_exterior_doors_compile_as_inward_swinging_cuboids() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        let doors = compile_operable_doors(&plan);

        assert!(!doors.is_empty());
        for door in doors {
            assert!(door.size_metres.cmpgt(Vec3::ZERO).all());
            let closed_arm = Vec3::new(door.tangent.x, 0.0, door.tangent.y);
            let open_arm = Quat::from_rotation_y(door.open_angle_radians) * closed_arm;
            assert!(Vec2::new(open_arm.x, open_arm.z).dot(-door.outward) > 0.0);
        }
    }

    #[test]
    fn sweep_radius_bounds_rotated_leaf_corners_with_offset_hinges() {
        let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
        for mut door in compile_operable_doors(&plan) {
            door.hinge_centre += Vec3::new(0.03, 0.0, -0.02);
            let radius = door.horizontal_sweep_radius_metres();
            let mut furthest = 0.0_f32;
            for angle in [0.0, 0.3, door.open_angle_radians] {
                let pivot = Quat::from_rotation_y(angle);
                let leaf = Quat::from_rotation_y(door.closed_yaw_radians + angle);
                let centre = door.hinge_centre + pivot * (door.closed_centre - door.hinge_centre);
                for x in [-1.0, 1.0] {
                    for z in [-1.0, 1.0] {
                        let point = centre
                            + leaf
                                * Vec3::new(
                                    x * door.size_metres.x * 0.5,
                                    0.0,
                                    z * door.size_metres.z * 0.5,
                                );
                        let distance =
                            Vec2::new(point.x - door.hinge_centre.x, point.z - door.hinge_centre.z)
                                .length();
                        assert!(distance <= radius + 0.000_01);
                        furthest = furthest.max(distance);
                    }
                }
            }
            assert!((radius - furthest).abs() < 0.000_01);
        }
    }
}
