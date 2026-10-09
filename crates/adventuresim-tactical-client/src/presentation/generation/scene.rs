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
    ) -> Result<GeneratedTacticalScene, String> {
        if !self.scene.buildings.is_empty() || self.placements != input.buildings {
            return Err("scene transfer changed its exact occupied building bindings".into());
        }
        self.scene.buildings = self
            .placements
            .into_iter()
            .map(|placement| {
                let venue = products
                    .venues
                    .iter()
                    .find(|venue| venue.recipe.program == placement.program)
                    .ok_or("scene transfer requires its exact prepared occupied recipe")?;
                Ok(GeneratedBuilding {
                    placement,
                    plan: venue.recipe.plan.clone(),
                    collision: venue.recipe.collision.clone(),
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(self.scene)
    }

    pub(super) fn digest(&self) -> &str {
        &self.scene.digest
    }
}

#[cfg(test)]
mod tests;
