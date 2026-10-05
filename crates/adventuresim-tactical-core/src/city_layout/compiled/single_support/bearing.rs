//! Bind a single property's exact floor contact to its selected scene member.
use super::*;
pub(super) struct SingleBearingProjection {
    pub bounds: CityPlotBounds,
    pub outline: crate::scene_coordinates::ScenePlanPolygon,
}
impl SingleBearingProjection {
    pub(super) fn from_recipe(
        property: &CitySingleProperty,
        placement: &TacticalBuildingPlacement,
        recipe: &recipes::Recipe,
    ) -> Result<Self, CitySupportError> {
        let contact = recipe
            .collision
            .ground_floor_contact_bounds()
            .map_err(|issue| CitySupportError::InvalidBearing {
                property: property.id,
                building: property.building_id,
                issue,
            })?
            .ok_or(CitySupportError::MissingBinding {
                property: property.id,
                building: property.building_id,
                outward: Vec2::ZERO,
            })?;
        let bounds = CityPlotBounds {
            centre_metres: placement.centre_metres
                + placement
                    .orientation
                    .local_to_world(contact.centre().xz() - recipe.collision.bounds.centre().xz()),
            dimensions_metres: contact.plan_half_extents() * 2.0,
            orientation: placement.orientation,
        };
        let footprint = recipe
            .collision
            .ground_floor_footprint()
            .map_err(|issue| CitySupportError::InvalidBearing {
                property: property.id,
                building: property.building_id,
                issue,
            })?
            .ok_or(CitySupportError::MissingBinding {
                property: property.id,
                building: property.building_id,
                outward: Vec2::ZERO,
            })?;
        let invalid = |issue| CitySupportError::InvalidBearing {
            property: property.id,
            building: property.building_id,
            issue,
        };
        let projection = crate::scene_coordinates::ArchitecturalPlanProjection::from_placement(
            placement,
            recipe.collision.bounds,
        )
        .map_err(invalid)?;
        let outline = crate::scene_coordinates::ScenePlanPolygon::from_architectural(
            footprint.polygon(),
            projection,
        )
        .map_err(invalid)?;
        Ok(Self { bounds, outline })
    }
}
