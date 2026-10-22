use super::PreparationResult;
use adventuresim_building_generator::{
    BuildingKit, BuildingLodLevel, BuildingProgram, LodMesh, TimberInstance, compile_building_lod,
    compile_program_shell,
    signs::{SignMount, SignSite},
};
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(in crate::presentation) struct PreparedFacade {
    pub program: BuildingProgram,
    pub site: adventuresim_tactical_core::scene_input::furniture::FurnitureSiteRecipe,
    pub local_origin: adventuresim_building_generator::spatial_geometry::Position<
        adventuresim_building_generator::spatial_geometry::Architectural,
    >,

    pub sign_sites: Vec<(SignMount, SignSite)>,
    pub facade: Vec<LodMesh>,
    pub shell: Vec<LodMesh>,
    pub instances: Vec<TimberInstance>,
}

impl PreparedFacade {
    pub(super) fn generate(program: BuildingProgram) -> PreparationResult<Self> {
        let recipe = GeneratedBuildingRecipe::generate(program.clone())?;
        let kit = BuildingKit::new(&recipe.plan)?;
        let local_origin = recipe.collision.bounds.centre()?;
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
            )?,
            shell: match compile_program_shell(&program) {
                Some(shell) => shell,
                None => compile_building_lod(&recipe.plan, BuildingLodLevel::Shell)?,
            }
            .meshes,
            facade: kit.facade()?.meshes,
            instances: kit
                .instances
                .into_iter()
                .filter(|instance| instance.facade)
                .collect(),

            local_origin,
            sign_sites,
            program,
        })
    }
}
