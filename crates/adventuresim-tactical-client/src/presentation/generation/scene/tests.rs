use super::*;
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe;

fn fixture() -> (TacticalSceneInput, GeneratedTacticalScene, PreparedProducts) {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/compound-review.json"
    )))
    .unwrap();
    let scene = input.generate_unfurnished(Default::default()).unwrap();
    assert!(!scene.buildings.is_empty());
    let venues = scene
        .buildings
        .iter()
        .map(|building| {
            let recipe = GeneratedBuildingRecipe {
                program: building.placement.program.clone(),
                plan: building.plan.clone(),
                collision: building.collision.clone(),
            };
            venue::PreparedVenue::generate(recipe.program.clone(), Some(recipe)).unwrap()
        })
        .collect();
    (
        input,
        scene,
        PreparedProducts {
            venues,
            ..Default::default()
        },
    )
}

#[test]
fn scene_transfer_reuses_prepared_recipes_and_restores_the_exact_complete_scene() {
    let (input, scene, products) = fixture();
    let expected = serde_json::to_value(&scene).unwrap();
    let mut full_bytes = Vec::new();
    ciborium::into_writer(&scene, &mut full_bytes).unwrap();
    let product = SceneProduct::from_generated(scene);
    let mut bytes = Vec::new();
    ciborium::into_writer(&product, &mut bytes).unwrap();
    assert!(
        bytes.len() < full_bytes.len(),
        "transfer must omit duplicate construction recipes"
    );
    let decoded: SceneProduct = ciborium::from_reader(bytes.as_slice()).unwrap();
    let restored = decoded.restore(&input, &products).unwrap();
    assert_eq!(serde_json::to_value(restored).unwrap(), expected);
}

#[test]
fn scene_transfer_rejects_missing_recipes_changed_bindings_and_embedded_recipe_copies() {
    let (input, scene, mut products) = fixture();
    let product = || SceneProduct::from_generated(scene.clone());
    assert!(
        product()
            .restore(&input, &PreparedProducts::default())
            .is_err()
    );
    let mut changed = input.clone();
    let [changed_building, ..] = changed.buildings.as_mut_slice() else {
        panic!("fixture has a building");
    };
    changed_building.base_elevation_metres =
        adventuresim_tactical_core::city_layout::grounding::SupportElevation::from_metres(
            changed_building.base_elevation_metres.metres() + 1.0,
        )
        .unwrap();
    assert!(product().restore(&changed, &products).is_err());
    let mut wrong_program = product();
    let [placement, ..] = wrong_program.placements.as_mut_slice() else {
        panic!("fixture has a placement");
    };
    placement.program.seed = placement.program.seed.wrapping_offset(1);
    assert!(wrong_program.restore(&input, &products).is_err());
    let [generated_building, ..] = scene.buildings.as_slice() else {
        panic!("fixture has a generated building");
    };
    let mut duplicate_geometry = product();
    duplicate_geometry
        .scene
        .buildings
        .push(generated_building.clone());
    assert!(duplicate_geometry.restore(&input, &products).is_err());
    let [input_building, ..] = input.buildings.as_slice() else {
        panic!("fixture has an input building");
    };
    let wanted = &input_building.program;
    products
        .venues
        .retain(|venue| venue.recipe.program != *wanted);
    assert!(product().restore(&input, &products).is_err());
}
