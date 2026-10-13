use super::*;
use adventuresim_building_generator::BuildingArchetype;
static TEST_PRODUCTS: Mutex<()> = Mutex::new(());

#[test]
#[ignore = "requires REGIONAL_MAP_CITY_INPUT from the imported-settlement producer"]
fn focused_city_workers_keep_canonical_support_without_actor_preparation() {
    use adventuresim_tactical_core::regional_city::RegionalCityInput;

    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let document_json = std::fs::read_to_string(
        std::env::var_os("REGIONAL_MAP_CITY_INPUT").expect("canonical city fixture path"),
    )
    .unwrap();
    let document: RegionalCityInput = serde_json::from_str(&document_json).unwrap();
    let graphics = include_str!("../../../../../assets/config/tactical-graphics.yaml");
    let actor = begin(PresentationOwner::Scene).unwrap();
    assert!(matches!(
        city::jobs(actor, &document_json, graphics),
        Err(PreparationError::PreparationOwner)
    ));
    let preview = begin(PresentationOwner::RegionalMap).unwrap();
    assert!(matches!(
        jobs(preview, &document_json),
        Err(PreparationError::PreparationOwner)
    ));
    let requests = city::jobs(preview, &document_json, graphics).unwrap();
    assert!(requests.iter().any(|job| matches!(
        serde_json::from_str::<GenerationJob>(job).unwrap(),
        GenerationJob::RegionalCity { .. }
    )));
    for job in requests {
        assert!(matches!(
            serde_json::from_str::<GenerationJob>(&job).unwrap(),
            GenerationJob::RegionalCity { .. } | GenerationJob::Building(_)
        ));
        let bytes = generate(&job, &dependencies(preview, &job).unwrap()).unwrap();
        receive(preview, &job, &bytes).unwrap();
    }
    ownership::finish_city(preview, &document).unwrap();
    let installed = activate_city(preview, &document).unwrap();
    let expected = document
        .input()
        .prepare_supported_terrain(&mut Default::default())
        .unwrap();
    assert_eq!(installed.terrain, expected.terrain);
    let expected_landform = document
        .input()
        .generate_unfurnished(Default::default())
        .unwrap()
        .terrain_patch;
    match (&installed.landform, &expected_landform) {
        (PreparedTerrainLandform::Natural, None) => {}
        (PreparedTerrainLandform::Patch(actual), Some(expected)) => assert_eq!(actual, expected),
        _ => panic!("city worker must retain the canonical landform patch"),
    }
    assert_eq!(installed.document, document);
    assert!(document.input().physical_placements().len() > document.input().buildings.len());
    {
        let active = active_products(PresentationOwner::RegionalMap).unwrap();
        assert!(active.scene.is_none());
        assert!(active.venues.is_empty());
    }
    // Actor cancellation cannot discard the installed city's graded support.
    cancel(actor).unwrap();
    let replacement = begin(PresentationOwner::RegionalMap).unwrap();
    assert!(
        city::jobs(replacement, &document_json, graphics)
            .unwrap()
            .iter()
            .all(|job| matches!(
                serde_json::from_str::<GenerationJob>(job).unwrap(),
                GenerationJob::Building(_)
            ))
    );
    cancel(replacement).unwrap();
    let active = active_products(PresentationOwner::RegionalMap).unwrap();
    let retained = active.regional_city.as_ref().unwrap();
    assert!(Arc::ptr_eq(&installed, retained));
    assert!(Arc::ptr_eq(&installed.ground, &retained.ground));
    drop(active);
    assert!(matches!(
        receive(preview, "invalid old job", &[]),
        Err(PreparationError::StalePreparation)
    ));
}

#[test]
fn massive_city_workers_prepare_only_shared_exteriors() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(ticket, input).unwrap();
    assert!(
        requests.len() < 200,
        "the 40,000-person fixture must share facade recipes"
    );
    assert!(matches!(
        serde_json::from_str::<GenerationJob>(&requests[0]).unwrap(),
        GenerationJob::Building(_)
    ));
    assert_eq!(requests, jobs(ticket, input).unwrap());
    println!("massive city generation jobs: {}", requests.len());
}

