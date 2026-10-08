//! Scene transfer reuses exact occupied recipes already prepared by venue jobs.
//! Placement identity and support stay in the Scene product; immutable local
//! construction geometry belongs to the shared programme recipe.

use super::*;
use adventuresim_tactical_core::scene_input::{GeneratedBuilding, TacticalBuildingPlacement};

#[derive(Serialize, Deserialize)]
pub(super) struct SceneProduct {
    scene: GeneratedTacticalScene,
    placements: Vec<TacticalBuildingPlacement>,
}

impl SceneProduct {
    pub(super) fn from_generated(mut scene: GeneratedTacticalScene) -> Self {
        let placements = scene
            .buildings
            .drain(..)
            .map(|building| building.placement)
            .collect();
        Self { scene, placements }
    }

    pub(super) fn restore(
        mut self,
        input: &TacticalSceneInput,
        products: &PreparedProducts,
    ) -> PreparationResult<GeneratedTacticalScene> {
        if !self.scene.buildings.is_empty() || self.placements != input.buildings {
            return Err(PreparationError::SceneBindingsMismatch);
        }
        self.scene.buildings = self
            .placements
            .into_iter()
            .map(|placement| {
                let venue = products
                    .venues
                    .iter()
                    .find(|venue| venue.recipe.program == placement.program)
                    .ok_or(PreparationError::NotPrepared {
                        product: ProductKind::Venue,
                    })?;
                Ok(GeneratedBuilding {
                    placement,
                    plan: venue.recipe.plan.clone(),
                    collision: venue.recipe.collision.clone(),
                })
            })
            .collect::<PreparationResult<_>>()?;
        Ok(self.scene)
    }

    pub(super) fn digest(&self) -> &str {
        &self.scene.digest
    }
}

#[cfg(test)]
mod tests;
