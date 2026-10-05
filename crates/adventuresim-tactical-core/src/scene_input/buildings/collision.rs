//! Static collision shares the architectural placement root with presentation.
use adventuresim_building_generator::BuildingCollision;
use avian3d::prelude::Collider;
use bevy::math::Quat;

/// Compile the accepted fixed building geometry. Operable closures belong to
/// separate tactical entities and are not baked into this immutable collider.
pub fn compile_tactical_building_collider(collision: &BuildingCollision) -> Collider {
    let local_origin = collision.bounds.centre();
    Collider::compound(
        collision
            .cuboids
            .iter()
            .map(|cuboid| {
                let translation = cuboid.centre - local_origin;
                let rotation = Quat::from_rotation_y(cuboid.yaw_radians)
                    * Quat::from_rotation_x(cuboid.crossfall_radians)
                    * Quat::from_rotation_z(cuboid.longfall_radians);
                (
                    translation,
                    rotation,
                    Collider::cuboid(cuboid.size.x, cuboid.size.y, cuboid.size.z),
                )
            })
            .collect(),
    )
}
