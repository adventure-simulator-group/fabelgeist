//! Transforms generator-local furnishing and retains its circulation proof.
use adventuresim_building_generator::interior::{
    InteriorLayout, InteriorPlacement, furnish, furniture_floor_height,
};
use fabelgeist_determinism::mix64;
use serde::Serialize;

use super::*;

const INTERIOR_INSTANCE_DOMAIN: u64 = 0x696e_7465_7269_6f72;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InteriorBuildingLayout {
    pub building_id: u64,
    pub layout: InteriorLayout,
}

pub(super) fn append(
    furniture: &mut FurnitureLayout,
    buildings: &[GeneratedBuilding],
) -> Result<(), super::super::SceneInputError> {
    for building in buildings {
        let layout = furnish(&building.plan, &building.placement.program).map_err(|error| {
            super::super::SceneInputError::Validation(format!(
                "building {} interior: {error}",
                building.placement.id,
            ))
        })?;
        for (index, placement) in layout.placements.iter().enumerate() {
            furniture
                .instances
                .push(instance(building, placement, index));
        }
        furniture.interiors.push(InteriorBuildingLayout {
            building_id: building.placement.id,
            layout,
        });
    }
    Ok(())
}

fn instance(
    building: &GeneratedBuilding,
    placement: &InteriorPlacement,
    index: usize,
) -> GeneratedFurniture {
    let origin = building.collision.bounds.centre();
    let position = building.placement.centre_metres
        + building
            .placement
            .orientation
            .local_to_world(placement.centre_metres - Vec2::new(origin.x, origin.z));
    let height = building.pad_elevation_metres + furniture_floor_height(&building.plan, placement)
        - building.collision.bounds.min.y;
    GeneratedFurniture {
        scene: SceneFurniture {
            id: FurnitureInstanceId(mix64(
                mix64(building.placement.id ^ INTERIOR_INSTANCE_DOMAIN) ^ index as u64,
            )),
            key: placement.key,
            location: FurnitureLocation::Interior {
                building_id: building.placement.id,
                room_id: placement.room_id,
                storey: placement.storey,
            },
        },
        position_metres: Vec3::new(position.x, height, position.y),
        orientation: BuildingOrientation::from_radians(
            building.placement.orientation.yaw_radians() + placement.yaw_radians(),
        )
        .expect("generated cardinal furniture facing is finite"),
    }
}