#[test]
fn worker_products_round_trip_geometry_and_reject_wrong_inputs() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let program = BuildingProgram::fixture(
        BuildingArchetype::FachwerkCottage,
        fabelgeist_determinism::Seed::from_u64(u64::MAX),
    );
    let job = serde_json::to_string(&GenerationJob::Building(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job, &dependencies(ticket, &job).unwrap()).unwrap();
    receive(ticket, &job, &bytes).unwrap();
    let actual = staged_products(ticket).unwrap().facades.pop().unwrap();
    let expected = PreparedFacade::generate(program.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(matches!(
        receive(ticket, &job, b"truncated"),
        Err(PreparationError::ProductDecode(_))
    ));
    let mut changed = program;
    changed.seed = fabelgeist_determinism::Seed::from_u64(42);
    let changed = serde_json::to_string(&GenerationJob::Building(Box::new(changed))).unwrap();
    assert!(matches!(
        receive(ticket, &changed, &bytes),
        Err(PreparationError::ProductMismatch)
    ));
}

#[test]
fn retained_facades_skip_disk_jobs_and_clearing_geometry_releases_residency() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(ticket, input).unwrap();
    for request in requests.iter() {
        let GenerationJob::Building(program) = serde_json::from_str(request).unwrap() else {
            panic!("facade job");
        };
        retain_facade(&program).unwrap();
    }
    assert!(jobs(ticket, input).unwrap().is_empty());
    let original: TacticalSceneInput = serde_json::from_str(input).unwrap();
    let mut placement = adventuresim_tactical_core::scene_input::TacticalBuildingPlacement::from(
        original.distant_buildings[0],
    );
    placement.centre_metres =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            bevy::math::Vec2::ZERO,
        )
        .unwrap();
    let first = grounded_test_building(placement.clone(), 1.5);
    let grounded = grounded_test_building(placement, 5.0);
    assert_ne!(grounded.digest().unwrap(), first.digest().unwrap());
    assert!(
        (grounded.buildings[0].base_elevation_metres.metres()
            - first.buildings[0].base_elevation_metres.metres()
            - 3.5)
            .abs()
            < 0.001
    );
    assert_eq!(grounded.buildings[0].program, first.buildings[0].program);
    assert_eq!(
        jobs(ticket, &serde_json::to_string(&grounded).unwrap())
            .unwrap()
            .len(),
        0,
        "new floor bindings need no resident facade generation"
    );
    clear_residency().unwrap();
    assert!(matches!(
        jobs(ticket, input),
        Err(PreparationError::StalePreparation)
    ));
    let ticket = begin(PresentationOwner::Scene).unwrap();
    assert_eq!(jobs(ticket, input).unwrap(), requests);
}

#[test]
fn venue_worker_preserves_meshes_tangents_and_interior_layout() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let program = BuildingProgram::fixture(
        BuildingArchetype::FachwerkCottage,
        fabelgeist_determinism::Seed::from_u64(47),
    );
    let recipe =
        adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe::generate(program.clone())
            .unwrap();
    let expected =
        adventuresim_building_generator::compile_static_building_detail(&recipe.plan).unwrap();
    let job = serde_json::to_string(&GenerationJob::Venue(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job, &dependencies(ticket, &job).unwrap()).unwrap();
    receive(ticket, &job, &bytes).unwrap();
    let geometry = staged_products(ticket).unwrap().venues[0]
        .geometry
        .clone()
        .unwrap();
    assert_eq!(geometry.detail.len(), expected.meshes.len());
    for (prepared, expected) in geometry.detail.iter().cloned().zip(&expected.meshes) {
        let expected = super::super::recipe_mesh::recipe_mesh(
            expected,
            recipe.collision.bounds.centre().unwrap().metres(),
        );
        let actual = prepared.into_mesh();
        for attribute in [
            bevy::mesh::Mesh::ATTRIBUTE_POSITION,
            bevy::mesh::Mesh::ATTRIBUTE_NORMAL,
            bevy::mesh::Mesh::ATTRIBUTE_UV_0,
            bevy::mesh::Mesh::ATTRIBUTE_TANGENT,
        ] {
            assert_eq!(
                format!("{:?}", actual.attribute(attribute)),
                format!("{:?}", expected.attribute(attribute))
            );
        }
        assert_eq!(
            format!("{:?}", actual.indices()),
            format!("{:?}", expected.indices())
        );
    }
    assert_eq!(
        staged_products(ticket).unwrap().venues[0].interior,
        adventuresim_building_generator::interior::furnish(&recipe.plan, &program).unwrap()
    );
}

