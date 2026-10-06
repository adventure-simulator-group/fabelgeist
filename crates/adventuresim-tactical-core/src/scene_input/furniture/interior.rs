//! Transforms generator-local furnishing and retains its circulation proof.
use adventuresim_building_generator::interior::{InteriorLayout, InteriorPlacement, furnish};
use fabelgeist_determinism::StreamId;
use serde::{Deserialize, Serialize};

use super::*;
use crate::scene_input::SceneInputResult as Result;

const INTERIOR_INSTANCE_DOMAIN: StreamId = StreamId::new("furniture.interior-identity");

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InteriorBuildingLayout {
    pub building_id: u64,
    pub layout: InteriorLayout,
}

pub(super) fn append(
    furniture: &mut FurnitureLayout,
    buildings: &[GeneratedBuilding],
) -> Result<()> {
    for building in buildings {
        let layout = furnish(&building.plan, &building.placement.program).map_err(|cause| {
            super::super::SceneInputError::Interior {
                building_id: building.placement.id,
                cause,
            }
        })?;
        install(furniture, building, layout)?;
    }
    Ok(())
}

pub(super) fn install(
    furniture: &mut FurnitureLayout,
    building: &GeneratedBuilding,
    layout: InteriorLayout,
) -> Result<()> {
    for placement in &layout.placements {
        furniture.instances.push(instance(building, placement)?);
    }
    furniture.interiors.push(InteriorBuildingLayout {
        building_id: building.placement.id,
        layout,
    });
    Ok(())
}

fn instance(
    building: &GeneratedBuilding,
    placement: &InteriorPlacement,
) -> Result<GeneratedFurniture> {
    let origin = building.collision.bounds.centre()?.metres();
    let position = building.placement.centre_metres
        + building
            .placement
            .orientation
            .local_to_world(placement.centre_metres.metres() - Vec2::new(origin.x, origin.z));
    let height = building.placement.base_elevation_metres
        + placement
            .floor_height(&building.plan)
            .map_err(|cause| super::super::SceneInputError::Interior {
                building_id: building.placement.id,
                cause,
            })?
            .metres();
    Ok(GeneratedFurniture {
        scene: SceneFurniture {
            id: FurnitureInstanceId(
                INTERIOR_INSTANCE_DOMAIN
                    .seed(
                        building.placement.id,
                        &[
                            u64::from(placement.room_id.serialized_ordinal()),
                            placement.storey.index() as u64,
                            u64::from(placement.centre_metres.metres().x.to_bits()),
                            u64::from(placement.centre_metres.metres().y.to_bits()),
                            placement.facing as u64,
                            placement.key.kind() as u64,
                        ],
                    )
                    .to_u64(),
            ),
            key: placement.key,
            location: FurnitureLocation::Interior {
                building_id: building.placement.id,
                room_id: placement.room_id.serialized_ordinal(),
                storey: placement.storey.serialized_ordinal().map_err(|cause| {
                    super::super::SceneInputError::Interior {
                        building_id: building.placement.id,
                        cause: cause.into(),
                    }
                })?,
            },
        },
        position_metres: Vec3::new(position.x, height, position.y),
        orientation: BuildingOrientation::from_radians(
            building.placement.orientation.yaw_radians() + placement.yaw_radians().radians(),
        )
        .ok_or(
            adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection,
        )?,
    })
}
