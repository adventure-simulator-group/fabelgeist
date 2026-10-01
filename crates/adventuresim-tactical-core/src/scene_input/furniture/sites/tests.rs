use super::*;
use crate::scene_input::GeneratedBuildingRecipes;

#[test]
fn distant_furniture_uses_only_visible_prototypes_and_their_scaled_doors() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut recipes = GeneratedBuildingRecipes::default();
    let sites = collect(&input, &[], &mut recipes).unwrap();
    let mut programs = Vec::new();
    for (site, distant) in sites.iter().zip(&input.distant_buildings) {
        let program = distant.exterior_program();
        let scale = distant.exterior_scale(&program);
        let generated = recipes.get_or_generate(&program).unwrap();
        let bounds = generated.collision.bounds;
        assert_eq!(site.half_extents, bounds.plan_half_extents() * scale);
        assert_eq!(site.placement.program.usage, distant.usage);
        let origin = Vec2::new(bounds.centre().x, bounds.centre().z);
        for door in generated.plan.opening_assemblies.iter().filter(|opening| {
            opening.use_kind == OpeningUse::Door && opening.frame.outside_room.is_none()
        }) {
            let approach = distant.centre_metres
                + distant.orientation.local_to_world(
                    (door.frame.origin - origin
                        + door.frame.outward * reservations::DOOR_APPROACH_METRES * 0.5)
                        * scale,
                );
            assert!(site.routes.iter().any(|route| route.contains(approach)));
        }
        if !programs.contains(&program) {
            programs.push(program);
        }
    }
    assert!(programs.len() <= adventuresim_building_generator::BuildingArchetype::ALL.len() * 3);
    for program in programs {
        assert!(recipes.take(&program).is_some());
    }
    let mut reused = GeneratedBuildingRecipes::default();
    reused.sites = recipes.sites.clone();
    let retained = collect(&input, &[], &mut reused).unwrap();
    for (actual, expected) in retained.iter().zip(&sites) {
        assert_eq!(actual.half_extents, expected.half_extents);
        assert_eq!(actual.routes, expected.routes);
        assert!(reused.take(&actual.placement.program).is_none());
    }
    recipes.sites.clear();
    assert!(
        recipes.is_empty(),
        "furniture generated an occupied distant recipe"
    );
}
