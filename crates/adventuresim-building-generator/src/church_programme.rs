//! Frozen physical church programmes, independent of ecclesiastical status.
use crate::{
    BuildingArchetype, BuildingProgram, ChurchProgram, Direction, GenerationError,
    ServiceBuildingSize,
};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::math::Vec2;

/// Reservation in the basilica's street frame, centred on collision bounds.
/// Includes apse and tower eaves, buttresses and the western entrance approach.
/// Geometry tests enforce this authored envelope against detailed vertices.
pub(crate) const URBAN_BASILICA_PLOT_METRES: Vec2 = Vec2::new(24.0, 48.0);

impl BuildingProgram {
    /// Main public entrance in the physical building's coordinate frame.
    pub fn frontage_direction(&self) -> Direction {
        if self.church_program.is_some() {
            Direction::West
        } else {
            Direction::South
        }
    }

    pub(crate) fn validate_church_program(&self) -> Result<(), GenerationError> {
        let principal = self.archetype == BuildingArchetype::ParishChurch
            && self.usage == Some(BuildingUse::ParishChurch)
            && self.service_size == Some(ServiceBuildingSize::Large);
        let urban = self.archetype == BuildingArchetype::Cathedral || principal;
        if urban != self.church_program.is_some()
            || self.church_program.is_some_and(|programme| {
                programme != ChurchProgram::URBAN_BRICK_BASILICA
                    || self.footprint
                        != crate::Footprint::Rectangle {
                            width: 28,
                            depth: 14,
                        }
            })
        {
            return Err(GenerationError::InvalidChurchProgram);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
