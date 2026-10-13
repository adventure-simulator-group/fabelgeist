use super::*;
use adventuresim_building_generator::BuildingArchetype;
use fabelgeist_determinism::Seed;
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
        assert!(active.scenes.is_empty());
        assert!(active.venues.is_empty());
        assert!(active.grass.is_empty());
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
fn landscape_workers_preserve_grass_and_residency_is_configuration_specific() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let input_json = include_str!("../../../../../assets/tactical-scenes/sparse-woodland.json");
    let input: TacticalSceneInput = serde_json::from_str(input_json).unwrap();
    let graphics = include_str!("../../../../../assets/config/tactical-graphics.yaml");
    for job in jobs(ticket, input_json).unwrap() {
        receive(
            ticket,
            &job,
            &generate(&job, &dependencies(ticket, &job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    let requests = landscape::jobs(ticket, input_json, graphics).unwrap();
    assert_eq!(requests.len(), 2);
    for job in requests {
        receive(
            ticket,
            &job,
            &generate(&job, &dependencies(ticket, &job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    let digest = input.digest().unwrap();
    finish(ticket, &input).unwrap();
    let mut other_document = input.clone();
    other_document.seed = other_document.seed.wrapping_offset(1);
    assert!(matches!(
        activate(ticket, &other_document),
        Err(PreparationError::PreparationInputMismatch)
    ));
    assert!(matches!(
        finish(begin(PresentationOwner::RegionalMap).unwrap(), &input),
        Err(PreparationError::PreparationOwner)
    ));
    activate(ticket, &input).unwrap();
    let grass = landscape::grass(PresentationOwner::Scene, &digest)
        .unwrap()
        .unwrap();
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
    let ticket = begin(PresentationOwner::Scene).unwrap();
    assert!(
        landscape::jobs(ticket, input_json, graphics)
            .unwrap()
            .is_empty()
    );
    let changed = graphics.replace("density_scale: 1.0", "density_scale: 0.5");
    assert_ne!(graphics, changed);
    assert_eq!(
        landscape::jobs(ticket, input_json, &changed).unwrap().len(),
        2
    );
    assert!(
        landscape::grass(PresentationOwner::Scene, &digest)
            .unwrap()
            .is_some(),
        "Preparing different quality must retain the installed scene until activation"
    );
    clear_residency().unwrap();
    assert!(
        landscape::ground(PresentationOwner::Scene, &digest)
            .unwrap()
            .is_none()
    );
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
        GenerationJob::Scene(_)
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
fn scene_transport_preserves_static_assets_and_full_width_seed() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.seed = u64::MAX.into();
    let request = serde_json::to_string(&input).unwrap();
    let jobs = jobs(ticket, &request).unwrap();
    assert!(jobs[0].contains(&u64::MAX.to_string()));
    let bytes = generate(&jobs[0], &dependencies(ticket, &jobs[0]).unwrap()).unwrap();
    receive(ticket, &jobs[0], &bytes).unwrap();
    let actual = prepared_scene(ticket, &input).unwrap();
    let expected = input.generate().unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert!(prepared_scene(ticket, &input).is_ok());
    clear_residency().unwrap();
    assert!(matches!(
        prepared_scene(ticket, &input),
        Err(PreparationError::StalePreparation)
    ));
    assert!(matches!(
        super::jobs(ticket, "{}"),
        Err(PreparationError::Json(_))
    ));
}

#[test]
fn retained_facades_skip_disk_jobs_and_clearing_geometry_releases_residency() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let requests = jobs(ticket, input).unwrap();
    for request in requests.iter().skip(1) {
        let GenerationJob::Building(program) = serde_json::from_str(request).unwrap() else {
            panic!("facade job");
        };
        retain_facade(&program).unwrap();
    }
    assert_eq!(jobs(ticket, input).unwrap().len(), 1);
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
        1,
        "valid new floor/source bindings invalidate the scene without regenerating resident facades"
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
fn parallel_building_products_preserve_the_complete_tactical_scene() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    input.buildings.push(
        adventuresim_tactical_core::scene_input::TacticalBuildingPlacement {
            base_elevation_metres:
                adventuresim_tactical_core::city_layout::grounding::SupportElevation::from_metres(
                    2.0,
                )
                .unwrap(),
            id: 1.into(),
            program: BuildingProgram::fixture(
                BuildingArchetype::FachwerkCottage,
                fabelgeist_determinism::Seed::from_u64(47),
            ),
            centre_metres: adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                bevy::math::Vec2::ZERO,
            )
            .unwrap(),
            orientation: adventuresim_tactical_core::scene_input::BuildingOrientation::IDENTITY,
        },
    );
    let input = grounded_test_building(input.buildings.remove(0), 2.0);
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
    let actual = prepared_scene(ticket, &input).unwrap();
    let expected = input.generate().unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    assert!(
        venue_jobs(ticket, &request, r#"{"places":[],"people":[]}"#)
            .unwrap()
            .is_empty()
    );
    finish(ticket, &input).unwrap();
    activate(ticket, &input).unwrap();
    let replacement = begin(PresentationOwner::Scene).unwrap();
    let preview = begin(PresentationOwner::RegionalMap).unwrap();
    assert!(matches!(
        staged_products(ticket),
        Err(PreparationError::StalePreparation)
    ));
    let geometry =
        take_venue_geometry(PresentationOwner::Scene, &input.buildings[0].program).unwrap();
    assert!(
        !geometry.detail.is_empty(),
        "Both preparations retain the installed actor's pending geometry"
    );
    cancel(preview).unwrap();
    cancel(replacement).unwrap();
    assert_eq!(
        active_products(PresentationOwner::Scene)
            .unwrap()
            .scene_for_installation(&input)
            .unwrap()
            .digest,
        input.digest().unwrap()
    );
}

#[test]
#[ignore = "records cold preparation and warm recipe residency to GENERATION_BENCHMARK_OUTPUT"]
fn independent_city_generation_benchmark() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let input = include_str!("../../../../../assets/tactical-scenes/massive-city.json");
    let started = std::time::Instant::now();
    let mut requests = venue_jobs(ticket, input, r#"{"places":[],"people":[]}"#).unwrap();
    let mut scene_and_facades = jobs(ticket, input).unwrap();
    let scene = scene_and_facades.remove(0);
    requests.extend(scene_and_facades);
    requests.push(scene);
    let scheduling_seconds = started.elapsed().as_secs_f64();
    let mut bytes = 0_usize;
    let mut recipe_seconds = 0.0;
    let mut receive_seconds = 0.0;
    for job in &requests {
        let started = std::time::Instant::now();
        let product = generate(job, &dependencies(ticket, job).unwrap()).unwrap();
        recipe_seconds += started.elapsed().as_secs_f64();
        bytes += product.len();
        let started = std::time::Instant::now();
        receive(ticket, job, &product).unwrap();
        receive_seconds += started.elapsed().as_secs_f64();
        if let GenerationJob::Building(program) = serde_json::from_str(job).unwrap() {
            retain_facade(&program).unwrap();
        }
    }
    let started = std::time::Instant::now();
    let warm = jobs(ticket, input).unwrap();
    let warm_scheduling_seconds = started.elapsed().as_secs_f64();
    assert!(
        warm.is_empty(),
        "resident scene and facades require no jobs"
    );
    assert!(
        venue_jobs(ticket, input, r#"{"places":[],"people":[]}"#)
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
    clear_residency().unwrap();
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

#[test]
fn owned_terrain_survives_scene_and_landscape_worker_transport() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let source = include_str!("../../../../../assets/tactical-scenes/compound-review.json");
    let input: TacticalSceneInput = serde_json::from_str(source).unwrap();
    for job in venue_jobs(ticket, source, r#"{"places":[],"people":[]}"#).unwrap() {
        receive(
            ticket,
            &job,
            &generate(&job, &dependencies(ticket, &job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    let job = jobs(ticket, source).unwrap().remove(0);
    let bytes = generate(&job, &dependencies(ticket, &job).unwrap()).unwrap();
    receive(ticket, &job, &bytes).unwrap();
    let expected = input.generate_unfurnished(Default::default()).unwrap();
    assert!(
        !expected
            .terrain
            .property_surface()
            .unwrap()
            .foundations()
            .is_empty()
    );
    let graphics = include_str!("../../../../../assets/config/tactical-graphics.yaml");
    for request in landscape::jobs(ticket, source, graphics).unwrap() {
        let dependencies: Dependencies =
            ciborium::from_reader(dependencies(ticket, &request).unwrap().as_slice()).unwrap();
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
        for foundation in surface.foundations() {
            assert_eq!(
                foundation.member_building_ids(),
                input
                    .grounding
                    .as_ref()
                    .unwrap()
                    .surfaces()
                    .iter()
                    .find(|p| p.property_id() == foundation.property_id())
                    .unwrap()
                    .member_building_ids()
            );
        }
    }
    let actual = prepared_scene(ticket, &input).unwrap();
    assert_eq!(
        serde_json::to_value(actual.terrain).unwrap(),
        serde_json::to_value(expected.terrain).unwrap()
    );
    clear_residency().unwrap();
}

#[test]
fn retained_scene_products_skip_decode_and_never_retain_installed_mutations() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    let request = serde_json::to_string(&input).unwrap();
    for job in jobs(ticket, &request).unwrap() {
        receive(
            ticket,
            &job,
            &generate(&job, &dependencies(ticket, &job).unwrap()).unwrap(),
        )
        .unwrap();
    }
    finish(ticket, &input).unwrap();
    activate(ticket, &input).unwrap();
    let previous = ticket;
    let ticket = begin(PresentationOwner::Scene).unwrap();
    assert!(matches!(
        jobs(previous, &request),
        Err(PreparationError::StalePreparation)
    ));
    assert!(jobs(ticket, &request).unwrap().is_empty());
    let mut installed = prepared_scene(ticket, &input).unwrap();
    installed.obstacles.clear();
    installed.repairs.removed_corridor_obstacles = u32::MAX;
    let untouched = prepared_scene(ticket, &input).unwrap();
    assert_ne!(untouched.repairs.removed_corridor_obstacles, u32::MAX);
    assert_eq!(untouched.digest, input.digest().unwrap());
    input.seed = input.seed.wrapping_offset(1);
    let changed = serde_json::to_string(&input).unwrap();
    assert_eq!(jobs(ticket, &changed).unwrap().len(), 1);
    assert!(matches!(
        prepared_scene(ticket, &input),
        Err(PreparationError::NotPrepared {
            product: ProductKind::Scene
        })
    ));
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    assert_eq!(jobs(ticket, &request).unwrap().len(), 1);
}

#[test]
fn immutable_scene_retention_evicts_the_least_recent_input_within_its_bound() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/sparse-woodland.json"
    ))
    .unwrap();
    let mut inputs = Vec::new();
    for seed in (1..=4)
        .into_iter()
        .map(fabelgeist_determinism::Seed::from_u64)
    {
        input.seed = seed;
        let scene = input.generate_unfurnished(Default::default()).unwrap();
        staged_products(ticket).unwrap().retain_scene(scene);
        inputs.push(input.clone());
        if seed == Seed::from_u64(3) {
            assert_eq!(
                staged_products(ticket)
                    .unwrap()
                    .scene_readiness(&inputs[0])
                    .unwrap(),
                SceneReadiness::Prepared
            );
        }
    }
    assert_eq!(staged_products(ticket).unwrap().scenes.len(), 3);
    assert!(prepared_scene(ticket, &inputs[0]).is_ok());
    assert!(prepared_scene(ticket, &inputs[1]).is_err());
    assert!(prepared_scene(ticket, &inputs[2]).is_ok());
    assert!(prepared_scene(ticket, &inputs[3]).is_ok());
    clear_residency().unwrap();
}

#[test]
fn a_changed_grounding_binding_cannot_reuse_a_resident_scene_with_the_same_programme() {
    let _guard = TEST_PRODUCTS.lock().unwrap();
    clear_residency().unwrap();
    let ticket = begin(PresentationOwner::Scene).unwrap();
    let original: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut placement = adventuresim_tactical_core::scene_input::TacticalBuildingPlacement::from(
        original.distant_buildings[0],
    );
    placement.centre_metres =
        adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
            bevy::math::Vec2::ZERO,
        )
        .unwrap();
    let first = grounded_test_building(placement.clone(), 1.5);
    let changed = grounded_test_building(placement, 5.0);
    let scene = first.generate_unfurnished(Default::default()).unwrap();
    staged_products(ticket).unwrap().retain_scene(scene);
    assert_eq!(
        staged_products(ticket)
            .unwrap()
            .scene_readiness(&first)
            .unwrap(),
        SceneReadiness::Prepared
    );
    assert_eq!(
        staged_products(ticket)
            .unwrap()
            .scene_readiness(&changed)
            .unwrap(),
        SceneReadiness::Missing
    );
    assert!(
        staged_products(ticket)
            .unwrap()
            .scene_for_installation(&changed)
            .is_err()
    );
    let preserved = staged_products(ticket)
        .unwrap()
        .scene_for_installation(&first)
        .unwrap();
    assert_eq!(preserved.buildings[0].placement, first.buildings[0]);
    assert_eq!(first.buildings[0].program, changed.buildings[0].program);
    assert_ne!(
        first.buildings[0].base_elevation_metres.metres(),
        changed.buildings[0].base_elevation_metres.metres()
    );
    clear_residency().unwrap();
}

fn prepared_scene(
    ticket: PreparationTicket,
    input: &TacticalSceneInput,
) -> PreparationResult<GeneratedTacticalScene> {
    staged_products(ticket)?.generated_scene(input)
}
