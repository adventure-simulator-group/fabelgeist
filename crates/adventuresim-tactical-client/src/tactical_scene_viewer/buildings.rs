use adventuresim_building_generator::BuildingCollision;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

pub(super) fn spawn_boundaries(commands: &mut Commands, boundaries: Vec<GeneratedBoundary>) {
    for boundary in boundaries {
        let door = boundary
            .scene
            .boundary
            .gate
            .door(boundary.scene.property_id);
        let elevation = Vec3::Y * boundary.elevation_metres;
        let centre = door.closed_centre + elevation;
        commands.spawn((
            SceneDoor {
                building_id: boundary.scene.front_building_id,
                opening_id: door.opening.0,
                size_metres: door.size_metres,
                doorway_centre_metres: centre,
                tangent: Vec3::new(door.tangent.x, 0.0, door.tangent.y),
                outward: Vec3::new(door.outward.x, 0.0, door.outward.y),
            },
            Transform::from_translation(centre)
                .with_rotation(Quat::from_rotation_y(door.closed_yaw_radians)),
            super::building_review::ReviewLeafPose::boundary_gate(door, elevation),
        ));
        commands.spawn((boundary.scene, Transform::from_translation(elevation)));
    }
}

pub(super) fn spawn_tactical_buildings(commands: &mut Commands, buildings: Vec<GeneratedBuilding>) {
    for building in buildings {
        super::building_review::spawn_openings(commands, &building);
        let transform = building_transform(&building);
        let entity = commands.spawn_empty().id();
        commands.queue(move |world: &mut World| {
            super::building_review::insert_authored_sign(world, entity, building.placement.id);
        });
        commands.entity(entity).insert((
            Name::new(format!("Tactical building {}", building.placement.id)),
            SceneBuilding {
                id: building.placement.id,
                program: building.placement.program,
                orientation: building.placement.orientation,
            },
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            tactical_building_collider(&building.collision),
            transform,
        ));
    }
}

fn tactical_building_collider(collision: &BuildingCollision) -> Collider {
    let local_origin = collision.bounds.centre();
    Collider::compound(
        collision
            .cuboids
            .iter()
            .map(|cuboid| {
                let rotation = Quat::from_rotation_y(cuboid.yaw_radians)
                    * Quat::from_rotation_x(cuboid.crossfall_radians)
                    * Quat::from_rotation_z(cuboid.longfall_radians);
                (
                    cuboid.centre - local_origin,
                    rotation,
                    Collider::cuboid(cuboid.size.x, cuboid.size.y, cuboid.size.z),
                )
            })
            .collect(),
    )
}

pub(super) fn building_transform(building: &GeneratedBuilding) -> Transform {
    let origin = building.collision.bounds.centre();
    Transform::from_xyz(
        building.placement.centre_metres.x,
        building.pad_elevation_metres + origin.y - building.collision.bounds.min.y,
        building.placement.centre_metres.y,
    )
    .with_rotation(Quat::from_rotation_y(
        building.placement.orientation.yaw_radians(),
    ))
}
