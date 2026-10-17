use adventuresim_building_generator::furniture::FurnitureKey;
use avian3d::prelude::Collider;
use bevy::math::Quat;

/// The server and production review use the same physical recipe, including
/// open space between table legs and trough sides. Canvas and water are absent.
pub fn furniture_collider(
    key: FurnitureKey,
) -> Result<Collider, adventuresim_building_generator::furniture::FurnitureRecipeError> {
    Ok(Collider::compound(
        key.recipe()?
            .colliders
            .iter()
            .map(|cuboid| {
                (
                    cuboid.centre.metres(),
                    Quat::from_rotation_y(cuboid.yaw_radians.radians())
                        * Quat::from_rotation_x(cuboid.crossfall_radians.radians())
                        * Quat::from_rotation_z(cuboid.longfall_radians.radians()),
                    Collider::cuboid(
                        cuboid.size.metres().x,
                        cuboid.size.metres().y,
                        cuboid.size.metres().z,
                    ),
                )
            })
            .collect(),
    ))
}
