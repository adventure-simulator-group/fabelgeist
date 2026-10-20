//! Building-local strategic geometry never contains tactical terrain or actors.
#[cfg(not(target_family = "wasm"))]
use super::generation::PreparationResult;
#[cfg(not(target_family = "wasm"))]
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use adventuresim_tactical_core::scene_input::furniture::FurnitureLayout;
use adventuresim_tactical_core::scene_input::{GeneratedBuilding, GeneratedBuildingRecipes};
#[derive(Clone)]
pub(crate) struct PreparedStrategicScene {
    pub(crate) digest: String,
    pub(crate) buildings: Vec<GeneratedBuilding>,
    pub(crate) building_recipes: GeneratedBuildingRecipes,
    pub(crate) furniture: FurnitureLayout,
}

impl PreparedStrategicScene {
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn generate(input: &TacticalSceneInput) -> PreparationResult<Self> {
        input.validate()?;
        let mut scene = Self {
            digest: input.digest()?,
            buildings: Vec::new(),
            building_recipes: Default::default(),
            furniture: Default::default(),
        };
        for placement in &input.buildings {
            let recipe = scene.building_recipes.get_or_generate(&placement.program)?;
            scene.buildings.push(GeneratedBuilding {
                placement: placement.clone(),
                plan: recipe.plan.clone(),
                collision: recipe.collision.clone(),
            });
        }
        scene.furniture.furnish_interiors(&scene.buildings)?;
        Ok(scene)
    }
}
