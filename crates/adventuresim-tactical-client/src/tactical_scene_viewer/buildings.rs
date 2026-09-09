use adventuresim_building_generator::BuildingCollision;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

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
