//! Exact occupied buildings for strategic presentation, without tactical terrain.
use super::*;
use adventuresim_tactical_core::scene_input::furniture::FurnitureLayout;
use adventuresim_tactical_core::scene_input::{GeneratedBuilding, GeneratedBuildingRecipes};

impl PreparedStrategicScene {
    pub(super) fn from_products(
        input: &TacticalSceneInput,
        products: &PreparedProducts,
    ) -> PreparationResult<Self> {
        let mut scene = Self {
            digest: input.digest()?,
            buildings: Vec::new(),
            building_recipes: Default::default(),
            furniture: Default::default(),
        };
        scene.building_recipes.sites = products.sites.clone();
        for placement in &products.placements {
            if let Some(primary) = input
                .buildings
                .iter()
                .find(|primary| primary.id == placement.id)
            {
                if primary != placement {
                    return Err(PreparationError::SceneBindingsMismatch);
                }
            } else {
                let distant = input
                    .distant_buildings
                    .iter()
                    .find(|distant| distant.id == placement.id)
                    .ok_or(PreparationError::MissingDistantPlacement {
                        building: placement.id,
                    })?;
                if *placement != (*distant).into() {
                    return Err(PreparationError::PromotedPlacementMismatch {
                        building: placement.id,
                    });
                }
            }
            let venue = products
                .venues
                .iter()
                .find(|venue| venue.recipe.program == placement.program)
                .ok_or(PreparationError::NotPrepared {
                    product: ProductKind::Venue,
                })?;
            scene.building_recipes.insert(venue.recipe.clone());
            let building = GeneratedBuilding {
                placement: placement.clone(),
                plan: venue.recipe.plan.clone(),
                collision: venue.recipe.collision.clone(),
            };
            scene
                .furniture
                .install_interior(&building, venue.interior.clone())?;
            scene.buildings.push(building);
        }
        if input.buildings.iter().any(|primary| {
            !scene
                .buildings
                .iter()
                .any(|building| building.placement == *primary)
        }) {
            return Err(PreparationError::SceneBindingsMismatch);
        }
        Ok(scene)
    }
}
