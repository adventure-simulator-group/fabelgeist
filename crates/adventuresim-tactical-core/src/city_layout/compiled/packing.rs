//! Post-selection geometry validation before publishing any physical placement.
use super::*;
mod seating;

impl CompiledCityLayout {
    pub(super) fn finalize_packing(
        &mut self,
        context: &super::super::packing::CityPackingContext,
    ) -> Result<(), CityCompileError> {
        self.seat_single_bearings()?;
        let mut envelopes = Vec::with_capacity(self.buildings.len());
        for building in &self.buildings {
            let recipe = self.support_recipes.for_program(&building.program)?;
            let half = recipe.collision.bounds.plan_half_extents();
            let min = recipe.render_min.min(-half);
            let max = recipe.render_max.max(half);
            let footprint =
                recipe
                    .collision
                    .ground_floor_footprint()
                    .ok_or(CityCompileError::Packing {
                        property: CityPropertyId(building.id),
                        issue: CityPackingIssue::MissingBearing {
                            building: building.id,
                        },
                    })?;
            let origin = recipe.collision.bounds.centre();
            let bearing_outline = footprint
                .vertices()
                .iter()
                .map(|point| {
                    building.centre_metres
                        + building
                            .orientation
                            .local_to_world(*point - Vec2::new(origin.x, origin.z))
                })
                .collect();
            envelopes.push(super::super::packing::MeasuredBuildingEnvelope {
                building: building.id,
                bearing_outline,
                body: CityPlotBounds {
                    centre_metres: building.centre_metres
                        + building.orientation.local_to_world((min + max) * 0.5),
                    dimensions_metres: max - min,
                    orientation: building.orientation,
                },
            });
        }
        self.pack_properties(context, &envelopes)?;
        for compound in &self.compounds {
            let front = self
                .buildings
                .iter()
                .find(|b| b.id == compound.front_building_id)
                .expect("compiled compound retains its front member");
            let rear = self
                .buildings
                .iter()
                .find(|b| b.id == compound.rear_building_id)
                .expect("compiled compound retains its rear member");
            let front_recipe = self.support_recipes.for_program(&front.program)?;
            let rear_recipe = self.support_recipes.for_program(&rear.program)?;
            property::validate_access(
                compound,
                front,
                &front_recipe,
                rear,
                &rear_recipe,
                &self.streets,
            )?;
        }
        for parish in &self.parishes {
            let centre = |id| {
                self.buildings
                    .iter()
                    .find(|b| b.id == id)
                    .expect("compiled parish retains its physical members")
                    .centre_metres
            };
            for member in [Some(parish.rectory_building_id), parish.school_building_id]
                .into_iter()
                .flatten()
            {
                if centre(parish.church_building_id).distance(centre(member))
                    > CITY_PARISH_PRECINCT_RADIUS_METRES
                {
                    return Err(CityCompileError::Parish {
                        parish: parish.programme.id,
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
