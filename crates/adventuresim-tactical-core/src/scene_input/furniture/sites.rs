//! Lightweight frontage geometry shared by tactical and distant building sites.
use super::*;
use crate::scene_input::{SceneInputError, TacticalBuildingPlacement};
use adventuresim_building_generator::{BuildingPlan, OpeningUse};
#[cfg(test)]
mod tests;
#[derive(Clone)]
pub(super) struct FurnitureSite {
    pub placement: TacticalBuildingPlacement,
    pub half_extents: Vec2,
    pub routes: Vec<FurnitureFootprint>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurnitureSiteRecipe {
    half_extents: Vec2,
    routes: Vec<FurnitureFootprint>,
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
            routes.push(reservations::route(
                start,
                start + opening.frame.outward * reservations::DOOR_APPROACH_METRES,
                opening.profile.exterior_width_metres() * 0.5 + reservations::DOOR_SHOULDER_METRES,
            ));
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
                routes.push(FurnitureFootprint {
                    centre_metres: (min + max) * 0.5 - origin,
                    half_extents_metres: (max - min) * 0.5,
                    orientation: BuildingOrientation::IDENTITY,
                });
            }
        }
        Ok(Self {
            half_extents: bounds.plan_half_extents()?.metres(),
            routes,
        })
    }
    fn place(&self, placement: TacticalBuildingPlacement) -> FurnitureSite {
        let routes = self
            .routes
            .iter()
            .map(|r| FurnitureFootprint {
                centre_metres: placement.centre_metres
                    + placement.orientation.local_to_world(r.centre_metres),
                half_extents_metres: r.half_extents_metres,
                orientation: BuildingOrientation::from_radians(
                    placement.orientation.yaw_radians() + r.orientation.yaw_radians(),
                )
                .unwrap(),
            })
            .collect();
        FurnitureSite {
            placement,
            half_extents: self.half_extents,
            routes,
        }
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
                .map(|r| r.place(b.placement.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for distant in &input.distant_buildings {
        let program = distant.occupied_program();
        let recipe = if let Some((_, recipe)) =
            recipes.sites.iter().find(|(key, _)| *key == program)
        {
            recipe.clone()
        } else {
            let generated = recipes.get_or_generate(&program).map_err(|e| {
                SceneInputError::Validation(format!("distant furniture site {}: {e}", distant.id))
            })?;
            let recipe = FurnitureSiteRecipe::new(&generated.plan, generated.collision.bounds)?;
            recipes.sites.push((program.clone(), recipe.clone()));
            recipe
        };
        sites.push(recipe.place((*distant).into()));
    }
    Ok(sites)
}
