use super::*;
use adventuresim_building_generator::{
    CollisionCuboid, ResolvedItemId, interior::standing_path_clear,
};
use bevy::math::{Quat, Vec3};

pub(in crate::city_layout::compiled) fn validate(
    compound: &CityCompound,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    rear: &TacticalBuildingPlacement,
    rear_recipe: &Recipe,
) -> Result<(), CityCompileError> {
    validate_building_routes(compound, front, front_recipe, rear, rear_recipe)?;
    let mut fixed = compound
        .boundary
        .fixed_members()
        .iter()
        .enumerate()
        .map(|(index, member)| CollisionCuboid {
            source: ResolvedItemId(index as u64),
            centre: member.centre_metres,
            size: member.size_metres,
            yaw_radians: member.yaw_radians,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        })
        .collect::<Vec<_>>();
    for member in &fixed {
        let rotation = Quat::from_rotation_y(member.yaw_radians);
        if [-1.0, 1.0].into_iter().any(|x| {
            [-1.0, 1.0].into_iter().any(|z| {
                let p = member.centre + rotation * (member.size * Vec3::new(x, 0.0, z) * 0.5);
                !compound.plot.contains(Vec2::new(p.x, p.z))
            })
        }) {
            return Err(CityCompileError::Compound {
                property: compound.id,
                issue: CompoundIssue::GeometryOutsidePlot,
            });
        }
    }
    let door = compound.boundary.gate.door(compound.id);
    let mut obstacles = fixed.clone();
    obstacles.extend(super::gate_sweep::building_solids(front, front_recipe));
    obstacles.extend(super::gate_sweep::building_solids(rear, rear_recipe));
    if !super::gate_sweep::clear(door, &obstacles) {
        return Err(CityCompileError::Compound {
            property: compound.id,
            issue: CompoundIssue::GateSweepBlocked,
        });
    }
    let swing = Quat::from_rotation_y(door.open_angle_radians);
    fixed.push(CollisionCuboid {
        source: door.source,
        centre: door.hinge_centre + swing * (door.closed_centre - door.hinge_centre),
        size: door.size_metres,
        yaw_radians: door.closed_yaw_radians + door.open_angle_radians,
        crossfall_radians: 0.0,
        longfall_radians: 0.0,
    });
    let local = |p| {
        compound
            .plot
            .orientation
            .world_to_local(p - compound.plot.centre_metres)
    };
    for member in &mut fixed {
        let centre = local(Vec2::new(member.centre.x, member.centre.z));
        member.centre.x = centre.x;
        member.centre.z = centre.y;
        member.yaw_radians -= compound.plot.orientation.yaw_radians();
    }
    if compound.access.iter().any(|route| {
        !standing_path_clear(
            &fixed,
            local(route.start_metres),
            local(route.end_metres),
            0.0,
        )
    }) {
        return Err(CityCompileError::Compound {
            property: compound.id,
            issue: CompoundIssue::GateBlocksOpenPassage,
        });
    }
    Ok(())
}

fn validate_building_routes(
    compound: &CityCompound,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    rear: &TacticalBuildingPlacement,
    rear_recipe: &Recipe,
) -> Result<(), CityCompileError> {
    for (placement, recipe) in [(front, front_recipe), (rear, rear_recipe)] {
        let origin = recipe.collision.bounds.centre();
        let local = |point| {
            placement
                .orientation
                .world_to_local(point - placement.centre_metres)
                + Vec2::new(origin.x, origin.z)
        };
        if compound.access.iter().any(|route| {
            !standing_path_clear(
                &recipe.collision.cuboids,
                local(route.start_metres),
                local(route.end_metres),
                0.0,
            )
        }) {
            return Err(CityCompileError::Compound {
                property: compound.id,
                issue: CompoundIssue::AccessBlocked {
                    building: placement.id,
                },
            });
        }
    }
    Ok(())
}
