//! Post-selection geometry validation before publishing any physical placement.
use super::*;
mod envelope;
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
            envelopes.push(
                super::super::packing::MeasuredBuildingEnvelope::from_recipe(building, &recipe)?,
            );
        }
        self.pack_properties(context, &envelopes)?;
        for compound in &self.compounds {
            let front = self
                .buildings
                .iter()
                .find(|b| b.id == compound.front_building_id)
                .ok_or(CityCompileError::Packing {
                    property: compound.id,
                    issue: CityPackingIssue::MissingMember {
                        building: compound.front_building_id,
                    },
                })?;
            let rear = self
                .buildings
                .iter()
                .find(|b| b.id == compound.rear_building_id)
                .ok_or(CityCompileError::Packing {
                    property: compound.id,
                    issue: CityPackingIssue::MissingMember {
                        building: compound.rear_building_id,
                    },
                })?;
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
                    .map(|building| building.centre_metres)
                    .ok_or(CityCompileError::Parish {
                        parish: parish.programme.id,
                    })
            };
            for member in [Some(parish.rectory_building_id), parish.school_building_id]
                .into_iter()
                .flatten()
            {
                if centre(parish.church_building_id)?
                    .metres()
                    .distance(centre(member)?.metres())
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
