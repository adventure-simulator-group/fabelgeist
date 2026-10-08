use super::*;
use crate::city_layout::compiled::CityCompileResult as Result;
use adventuresim_building_generator::{
    CollisionCuboid, CollisionResult, ResolvedItemId, interior::StandingClearance,
};
use bevy::math::{Quat, Vec3};

pub(in crate::city_layout::compiled) fn validate(
    compound: &CityCompound,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    rear: &TacticalBuildingPlacement,
    rear_recipe: &Recipe,
) -> Result<()> {
    validate_building_routes(compound, front, front_recipe, rear, rear_recipe)?;
    use crate::scene_coordinates::PlotRelative;
    let mut fixed = compound
        .boundary
        .fixed_members()?
        .into_iter()
        .enumerate()
        .map(|(index, member)| member.packing_cuboid(ResolvedItemId(index as u64)))
        .collect::<CollisionResult<Vec<_>>>()?;
    for member in &fixed {
        let rotation = Quat::from_rotation_y(member.yaw_radians.radians());
        if [-1.0, 1.0].into_iter().any(|x| {
            [-1.0, 1.0].into_iter().any(|z| {
                let p = member.centre.metres()
                    + rotation * (member.size.metres() * Vec3::new(x, 0.0, z) * 0.5);
                !compound.plot.contains(Vec2::new(p.x, p.z))
            })
        }) {
            return Err(CityCompileError::Compound {
                property: compound.id,
                issue: CompoundIssue::GeometryOutsidePlot,
            });
        }
    }
    let door = compound.boundary.gate.door(compound.id)?;
    let mut obstacles = fixed.clone();
    obstacles.extend(super::gate_sweep::building_solids(front, front_recipe)?);
    obstacles.extend(super::gate_sweep::building_solids(rear, rear_recipe)?);
    if !super::gate_sweep::clear(door, &obstacles)? {
        return Err(CityCompileError::Compound {
            property: compound.id,
            issue: CompoundIssue::GateSweepBlocked,
        });
    }
    let swing = Quat::from_rotation_y(door.open_angle_radians.radians());
    fixed.push(CollisionCuboid::from_metres(
        door.source,
        door.hinge_centre.metres()
            + swing * (door.closed_centre.metres() - door.hinge_centre.metres()),
        door.size_metres.metres(),
        door.closed_yaw_radians.radians() + door.open_angle_radians.radians(),
        0.0,
        0.0,
    )?);
    let local = |p| {
        compound
            .plot
            .orientation()
            .world_to_local(p - compound.plot.centre_metres())
    };
    let fixed = fixed
        .into_iter()
        .map(|member| {
            let centre = local(Vec2::new(
                member.centre.metres().x,
                member.centre.metres().z,
            ));
            CollisionCuboid::<PlotRelative>::from_metres(
                member.source,
                Vec3::new(centre.x, member.centre.metres().y, centre.y),
                member.size.metres(),
                member.yaw_radians.radians() - compound.plot.orientation().yaw_radians(),
                member.crossfall_radians.radians(),
                member.longfall_radians.radians(),
            )
        })
        .collect::<CollisionResult<Vec<_>>>()?;
    let clearance = StandingClearance::new(
        &fixed,
        adventuresim_building_generator::spatial_geometry::Elevation::from_metres(0.0)?,
    )?;
    for route in &compound.access {
        let start = local(route.start_metres());
        let end = local(route.end_metres());
        if !clearance.is_clear(
            adventuresim_building_generator::spatial_geometry::Position::from_metres(Vec3::new(
                start.x, 0.0, start.y,
            ))?,
            adventuresim_building_generator::spatial_geometry::Position::from_metres(Vec3::new(
                end.x, 0.0, end.y,
            ))?,
        ) {
            return Err(CityCompileError::Compound {
                property: compound.id,
                issue: CompoundIssue::GateBlocksOpenPassage,
            });
        }
    }
    Ok(())
}

fn validate_building_routes(
    compound: &CityCompound,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    rear: &TacticalBuildingPlacement,
    rear_recipe: &Recipe,
) -> Result<()> {
    for (placement, recipe) in [(front, front_recipe), (rear, rear_recipe)] {
        let origin = recipe.collision.bounds.centre()?.metres();
        let local = |point| {
            placement
                .orientation
                .world_to_local(point - placement.centre_metres.metres())
                + Vec2::new(origin.x, origin.z)
        };
        let clearance = StandingClearance::new(
            &recipe.collision.cuboids,
            adventuresim_building_generator::spatial_geometry::Elevation::from_metres(0.0)?,
        )?;
        for route in &compound.access {
            let start = local(route.start_metres());
            let end = local(route.end_metres());
            if !clearance.is_clear(
                adventuresim_building_generator::spatial_geometry::Position::from_metres(
                    Vec3::new(start.x, 0.0, start.y),
                )?,
                adventuresim_building_generator::spatial_geometry::Position::from_metres(
                    Vec3::new(end.x, 0.0, end.y),
                )?,
            ) {
                return Err(CityCompileError::Compound {
                    property: compound.id,
                    issue: CompoundIssue::AccessBlocked {
                        building: placement.id,
                    },
                });
            }
        }
    }
    Ok(())
}
