//! Accepted support is bound once by the producer and reconstructed by consumers.
use super::*;
use crate::city_layout::grounding::GeographicSurface;
use crate::city_layout::{CitySceneLayout, CompoundGradingPolicy, SelectedCityGrounding};

impl TacticalSceneInput {
    /// Seat a complete generated layout against this scene's unchanged source.
    /// Selection must succeed for every owner before any placement is published.
    /// This is the producer boundary; loading never guesses a missing plan.
    pub fn ground_generated_city(
        mut self,
        layout: &CitySceneLayout,
        policy: CompoundGradingPolicy,
    ) -> Result<Self, SceneInputError> {
        self.validate_source()?;
        if self.grounding.is_some()
            || self.buildings != layout.playable
            || self.distant_buildings != layout.distant
            || self.compounds != layout.compounds
            || self.gardens != layout.gardens
            || self.streets != layout.streets
            || self.yards != layout.yards
            || self.parishes != layout.parishes
        {
            return invalid(SceneValidationError::ProducerLayout);
        }
        if self.buildings.is_empty() && self.distant_buildings.is_empty() {
            self.validate()?;
            return Ok(self);
        }
        let prepared = self.prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())?;
        let source = GeographicSurface::from_presented_scene(&prepared.terrain, &self.vista)
            .ok_or(SceneInputError::Validation(
                SceneValidationError::GeographicSupport,
            ))?;
        let grounded = SelectedCityGrounding::select(layout, &source, policy)?.compile()?;
        self.grounding = Some(grounded.support_projection()?);
        let (layout, _, _) = grounded.into_parts();
        self.buildings = layout.playable;
        self.distant_buildings = layout.distant;
        // Floors must not change geographic preparation or its asset context.
        // The consumer verifies the source digest before installing support.
        self.validate()?;
        Ok(self)
    }

    pub(in crate::scene_input) fn physical_placements(&self) -> Vec<TacticalBuildingPlacement> {
        self.buildings
            .iter()
            .cloned()
            .chain(
                self.distant_buildings
                    .iter()
                    .copied()
                    .map(TacticalBuildingPlacement::from),
            )
            .collect()
    }

    pub(in crate::scene_input) fn validate_grounding(&self) -> Result<(), SceneInputError> {
        let placements = self.physical_placements();
        match (&self.grounding, placements.is_empty()) {
            (None, true) => Ok(()),
            (Some(projection), false) => {
                projection.validate_bindings(&placements)?;
                projection.validate_compound_bindings(&self.compounds)?;
                Ok(())
            }
            (None, false) => invalid(SceneValidationError::MissingSupport),
            (Some(_), true) => invalid(SceneValidationError::UnexpectedSupport),
        }
    }
}

#[cfg(test)]
mod tests;
