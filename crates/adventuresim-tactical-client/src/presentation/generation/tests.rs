use super::*;
use adventuresim_building_generator::BuildingArchetype;
static TEST_PRODUCTS: Mutex<()> = Mutex::new(());

#[test]
fn landscape_workers_preserve_grass_and_residency_is_configuration_specific() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let input_json = include_str!("../../../../../assets/tactical-scenes/sparse-woodland.json");
    let input: TacticalSceneInput = serde_json::from_str(input_json).unwrap();
    let graphics = include_str!("../../../../../assets/config/tactical-graphics.yaml");
    for job in jobs(input_json).unwrap() {
        receive(&job, &generate(&job, &dependencies(&job).unwrap()).unwrap()).unwrap();
    }
    let requests = landscape::jobs(input_json, graphics).unwrap();
    assert_eq!(requests.len(), 2);
    for job in requests {
        receive(&job, &generate(&job, &dependencies(&job).unwrap()).unwrap()).unwrap();
    }
    let digest = input.digest().unwrap();
    let grass = landscape::grass(&digest).unwrap();
    let expected_scene = input.generate().unwrap();
    let expected = crate::presentation::vista::grass::PreparedGrass::new(
        &input,
        &expected_scene.terrain,
        &expected_scene.ground,
        &crate::presentation::config::TacticalGraphicsConfig::parse(graphics).unwrap(),
    );
    for (actual, expected) in [
        (grass.playable.to_batches(), expected.playable.to_batches()),
        (grass.vista.to_batches(), expected.vista.to_batches()),
    ] {
        for (a, b) in actual.iter().flatten().zip(expected.iter().flatten()) {
            assert_eq!(
                bytemuck::cast_slice::<_, u8>(a),
                bytemuck::cast_slice::<_, u8>(b)
            );
        }
    }
    assert!(landscape::jobs(input_json, graphics).unwrap().is_empty());
    let changed = graphics.replace("density_scale: 1.0", "density_scale: 0.5");
    assert_ne!(graphics, changed);
    assert_eq!(landscape::jobs(input_json, &changed).unwrap().len(), 2);
    assert!(
        landscape::grass(&digest).is_none(),
        "old-quality placements must not be used"
    );
    clear_residency();
    assert!(landscape::ground(&digest).is_none());
}

#[test]
fn massive_city_workers_prepare_only_shared_exteriors() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(input).unwrap();
    assert!(
        requests.len() < 200,
        "the 40,000-person fixture must share facade recipes"
    );
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
    assert!(take_scene(&input).is_ok());
    clear_residency();
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
    let original: TacticalSceneInput = serde_json::from_str(input).unwrap();
    let mut placement = adventuresim_tactical_core::scene_input::TacticalBuildingPlacement::from(
        original.distant_buildings[0],
    );
    placement.centre_metres = bevy::math::Vec2::ZERO;
    let first = grounded_test_building(placement.clone(), 1.5);
    let grounded = grounded_test_building(placement, 5.0);
    assert_ne!(grounded.digest().unwrap(), first.digest().unwrap());
    assert!(
        (grounded.buildings[0].base_elevation_metres
            - first.buildings[0].base_elevation_metres
            - 3.5)
            .abs()
            < 0.001
    );
    assert_eq!(grounded.buildings[0].program, first.buildings[0].program);
    assert_eq!(
        jobs(&serde_json::to_string(&grounded).unwrap())
            .unwrap()
            .len(),
        1,
        "valid new floor/source bindings invalidate the scene without regenerating resident facades"
    );
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
    let expected =
        adventuresim_building_generator::compile_static_building_detail(&recipe.plan).unwrap();
    let job = serde_json::to_string(&GenerationJob::Venue(Box::new(program.clone()))).unwrap();
    let bytes = generate(&job, &dependencies(&job).unwrap()).unwrap();
    receive(&job, &bytes).unwrap();
    let geometry = take_venue_geometry(&program).unwrap();
    assert_eq!(geometry.detail.len(), expected.meshes.len());
    for (prepared, expected) in geometry.detail.into_iter().zip(&expected.meshes) {
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
            base_elevation_metres: 2.0,
            id: 1,
            program: BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, 47),
            centre_metres: bevy::math::Vec2::ZERO,
            orientation: adventuresim_tactical_core::scene_input::BuildingOrientation::IDENTITY,
        },
    );
    let input = grounded_test_building(input.buildings.remove(0), 2.0);
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

