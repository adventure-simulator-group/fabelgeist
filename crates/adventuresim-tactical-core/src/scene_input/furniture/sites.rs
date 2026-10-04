//! Lightweight frontage geometry shared by tactical and distant building sites.
use super::*;
use crate::scene_input::SceneValidationError;
use crate::scene_input::{SceneInputError, TacticalBuildingPlacement};
use adventuresim_building_generator::{BuildingPlan, CollisionBounds, OpeningUse};
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
    pub fn new(plan: &BuildingPlan, bounds: CollisionBounds) -> Self {
        let centre = bounds.centre();
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
                let min = Vec2::new(passage.min.x, passage.min.z);
                let max = Vec2::new(passage.max.x, passage.max.z);
                routes.push(FurnitureFootprint {
                    centre_metres: (min + max) * 0.5 - origin,
                    half_extents_metres: (max - min) * 0.5,
                    orientation: BuildingOrientation::IDENTITY,
                });
            }
        }
        Self {
            half_extents: bounds.plan_half_extents(),
            routes,
        }
    }
    fn place(&self, placement: TacticalBuildingPlacement, scale: f32) -> FurnitureSite {
        let routes = self
            .routes
            .iter()
            .map(|r| FurnitureFootprint {
                centre_metres: placement.centre_metres
                    + placement
                        .orientation
                        .local_to_world(r.centre_metres * scale),
                half_extents_metres: r.half_extents_metres * scale,
                orientation: BuildingOrientation::from_radians(
                    placement.orientation.yaw_radians() + r.orientation.yaw_radians(),
                )
                .unwrap(),
            })
            .collect();
        FurnitureSite {
            placement,
            half_extents: self.half_extents * scale,
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
            FurnitureSiteRecipe::new(&b.plan, b.collision.bounds).place(b.placement.clone(), 1.0)
        })
        .collect::<Vec<_>>();
    for distant in &input.distant_buildings {
        let program = distant.exterior_program();
        let recipe =
            if let Some((_, recipe)) = recipes.sites.iter().find(|(key, _)| *key == program) {
                recipe.clone()
            } else {
                let generated = recipes.get_or_generate(&program).map_err(|e| {
                    SceneInputError::Validation(SceneValidationError::FurnitureSite {
                        building: distant.id,
                        source: e,
                    })
                })?;
                let recipe = FurnitureSiteRecipe::new(&generated.plan, generated.collision.bounds);
                recipes.sites.push((program.clone(), recipe.clone()));
                recipe
            };
        sites.push(recipe.place(
            TacticalBuildingPlacement {
                id: distant.id,
                // Occupation selects street activity, while the visible prototype
                // owns its physical footprint and door reservations.
                program: distant.occupied_program(),
                centre_metres: distant.centre_metres,
                orientation: distant.orientation,
            },
            distant.exterior_scale(&program),
        ));
    }
    Ok(sites)
}
