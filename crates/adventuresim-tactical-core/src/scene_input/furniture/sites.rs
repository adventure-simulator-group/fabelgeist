//! Lightweight frontage geometry shared by tactical and distant building sites.
use super::*;
use crate::scene_input::SceneValidationError;
use crate::scene_input::{SceneInputError, TacticalBuildingPlacement};
use adventuresim_building_generator::{BuildingPlan, OpeningUse};
mod local;
#[cfg(test)]
mod tests;
use adventuresim_building_generator::spatial_geometry::PlanExtents;
use local::LocalReservation;
#[derive(Clone)]
pub(super) struct FurnitureSite {
    pub placement: TacticalBuildingPlacement,
    pub half_extents: PlanExtents,
    pub routes: Vec<FurnitureFootprint>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurnitureSiteRecipe {
    half_extents: PlanExtents,
    routes: Vec<LocalReservation>,
}
impl FurnitureSiteRecipe {
    pub fn new(
        plan: &BuildingPlan,
        bounds: adventuresim_building_generator::spatial_geometry::SpatialBounds<
            adventuresim_building_generator::spatial_geometry::Architectural,
        >,
    ) -> Result<Self, SceneInputError> {
        let centre = bounds.centre()?.metres();
        let origin = Vec2::new(centre.x, centre.z);
        let mut routes = Vec::new();
        for opening in plan
            .opening_assemblies
            .iter()
            .filter(|o| o.use_kind == OpeningUse::Door && o.frame.outside_room.is_none())
        {
            let start = opening.frame.origin - origin;
            routes.push(LocalReservation::route(
                start,
                start + opening.frame.outward * reservations::DOOR_APPROACH_METRES,
                opening.profile.exterior_width_metres() * 0.5 + reservations::DOOR_SHOULDER_METRES,
            )?);
        }
        if let Some(workplace) = &plan.workplace {
            for passage in &workplace.passages {
                let min = Vec2::new(
                    passage.bounds.min().metres().x,
                    passage.bounds.min().metres().z,
                );
                let max = Vec2::new(
                    passage.bounds.max().metres().x,
                    passage.bounds.max().metres().z,
                );
                routes.push(LocalReservation::from_metres(
                    (min + max) * 0.5 - origin,
                    (max - min) * 0.5,
                    BuildingOrientation::IDENTITY,
                )?);
            }
        }
        Ok(Self {
            half_extents: bounds.plan_half_extents()?,
            routes,
        })
    }
    fn place(
        &self,
        placement: TacticalBuildingPlacement,
    ) -> Result<FurnitureSite, SceneInputError> {
        let routes = self
            .routes
            .iter()
            .map(|route| route.place(&placement))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FurnitureSite {
            placement,
            half_extents: self.half_extents,
            routes,
        })
    }
}
pub(super) fn collect(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
    recipes: &mut crate::scene_input::GeneratedBuildingRecipes,
) -> Result<Vec<FurnitureSite>, SceneInputError> {
    let mut sites = buildings
        .iter()
        .map(|b| {
            FurnitureSiteRecipe::new(&b.plan, b.collision.bounds)
                .and_then(|r| r.place(b.placement.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for distant in &input.distant_buildings {
        let program = distant.occupied_program();
        let recipe = if let Some(site) = recipes.sites.iter().find(|site| site.program == program) {
            site.recipe.clone()
        } else {
            let generated = recipes.get_or_generate(&program).map_err(|e| {
                SceneInputError::Validation(SceneValidationError::FurnitureSite {
                    building: distant.id,
                    source: e,
                })
            })?;
            let recipe = FurnitureSiteRecipe::new(&generated.plan, generated.collision.bounds)?;
            recipes
                .sites
                .push(crate::scene_input::recipes::ProgramFurnitureSite {
                    program: program.clone(),
                    recipe: recipe.clone(),
                });
            recipe
        };
        sites.push(recipe.place((*distant).into())?);
    }
    Ok(sites)
}
