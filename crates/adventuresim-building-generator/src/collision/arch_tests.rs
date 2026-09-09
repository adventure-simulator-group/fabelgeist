use super::*;
use crate::{BuildingArchetype, BuildingProgram, OpeningUse, generate};

#[test]
fn diagonal_arch_collision_preserves_its_local_clear_crown() {
    let mut plan = generate(&BuildingProgram::fixture(
        BuildingArchetype::ParishChurch,
        42,
    ))
    .unwrap();
    let opening = plan
        .opening_assemblies
        .iter()
        .find(|opening| opening.use_kind == OpeningUse::Window)
        .unwrap();
    let solid = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|solid| solid.id == opening.head_solid)
        .unwrap()
        .clone();
    let wall_id = opening.host_wall;
    let wall = plan
        .wall_assemblies
        .iter_mut()
        .find(|wall| wall.id == wall_id)
        .unwrap();
    let rotation = bevy::math::Quat::from_rotation_y(0.7);
    let tangent = rotation * Vec3::X;
    wall.frame.tangent = Vec2::new(tangent.x, tangent.z);
    let mut solid = solid;
    solid.size.x = 0.8;
    solid.size.z = 0.8;
    let sections = collision_parts(&plan, &solid);
    assert!(sections.len() > 1);
    let clear = solid.centre - Vec3::Y * 0.1;
    assert!(!sections.iter().any(|section| {
        let inverse = bevy::math::Quat::from_rotation_y(-section.yaw_radians);
        (inverse * (clear - section.centre))
            .abs()
            .cmplt(section.size * 0.5)
            .all()
    }));
    assert!(
        sections
            .iter()
            .all(|section| section.yaw_radians.abs() > 0.1)
    );
}
