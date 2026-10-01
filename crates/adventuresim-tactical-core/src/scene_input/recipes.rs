//! Transient client/server generation products, never part of a scene document.
use adventuresim_building_generator::{
    BuildingCollision, BuildingPlan, BuildingProgram, GenerationError, compile_building_collision,
    generate,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GeneratedBuildingRecipe {
    pub program: BuildingProgram,
    pub plan: BuildingPlan,
    pub collision: BuildingCollision,
}

impl GeneratedBuildingRecipe {
    pub fn generate(program: BuildingProgram) -> Result<Self, GenerationError> {
        let plan = generate(&program)?;
        let collision = compile_building_collision(&plan);
        Ok(Self {
            program,
            plan,
            collision,
        })
    }
}

/// Shares each exact program's plan and collision across preparation consumers.
/// Render compilation takes ownership so temporary CPU geometry can be released.
#[derive(Debug, Default)]
pub struct GeneratedBuildingRecipes {
    recipes: Vec<GeneratedBuildingRecipe>,
    pub sites: Vec<(BuildingProgram, super::furniture::FurnitureSiteRecipe)>,
}

impl GeneratedBuildingRecipes {
    pub fn insert(&mut self, recipe: GeneratedBuildingRecipe) {
        self.recipes
            .retain(|existing| existing.program != recipe.program);
        self.recipes.push(recipe);
    }
    pub fn get_or_generate(
        &mut self,
        program: &BuildingProgram,
    ) -> Result<&GeneratedBuildingRecipe, GenerationError> {
        let index = match self
            .recipes
            .iter()
            .position(|recipe| recipe.program == *program)
        {
            Some(index) => index,
            None => {
                self.recipes
                    .push(GeneratedBuildingRecipe::generate(program.clone())?);
                self.recipes.len() - 1
            }
        };
        Ok(&self.recipes[index])
    }

    pub fn take(&mut self, program: &BuildingProgram) -> Option<GeneratedBuildingRecipe> {
        let index = self
            .recipes
            .iter()
            .position(|recipe| recipe.program == *program)?;
        Some(self.recipes.swap_remove(index))
    }

    pub fn clear(&mut self) {
        self.recipes.clear();
        self.sites.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty() && self.sites.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::BuildingArchetype;

    #[test]
    fn reuse_preserves_geometry_and_distinguishes_complete_programs() {
        let program = BuildingProgram::fixture(BuildingArchetype::TownHouse, 42);
        let mut recipes = GeneratedBuildingRecipes::default();
        let plan = &recipes.get_or_generate(&program).unwrap().plan as *const BuildingPlan;
        assert_eq!(
            plan,
            &recipes.get_or_generate(&program).unwrap().plan as *const _
        );
        let mut changed = program.clone();
        changed.seed = 47;
        recipes.get_or_generate(&changed).unwrap();
        let moved = recipes.take(&program).unwrap();
        assert!(recipes.take(&program).is_none());
        assert_eq!(recipes.take(&changed).unwrap().program, changed);
        let fresh = GeneratedBuildingRecipe::generate(program).unwrap();
        assert_eq!(
            serde_json::to_value(moved.plan).unwrap(),
            serde_json::to_value(fresh.plan).unwrap()
        );
        assert_eq!(moved.collision.bounds, fresh.collision.bounds);
    }
}
