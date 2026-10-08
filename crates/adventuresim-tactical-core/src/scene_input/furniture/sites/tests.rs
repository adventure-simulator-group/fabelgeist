use super::*;
use crate::scene_input::GeneratedBuildingRecipes;
use bevy::math::Vec3Swizzles;

#[test]
fn distant_furniture_reserves_the_occupied_footprint_and_unscaled_doors() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut recipes = GeneratedBuildingRecipes::default();
    let sites = collect(&input, &[], &mut recipes).unwrap();
    let mut programs = Vec::new();
    for (site, distant) in sites.iter().zip(&input.distant_buildings) {
        let program = distant.occupied_program();
        let generated = recipes.get_or_generate(&program).unwrap();
        let bounds = generated.collision.bounds;
        assert_eq!(
            site.half_extents.metres(),
            bounds.plan_half_extents().unwrap().metres()
        );
        assert_eq!(site.placement.program.usage, distant.usage);
        let origin = Vec2::new(
            bounds.centre().unwrap().metres().x,
            bounds.centre().unwrap().metres().z,
        );
        for door in generated.plan.opening_assemblies.iter().filter(|opening| {
            opening.use_kind == OpeningUse::Door && opening.frame.outside_room.is_none()
        }) {
            let approach = distant.centre_metres.metres()
                + distant.orientation.local_to_world(
                    door.frame.origin - origin
                        + door.frame.outward * reservations::DOOR_APPROACH_METRES * 0.5,
                );
            assert!(
                site.routes.iter().any(|route| route.contains(
                    crate::scene_coordinates::ScenePlanPoint::try_from(approach).unwrap()
                ))
            );
        }
        if !programs.contains(&program) {
            programs.push(program);
        }
    }
    assert!(programs.len() < input.distant_buildings.len());
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

#[test]
fn carpenter_passage_reserves_the_reported_exterior_approach() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-grounding/carpenter-passage-access.json"
    ))
    .unwrap();
    let placement: TacticalBuildingPlacement =
        serde_json::from_value(fixture["placement"].clone()).unwrap();
    let recipe =
        crate::scene_input::GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
    let site = FurnitureSiteRecipe::new(&recipe.plan, recipe.collision.bounds)
        .unwrap()
        .place(placement.clone())
        .unwrap();
    let contact: Vec2 = serde_json::from_value(fixture["blocked_contact_metres"].clone()).unwrap();
    assert_eq!(placement.id, (9).into());
    assert!(
        !site.routes.iter().any(|route| route
            .contains(crate::scene_coordinates::ScenePlanPoint::try_from(contact).unwrap())),
        "fixture must retain the missing recipe-only exterior reservation"
    );
    let surface: crate::city_layout::grounding::PropertySupportSurface =
        serde_json::from_value(fixture["surface"].clone()).unwrap();
    surface.validate_encoded().unwrap();
    assert_eq!(surface.member_building_ids(), [placement.id]);
    let routes: Vec<_> = surface
        .doorway_approaches()
        .iter()
        .copied()
        .map(|region| FurnitureFootprint::accepted_approach(region).unwrap())
        .collect();
    assert!(routes.iter().any(|route| {
        route.contains(crate::scene_coordinates::ScenePlanPoint::try_from(contact).unwrap())
    }));
    for entrance in adventuresim_building_generator::compile_ground_entrances(&recipe.plan).unwrap()
    {
        let start = placement.centre_metres.metres()
            + placement.orientation.local_to_world(
                entrance.threshold_metres.metres()
                    - recipe.collision.bounds.centre().unwrap().metres().xz(),
            );
        let outside = start
            + placement
                .orientation
                .local_to_world(entrance.outward.vector())
                * reservations::DOOR_APPROACH_METRES
                * 0.5;
        assert!(routes.iter().any(|route| {
            route.contains(crate::scene_coordinates::ScenePlanPoint::try_from(outside).unwrap())
        }));
    }
    assert!(!routes.iter().any(|route| route.contains(
        crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::splat(10000.0)).unwrap()
    )));
    assert_eq!(site.placement, placement);
}
