//! Select detailed capture members only after immutable support reconstruction.
use super::Contract;
use adventuresim_tactical_core::scene_input::{
    GeneratedBoundary, GeneratedBuilding, GeneratedBuildingRecipe, GeneratedTacticalScene,
    SceneGarden, SceneInputError, TacticalSceneInput,
};
use std::collections::BTreeSet;

impl Contract {
    pub(in crate::tactical_scene_viewer) fn generate(
        &self,
        input: &TacticalSceneInput,
    ) -> Result<GeneratedTacticalScene, SceneInputError> {
        let mut generated = input.generate()?;
        let Some(document) = &self.0 else {
            return Ok(generated);
        };
        let requested: BTreeSet<_> = document.playable_building_ids.iter().copied().collect();
        assert_eq!(requested.len(), document.playable_building_ids.len());
        let mut promoted = Vec::new();
        for id in requested {
            if generated
                .buildings
                .iter()
                .any(|building| building.placement.id == id)
            {
                continue;
            }
            let placement: adventuresim_tactical_core::scene_input::TacticalBuildingPlacement =
                input
                    .distant_buildings
                    .iter()
                    .find(|building| building.id == id)
                    .copied()
                    .expect("explicit capture member must exist")
                    .into();
            let recipe = generated
                .building_recipes
                .take(&placement.program)
                .map(Ok)
                .unwrap_or_else(|| GeneratedBuildingRecipe::generate(placement.program.clone()))
                .map_err(|error| {
                    SceneInputError::Validation(format!(
                        "capture building {id} program is invalid: {error}"
                    ))
                })?;
            promoted.push(GeneratedBuilding {
                placement,
                plan: recipe.plan,
                collision: recipe.collision,
            });
        }
        let owners: BTreeSet<_> = promoted.iter().map(|b| b.placement.id).collect();
        generated.boundaries.extend(
            input
                .compounds
                .iter()
                .filter(|p| owners.contains(&p.front_building_id))
                .map(|p| GeneratedBoundary::project(p, &generated.terrain))
                .collect::<Result<Vec<_>, _>>()?,
        );
        generated.gardens.extend(
            input
                .gardens
                .iter()
                .filter(|p| owners.contains(&p.front_building_id))
                .map(|p| SceneGarden::project(p.clone(), &generated.terrain))
                .collect::<Result<Vec<_>, _>>()?,
        );
        generated.furniture.furnish_interiors(&promoted)?;
        generated.buildings.extend(promoted);
        Ok(generated)
    }
}

#[cfg(test)]
mod tests;
