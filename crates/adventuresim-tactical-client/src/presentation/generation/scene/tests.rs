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
    changed.buildings[0].base_elevation_metres += 1.0;
    assert!(product().restore(&changed, &products).is_err());
    let mut wrong_program = product();
    wrong_program.placements[0].program.seed =
        wrong_program.placements[0].program.seed.wrapping_add(1);
    assert!(wrong_program.restore(&input, &products).is_err());
    let mut duplicate_geometry = product();
    duplicate_geometry
        .scene
        .buildings
        .push(scene.buildings[0].clone());
    assert!(duplicate_geometry.restore(&input, &products).is_err());
    let wanted = &input.buildings[0].program;
    products
        .venues
        .retain(|venue| venue.recipe.program != *wanted);
    assert!(product().restore(&input, &products).is_err());
}