#[test]
#[ignore = "records cold preparation and warm recipe residency to GENERATION_BENCHMARK_OUTPUT"]
fn independent_city_generation_benchmark() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let started = std::time::Instant::now();
    let mut requests = venue_jobs(input, r#"{"places":[],"people":[]}"#).unwrap();
    let mut scene_and_facades = jobs(input).unwrap();
    let scene = scene_and_facades.remove(0);
    requests.extend(scene_and_facades);
    requests.push(scene);
    let scheduling_seconds = started.elapsed().as_secs_f64();
    let mut bytes = 0_usize;
    let mut recipe_seconds = 0.0;
    let mut receive_seconds = 0.0;
    for job in &requests {
        let started = std::time::Instant::now();
        let product = generate(job, &dependencies(job).unwrap()).unwrap();
        recipe_seconds += started.elapsed().as_secs_f64();
        bytes += product.len();
        let started = std::time::Instant::now();
        receive(job, &product).unwrap();
        receive_seconds += started.elapsed().as_secs_f64();
        if let GenerationJob::Building(program) = serde_json::from_str(job).unwrap() {
            retain_facade(&program);
        }
    }
    let started = std::time::Instant::now();
    let warm = jobs(input).unwrap();
    let warm_scheduling_seconds = started.elapsed().as_secs_f64();
    assert!(
        warm.is_empty(),
        "resident scene and facades require no jobs"
    );
    assert!(
        venue_jobs(input, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
    let report = serde_json::json!({
        "fixture": "massive-city", "jobs": requests.len(), "product_bytes": bytes,
        "cold_scheduling_seconds": scheduling_seconds,
        "serial_generation_seconds": recipe_seconds, "receive_seconds": receive_seconds,
        "warm_scheduling_seconds": warm_scheduling_seconds,
        "warm_jobs": warm.len(),
        "execution": "native serial venue, facade and scene preparation",
    });
    let path = std::env::var("GENERATION_BENCHMARK_OUTPUT").expect("benchmark output path");
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{report}");
    clear_residency();
}

/// Explicit authored producer reservation, rather than an unbound occupied draft.
fn grounded_test_building(
    placement: adventuresim_tactical_core::scene_input::TacticalBuildingPlacement,
    source_elevation_metres: f32,
) -> TacticalSceneInput {
    use adventuresim_tactical_core::city_layout::{
        CityPlotBounds, CityPropertyId, CitySceneLayout, CitySingleProperty, CompoundGradingPolicy,
    };
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.playable.heights_metres.fill(source_elevation_metres);
    input.vista.lods.clear();
    input.buildings = vec![placement.clone()];
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        single_properties: vec![CitySingleProperty {
            id: CityPropertyId(placement.id),
            building_id: placement.id,
            plot: CityPlotBounds {
                centre_metres: placement.centre_metres,
                dimensions_metres: bevy::math::Vec2::splat(35.0),
                orientation: placement.orientation,
            },
        }],
        ..Default::default()
    };
    input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap()
}

#[test]
fn owned_terrain_survives_scene_and_landscape_worker_transport() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let source = include_str!("../../../../../assets/tactical-scenes/compound-review.json");
    let input: TacticalSceneInput = serde_json::from_str(source).unwrap();
    for job in venue_jobs(source, r#"{"places":[],"people":[]}"#).unwrap() {
        receive(&job, &generate(&job, &dependencies(&job).unwrap()).unwrap()).unwrap();
    }
    let job = jobs(source).unwrap().remove(0);
    let bytes = generate(&job, &dependencies(&job).unwrap()).unwrap();
    receive(&job, &bytes).unwrap();
    let expected = input.generate_unfurnished(Default::default()).unwrap();
    assert!(
        !expected
            .terrain
            .property_surface()
            .unwrap()
            .foundations
            .is_empty()
    );
    let graphics = include_str!("../../../../../assets/config/tactical-graphics.yaml");
    for request in landscape::jobs(source, graphics).unwrap() {
        let dependencies: Dependencies =
            ciborium::from_reader(dependencies(&request).unwrap().as_slice()).unwrap();
        let actual = match serde_json::from_str::<GenerationJob>(&request).unwrap() {
            GenerationJob::Ground { .. } => dependencies.ground.unwrap().terrain,
            GenerationJob::Grass { .. } => dependencies.grass.unwrap().terrain,
            _ => panic!("landscape request"),
        };
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected.terrain).unwrap()
        );
        let surface = actual.property_surface().unwrap();
        for foundation in &surface.foundations {
            assert_eq!(
                foundation.member_building_ids,
                input
                    .grounding
                    .as_ref()
                    .unwrap()
                    .surfaces()
                    .iter()
                    .find(|p| p.property_id() == foundation.property_id)
                    .unwrap()
                    .member_building_ids()
            );
        }
    }
    let actual = take_scene(&input).unwrap();
    assert_eq!(
        serde_json::to_value(actual.terrain).unwrap(),
        serde_json::to_value(expected.terrain).unwrap()
    );
    clear_residency();
}

