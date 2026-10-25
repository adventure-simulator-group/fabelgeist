//! Building-local installation of one furniture-local represented solid.
use super::FurnitureLocal;
use crate::{
    CollisionCuboid, CollisionError,
    spatial_geometry::{Architectural, Position, Radians},
};
use bevy::math::Quat;

#[derive(Clone, Copy, Debug)]
pub struct ArchitecturalFurniturePose {
    pub centre: Position<Architectural>,
    pub yaw: Radians,
}
impl ArchitecturalFurniturePose {
    pub fn cuboid(
        self,
        local: CollisionCuboid<FurnitureLocal>,
    ) -> Result<CollisionCuboid<Architectural>, CollisionError> {
        let construct = || {
            Ok(CollisionCuboid {
                source: local.source,
                centre: Position::from_metres(
                    self.centre.metres()
                        + Quat::from_rotation_y(self.yaw.radians()) * local.centre.metres(),
                )?,
                size: local.size,
                yaw_radians: Radians::new(local.yaw_radians.radians() + self.yaw.radians())?,
                crossfall_radians: local.crossfall_radians,
                longfall_radians: local.longfall_radians,
            })
        };
        construct().map_err(|cause| CollisionError {
            source_id: local.source,
            cause,
        })
    }
}
