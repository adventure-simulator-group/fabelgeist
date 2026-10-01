use super::*;
use adventuresim_building_generator::BuildingArchetype;
static TEST_PRODUCTS: Mutex<()> = Mutex::new(());

#[test]
fn massive_city_workers_prepare_only_shared_exteriors() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(input).unwrap();
    assert!(requests.len() <= 1 + BuildingArchetype::ALL.len() * 3);
    assert!(matches!(
        serde_json::from_str::<GenerationJob>(&requests[0]).unwrap(),
        GenerationJob::Scene(_)
    ));
    assert_eq!(requests, jobs(input).unwrap());
    println!("massive city generation jobs: {}", requests.len());
}

#[test]
fn worker_products_round_trip_geometry_and_reject_wrong_inputs() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let program = BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, u64::MAX);
    let job = serde_json::to_string(&GenerationJob::Building(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job, &dependencies(&job).unwrap()).unwrap();
    receive(&job, &bytes).unwrap();
    let actual = take_facade(&program).unwrap();
    let expected = PreparedFacade::generate(program.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&actual).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert!(take_facade(&program).is_err());
    assert!(receive(&job, b"truncated").is_err());
    let mut changed = program;
    changed.seed = 42;
    let changed = serde_json::to_string(&GenerationJob::Building(Box::new(changed))).unwrap();
    assert!(receive(&changed, &bytes).is_err());
}

#[test]
fn scene_transport_preserves_static_assets_and_full_width_seed() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.seed = u64::MAX;
    let request = serde_json::to_string(&input).unwrap();
    let jobs = jobs(&request).unwrap();
    assert!(jobs[0].contains(&u64::MAX.to_string()));
    let bytes = generate(&jobs[0], &dependencies(&jobs[0]).unwrap()).unwrap();
    receive(&jobs[0], &bytes).unwrap();
    let actual = take_scene(&input).unwrap();
    let expected = input.generate().unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert!(take_scene(&input).is_err());
    assert!(super::jobs("{}").is_err());
}

#[test]
fn retained_facades_skip_disk_jobs_and_clearing_geometry_releases_residency() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(input).unwrap();
    for request in requests.iter().skip(1) {
        let GenerationJob::Building(program) = serde_json::from_str(request).unwrap() else {
            panic!("facade job");
        };
        retain_facade(&program);
    }
    assert_eq!(jobs(input).unwrap().len(), 1);
    clear_residency();
    assert_eq!(jobs(input).unwrap(), requests);
}

#[test]
fn venue_worker_preserves_meshes_tangents_and_interior_layout() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let program = BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, 47);
    let recipe =
        adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe::generate(program.clone())
            .unwrap();
    let expected = adventuresim_building_generator::compile_static_building_detail(&recipe.plan);
    let job = serde_json::to_string(&GenerationJob::Venue(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job, &dependencies(&job).unwrap()).unwrap();
    receive(&job, &bytes).unwrap();
    let geometry = take_venue_geometry(&program).unwrap();
    assert_eq!(geometry.detail.len(), expected.meshes.len());
    for (prepared, expected) in geometry.detail.into_iter().zip(&expected.meshes) {
        let expected =
            super::super::recipe_mesh::recipe_mesh(expected, recipe.collision.bounds.centre());
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
        products().venues[0].interior,
        adventuresim_building_generator::interior::furnish(&recipe.plan, &program).unwrap()
    );
}

#[test]
fn parallel_building_products_preserve_the_complete_tactical_scene() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.buildings.push(
        adventuresim_tactical_core::scene_input::TacticalBuildingPlacement {
            id: 1,
            program: BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, 47),
            centre_metres: bevy::math::Vec2::ZERO,
            orientation: adventuresim_tactical_core::scene_input::BuildingOrientation::IDENTITY,
        },
    );
    let request = serde_json::to_string(&input).unwrap();
    let venues = venue_jobs(&request, r#"{"places":[],"people":[]}"#).unwrap();
    for job in venues.iter().chain(&jobs(&request).unwrap()) {
        receive(job, &generate(job, &dependencies(job).unwrap()).unwrap()).unwrap();
    }
    let actual = take_scene(&input).unwrap();
    let expected = input.generate().unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert!(
        venue_jobs(&request, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
}