#[test]
fn retained_scene_products_skip_decode_and_never_retain_installed_mutations() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    let request = serde_json::to_string(&input).unwrap();
    for job in jobs(&request).unwrap() {
        receive(&job, &generate(&job, &dependencies(&job).unwrap()).unwrap()).unwrap();
    }
    products().begin_generation();
    assert!(jobs(&request).unwrap().is_empty());
    let mut installed = take_scene(&input).unwrap();
    installed.obstacles.clear();
    installed.repairs.removed_corridor_obstacles = u32::MAX;
    let untouched = take_scene(&input).unwrap();
    assert_ne!(untouched.repairs.removed_corridor_obstacles, u32::MAX);
    assert_eq!(untouched.digest, input.digest().unwrap());
    input.seed = input.seed.wrapping_add(1);
    let changed = serde_json::to_string(&input).unwrap();
    assert_eq!(jobs(&changed).unwrap().len(), 1);
    assert!(take_scene(&input).is_err());
    clear_residency();
    assert_eq!(jobs(&request).unwrap().len(), 1);
}

#[test]
fn immutable_scene_retention_evicts_the_least_recent_input_within_its_bound() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    let mut inputs = Vec::new();
    for seed in 1..=4 {
        input.seed = seed;
        let scene = input.generate_unfurnished(Default::default()).unwrap();
        products().retain_scene(scene);
        inputs.push(input.clone());
        if seed == 3 {
            assert!(products().scene_is_prepared(&inputs[0]).unwrap());
        }
    }
    assert_eq!(products().scenes.len(), 3);
    assert!(take_scene(&inputs[0]).is_ok());
    assert!(take_scene(&inputs[1]).is_err());
    assert!(take_scene(&inputs[2]).is_ok());
    assert!(take_scene(&inputs[3]).is_ok());
    clear_residency();
}

#[test]
fn a_changed_grounding_binding_cannot_reuse_a_resident_scene_with_the_same_programme() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency();
    let original: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut placement = adventuresim_tactical_core::scene_input::TacticalBuildingPlacement::from(
        original.distant_buildings[0],
    );
    placement.centre_metres = bevy::math::Vec2::ZERO;
    let first = grounded_test_building(placement.clone(), 1.5);
    let changed = grounded_test_building(placement, 5.0);
    let scene = first.generate_unfurnished(Default::default()).unwrap();
    products().retain_scene(scene);
    assert!(products().scene_is_prepared(&first).unwrap());
    assert!(!products().scene_is_prepared(&changed).unwrap());
    assert!(products().scene_for_installation(&changed).is_err());
    let preserved = products().scene_for_installation(&first).unwrap();
    assert_eq!(preserved.buildings[0].placement, first.buildings[0]);
    assert_eq!(first.buildings[0].program, changed.buildings[0].program);
    assert_ne!(
        first.buildings[0].base_elevation_metres,
        changed.buildings[0].base_elevation_metres
    );
    clear_residency();
}