#[test]
fn scene_transport_preserves_static_assets_and_full_width_seed() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.seed = u64::MAX.into();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let request = serde_json::to_string(&input).unwrap();
    assert!(jobs(ticket, &request).unwrap().is_empty());
    assert!(
        venue_jobs(ticket, &request, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
    finish(ticket, &input).unwrap();
    let mut wrong = input.clone();
    wrong.seed = wrong.seed.wrapping_offset(1);
    assert!(matches!(
        activate(ticket, &wrong),
        Err(PreparationError::PreparationInputMismatch)
    ));
    let actual = activate(ticket, &input).unwrap();
    assert_eq!(actual.digest, input.digest().unwrap());
    assert!(actual.buildings.is_empty());
    clear_residency().unwrap();
    assert!(matches!(
        activate(ticket, &input),
        Err(PreparationError::StalePreparation)
    ));
    let ticket = begin(PresentationOwner::Scene).unwrap();
    assert!(matches!(jobs(ticket, "{}"), Err(PreparationError::Json(_))));
}

#[test]
fn parallel_building_products_preserve_exact_occupied_geometry_and_floor_bindings() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let original: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let placement = original.buildings[0].clone();
    let input = grounded_test_building(placement, 2.0);
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let request = serde_json::to_string(&input).unwrap();
    let venues = venue_jobs(ticket, &request, r#"{"places":[],"people":[]}"#).unwrap();
    for job in venues.iter().chain(&jobs(ticket, &request).unwrap()) {
        receive(
            ticket,
            job,
            &generate(job, &dependencies(ticket, job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    // Reusing a recipe never authorizes changing the requested physical placement.
    let accepted = staged_products(ticket).unwrap().placements.clone();
    staged_products(ticket).unwrap().placements[0].base_elevation_metres =
        adventuresim_tactical_core::city_layout::grounding::SupportElevation::from_metres(9.0)
            .unwrap();
    assert!(matches!(
        finish(ticket, &input),
        Err(PreparationError::SceneBindingsMismatch)
    ));
    staged_products(ticket).unwrap().placements = accepted;
    finish(ticket, &input).unwrap();
    let mut actual = activate(ticket, &input).unwrap();
    let expected = PreparedStrategicScene::generate(&input).unwrap();
    assert_eq!(
        serde_json::to_value(&actual.buildings).unwrap(),
        serde_json::to_value(&expected.buildings).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&actual.furniture).unwrap(),
        serde_json::to_value(&expected.furniture).unwrap()
    );
    actual.buildings.clear();
    assert_eq!(
        active_products(PresentationOwner::Scene)
            .unwrap()
            .generated_scene(&input)
            .unwrap()
            .buildings
            .len(),
        1
    );
    let replacement = begin(PresentationOwner::Scene).unwrap();
    let preview = begin(PresentationOwner::RegionalMap).unwrap();
    assert!(matches!(
        staged_products(ticket),
        Err(PreparationError::StalePreparation)
    ));
    assert!(
        !take_venue_geometry(PresentationOwner::Scene, &input.buildings[0].program)
            .unwrap()
            .detail
            .is_empty()
    );
    assert!(
        venue_jobs(replacement, &request, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
    cancel(preview).unwrap();
    cancel(replacement).unwrap();
    assert_eq!(
        active_products(PresentationOwner::Scene)
            .unwrap()
            .generated_scene(&input)
            .unwrap()
            .digest,
        input.digest().unwrap()
    );
}

#[test]
fn retained_scene_products_skip_generation_and_never_retain_installed_mutations() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let original: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let first = grounded_test_building(original.buildings[0].clone(), 1.5);
    let changed = grounded_test_building(original.buildings[0].clone(), 5.0);
    let first_json = serde_json::to_string(&first).unwrap();
    let initial = begin(PresentationOwner::Scene).unwrap();
    for job in venue_jobs(initial, &first_json, r#"{"places":[],"people":[]}"#).unwrap() {
        receive(
            initial,
            &job,
            &generate(&job, &dependencies(initial, &job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    finish(initial, &first).unwrap();
    activate(initial, &first).unwrap();
    let next = begin(PresentationOwner::Scene).unwrap();
    let changed_json = serde_json::to_string(&changed).unwrap();
    assert!(jobs(next, &changed_json).unwrap().is_empty());
    assert!(
        venue_jobs(next, &changed_json, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
    finish(next, &changed).unwrap();
    // Completing a destination cannot replace the scene still being displayed.
    assert_eq!(
        active_products(PresentationOwner::Scene)
            .unwrap()
            .generated_scene(&first)
            .unwrap()
            .digest,
        first.digest().unwrap()
    );
    assert!(matches!(
        activate(next, &first),
        Err(PreparationError::PreparationInputMismatch)
    ));
    let second = activate(next, &changed).unwrap();
    assert_eq!(second.buildings[0].placement, changed.buildings[0]);
    assert_ne!(second.digest, first.digest().unwrap());
    assert!(matches!(
        activate(initial, &first),
        Err(PreparationError::StalePreparation)
    ));
    let cancelled = begin(PresentationOwner::Scene).unwrap();
    venue_jobs(cancelled, &first_json, r#"{"places":[],"people":[]}"#).unwrap();
    finish(cancelled, &first).unwrap();
    cancel(cancelled).unwrap();
    assert!(matches!(
        activate(cancelled, &first),
        Err(PreparationError::StalePreparation)
    ));
    assert_eq!(
        active_products(PresentationOwner::Scene)
            .unwrap()
            .generated_scene(&changed)
            .unwrap()
            .digest,
        second.digest
    );
}

fn grounded_test_building(
    mut placement: adventuresim_tactical_core::scene_input::TacticalBuildingPlacement,
    source_elevation_metres: f32,
) -> TacticalSceneInput {
    use adventuresim_tactical_core::city_layout::{
        CityPlotBounds, CityPropertyId, CitySceneLayout, CitySingleProperty, CompoundGradingPolicy,
    };
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    placement.centre_metres =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            bevy::math::Vec2::ZERO,
        )
        .unwrap();
    input.playable.heights_metres.fill(source_elevation_metres);
    input.vista.lods.clear();
    input.buildings = vec![placement.clone()];
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        single_properties: vec![CitySingleProperty {
            id: CityPropertyId(placement.id.0),
            building_id: placement.id,
            plot: CityPlotBounds::new(
                adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                    placement.centre_metres.metres(),
                )
                .unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    bevy::math::Vec2::splat(35.0),
                )
                .unwrap(),
                placement.orientation,
            )
            .unwrap(),
        }],
        ..Default::default()
    };
    input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap()
}
