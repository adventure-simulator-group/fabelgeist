//! Lightweight frontage geometry shared by tactical and distant building sites.
use super::*;
use crate::scene_input::{SceneInputError, TacticalBuildingPlacement};
use adventuresim_building_generator::{
    BuildingPlan, BuildingProgram, CollisionBounds, OpeningUse, compile_building_collision,
};
#[derive(Clone)]
pub(super) struct FurnitureSite {
    pub placement: TacticalBuildingPlacement,
    pub half_extents: Vec2,
    pub routes: Vec<FurnitureFootprint>,
}
#[derive(Clone)]
struct SiteRecipe {
    half_extents: Vec2,
    routes: Vec<FurnitureFootprint>,
}
impl SiteRecipe {
    fn new(plan: &BuildingPlan, bounds: CollisionBounds) -> Self {
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
) -> Result<Vec<FurnitureSite>, SceneInputError> {
    let mut sites = buildings
        .iter()
        .map(|b| SiteRecipe::new(&b.plan, b.collision.bounds).place(b.placement.clone()))
        .collect::<Vec<_>>();
    let mut cache: Vec<(BuildingProgram, SiteRecipe)> = Vec::new();
    for distant in &input.distant_buildings {
        let program = distant.program();
        let recipe = if let Some((_, recipe)) = cache.iter().find(|(key, _)| *key == program) {
            recipe.clone()
        } else {
            let plan = adventuresim_building_generator::generate(&program).map_err(|e| {
                SceneInputError::Validation(format!("distant furniture site {}: {e}", distant.id))
            })?;
            let recipe = SiteRecipe::new(&plan, compile_building_collision(&plan).bounds);
            cache.push((program.clone(), recipe.clone()));
            recipe
        };
        sites.push(recipe.place(TacticalBuildingPlacement {
            id: distant.id,
            program,
            centre_metres: distant.centre_metres,
            orientation: distant.orientation,
        }));
    }
    Ok(sites)
}
