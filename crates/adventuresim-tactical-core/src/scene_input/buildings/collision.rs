//! Static collision shares the architectural placement root with presentation.
use adventuresim_building_generator::BuildingCollision;
use avian3d::prelude::Collider;
use bevy::math::Quat;

/// Compile the accepted fixed building geometry. Operable closures belong to
/// separate tactical entities and are not baked into this immutable collider.
pub fn compile_tactical_building_collider(
    collision: &BuildingCollision,
) -> Result<Collider, adventuresim_building_generator::spatial_geometry::GeometryError> {
    let local_origin = collision.bounds.centre()?;
    Ok(Collider::compound(
        collision
            .cuboids
            .iter()
            .map(|cuboid| {
                let translation = local_origin.displacement_to(cuboid.centre)?.metres();
                let rotation = Quat::from_rotation_y(cuboid.yaw_radians.radians())
                    * Quat::from_rotation_x(cuboid.crossfall_radians.radians())
                    * Quat::from_rotation_z(cuboid.longfall_radians.radians());
                Ok((
                    translation,
                    rotation,
                    Collider::cuboid(cuboid.size.metres().x, cuboid.size.metres().y, cuboid.size.metres().z),
                ))
            })
            .collect::<Result<Vec<_>, adventuresim_building_generator::spatial_geometry::GeometryError>>()?,
    ))
}
