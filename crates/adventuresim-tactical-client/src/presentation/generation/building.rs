use adventuresim_building_generator::{
    BuildingKit, BuildingLodLevel, BuildingProgram, LodMesh, TimberInstance, compile_building_lod,
    compile_program_shell,
    signs::{SignMount, SignSite},
};
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(in crate::presentation) struct PreparedFacade {
    pub program: BuildingProgram,
    pub site: adventuresim_tactical_core::scene_input::furniture::FurnitureSiteRecipe,
    pub local_origin: Vec3,
    pub floor_offset_metres: f32,
    pub sign_sites: Vec<(SignMount, SignSite)>,
    pub facade: Vec<LodMesh>,
    pub shell: Vec<LodMesh>,
    pub instances: Vec<TimberInstance>,
}

impl PreparedFacade {
    pub(super) fn generate(program: BuildingProgram) -> Result<Self, String> {
        let recipe =
            GeneratedBuildingRecipe::generate(program.clone()).map_err(|e| e.to_string())?;
        let kit = BuildingKit::new(&recipe.plan);
        let local_origin = recipe.collision.bounds.centre();
        let sign_sites = if program
            .usage
            .and_then(adventuresim_building_generator::signs::shop_trade)
            .is_some()
        {
            super::super::buildings::signs::sites(&recipe.plan)
        } else {
            Vec::new()
        };
        Ok(Self {
            site: adventuresim_tactical_core::scene_input::furniture::FurnitureSiteRecipe::new(
                &recipe.plan,
                recipe.collision.bounds,
            ),
            shell: compile_program_shell(&program)
                .unwrap_or_else(|| compile_building_lod(&recipe.plan, BuildingLodLevel::Shell))
                .meshes,
            facade: kit.facade().meshes,
            instances: kit
                .instances
                .into_iter()
                .filter(|instance| instance.facade)
                .collect(),
            floor_offset_metres: local_origin.y - recipe.collision.bounds.min.y,
            local_origin,
            sign_sites,
            program,
        })
    }
}
