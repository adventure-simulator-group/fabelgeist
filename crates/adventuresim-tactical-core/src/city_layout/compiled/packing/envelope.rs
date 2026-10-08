//! Compile one exact occupied recipe into its measured packing envelope.
use super::*;
impl super::super::super::packing::MeasuredBuildingEnvelope {
    pub(super) fn from_recipe(
        building: &TacticalBuildingPlacement,
        recipe: &recipes::Recipe,
    ) -> CityCompileResult<Self> {
        let half = recipe.collision.bounds.plan_half_extents()?.metres();
        let min = recipe.render_min_metres().min(-half);
        let max = recipe.render_max_metres().max(half);
        let footprint = recipe
            .collision
            .ground_floor_footprint()
            .map_err(|issue| CityCompileError::Packing {
                property: CityPropertyId(building.id.0),
                issue: CityPackingIssue::InvalidBearing {
                    building: building.id,
                    issue,
                },
            })?
            .ok_or(CityCompileError::Packing {
                property: CityPropertyId(building.id.0),
                issue: CityPackingIssue::MissingBearing {
                    building: building.id,
                },
            })?;
        let invalid = |issue| CityCompileError::Packing {
            property: CityPropertyId(building.id.0),
            issue: CityPackingIssue::InvalidBearing {
                building: building.id,
                issue,
            },
        };
        let projection = crate::scene_coordinates::ArchitecturalPlanProjection::from_placement(
            building,
            recipe.collision.bounds,
        )
        .map_err(|cause| invalid(cause.into()))?;
        let bearing_outline = crate::scene_coordinates::ScenePlanPolygon::from_architectural(
            footprint.polygon(),
            projection,
        )
        .map_err(invalid)?;
        Ok(Self {
            building: building.id,
            bearing_outline,
            body: CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(
                    building.centre_metres.metres()
                        + building.orientation.local_to_world((min + max) * 0.5),
                )?,
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    max - min,
                )?,
                building.orientation,
            )?,
        })
    }
}
