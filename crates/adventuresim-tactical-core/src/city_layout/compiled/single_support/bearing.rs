//! Bind a single property's exact floor contact to its selected scene member.
use super::*;
pub(super) struct SingleBearingProjection {
    pub bounds: CityPlotBounds,
    pub region: crate::city_layout::grounding::FloorRegion,
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
                binding: super::super::support::SupportBindingRole::GroundBearing,
            })?;
        let bounds = CityPlotBounds::new(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                placement.centre_metres.metres()
                    + placement.orientation.local_to_world(
                        contact.centre()?.metres().xz()
                            - recipe.collision.bounds.centre()?.metres().xz(),
                    ),
            )?,
            adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                contact.plan_half_extents()?.metres() * 2.0,
            )?,
            placement.orientation,
        )?;
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
                binding: super::super::support::SupportBindingRole::GroundBearing,
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
        .map_err(|cause| invalid(cause.into()))?;
        let region = crate::city_layout::grounding::FloorRegion::from_architectural(
            footprint.polygon(),
            projection,
        )
        .map_err(invalid)?;
        Ok(Self { bounds, region })
    }
}
