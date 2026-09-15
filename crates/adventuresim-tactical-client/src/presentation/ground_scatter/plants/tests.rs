use super::*;
use adventuresim_tactical_core::prelude::SceneEnvironmentFixture;

#[test]
fn autumn_woodland_fixture_contains_ground_fruiting_fungi() {
    let input = adventuresim_tactical_core::scene_input::TacticalSceneInput::load(
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-scenes/fungi-woodland.json"
        )),
    )
    .unwrap();
    let scene = input.generate().unwrap();
    let environment = input.environment_snapshot(scene.digest.clone());
    let seed = stable_text_seed("fungi-woodland")
        ^ stable_text_seed(&environment.scene_digest)
        ^ PLANT_SEED;
    let sites = placements(&scene.terrain, &scene.ground, &environment, seed);
    assert!(!sites.is_empty());
    for species in adventuresim_plant_generator::fungus::FungusSpecies::ALL {
        assert!(
            sites
                .iter()
                .any(|site| site.species == PlantSpecies::Fungus(species)),
            "missing {species:?}"
        );
    }
}

#[test]
fn seasonal_catalog_shares_population_budget_and_preserves_spring_flowers() {
    let terrain = SceneTerrain::from_heightmap(65, 65, 2.0, vec![0.0; 65 * 65]).unwrap();
    let ground = SceneGround::from_samples(
        65,
        65,
        2.0,
        vec![
            GroundSurface {
                substrate: GroundSubstrate::Soil,
                cover: GroundCover::LeafLitter,
                cover_density_bps: 9000,
                cover_height_cm: 4
            };
            65 * 65
        ],
    )
    .unwrap();
    let mut environment =
        SceneEnvironmentFixture::TemperateHills.snapshot("seasonal-botanical-budget");
    environment.canopy_bps = 6500;
    environment.cultivation_bps = 0;
    environment.weather.ground_moisture_bps = 7000;
    environment.absolute_minute = 110 * MINUTES_PER_DAY;
    let spring = placements(&terrain, &ground, &environment, 42);
    assert_eq!(spring.len(), MAX_SPECIMENS);
    assert!(
        spring
            .iter()
            .all(|s| matches!(s.species, PlantSpecies::Flower(_)))
    );
    environment.absolute_minute = 270 * MINUTES_PER_DAY;
    let autumn = placements(&terrain, &ground, &environment, 42);
    assert_eq!(autumn.len(), MAX_SPECIMENS);
    assert!(
        autumn
            .iter()
            .all(|s| matches!(s.species, PlantSpecies::Fungus(_)))
    );
    for species in adventuresim_plant_generator::fungus::FungusSpecies::ALL {
        assert!(
            autumn
                .iter()
                .any(|s| s.species == PlantSpecies::Fungus(species))
        );
    }
}

#[test]
fn spring_woodland_fixture_has_suitable_flower_openings() {
    let input = adventuresim_tactical_core::scene_input::TacticalSceneInput::load(
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/tactical-scenes/flower-woodland-edge.json"
        )),
    )
    .unwrap();
    let scene = input.generate().unwrap();
    let environment = input.environment_snapshot(scene.digest.clone());
    let sites = placements(&scene.terrain, &scene.ground, &environment, 42);
    let litter = scene
        .ground
        .samples()
        .iter()
        .filter(|s| s.cover == GroundCover::LeafLitter)
        .count();
    assert!(
        !sites.is_empty(),
        "litter={litter}; environment={environment:?}; obstacles={}",
        scene.obstacles.len()
    );
}

#[test]
fn placement_is_repeatable_bounded_grounded_and_excludes_roads_water_and_winter() {
    let terrain = SceneTerrain::from_heightmap(65, 65, 2.0, vec![2.0; 65 * 65]).unwrap();
    let make_ground = |substrate| {
        SceneGround::from_samples(
            65,
            65,
            2.0,
            vec![
                GroundSurface {
                    substrate,
                    ..default()
                };
                65 * 65
            ],
        )
        .unwrap()
    };
    let mut environment = SceneEnvironmentFixture::TemperateHills.snapshot("plant-placement");
    environment.absolute_minute = 180 * MINUTES_PER_DAY;
    environment.weather.ground_moisture_bps = 5000;
    let ground = make_ground(GroundSubstrate::Soil);
    let sites = placements(&terrain, &ground, &environment, 42);
    assert!(!sites.is_empty());
    assert!(sites.len() <= MAX_SPECIMENS);
    assert_eq!(sites, placements(&terrain, &ground, &environment, 42));
    assert!(
        sites
            .iter()
            .all(|s| (s.root.y - (2.0 - ROOT_EMBED_METRES)).abs() < 0.0001)
    );
    assert!(sites.iter().any(|s| s.root.x > 0.0));
    assert!(sites.iter().any(|s| s.root.x < 0.0));
    let dense_sward = SceneGround::from_samples(
        65,
        65,
        2.0,
        vec![
            GroundSurface {
                substrate: GroundSubstrate::Soil,
                cover: GroundCover::TallGrass,
                cover_density_bps: 10000,
                cover_height_cm: 82,
            };
            65 * 65
        ],
    )
    .unwrap();
    assert!(
        placements(&terrain, &dense_sward, &environment, 42).is_empty(),
        "dense tall grass has no low-herb openings"
    );
    for substrate in [
        GroundSubstrate::Water,
        GroundSubstrate::Road,
        GroundSubstrate::Stone,
    ] {
        assert!(placements(&terrain, &make_ground(substrate), &environment, 42).is_empty());
    }
    environment.absolute_minute = 10 * MINUTES_PER_DAY;
    assert!(placements(&terrain, &ground, &environment, 42).is_empty());
}

#[test]
fn scene_plugin_spawns_batches_and_is_idempotent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins(PlantPresentationPlugin);
    let mut environment = SceneEnvironmentFixture::TemperateHills.snapshot("plant-ecs");
    environment.absolute_minute = 180 * MINUTES_PER_DAY;
    let entity = app
        .world_mut()
        .spawn((
            SceneId("plant-ecs".into()),
            SceneTerrain::from_heightmap(17, 17, 2.0, vec![0.0; 17 * 17]).unwrap(),
            SceneGround::from_samples(17, 17, 2.0, vec![GroundSurface::default(); 17 * 17])
                .unwrap(),
            environment,
        ))
        .id();
    app.update();
    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Mesh3d>>()
        .iter(app.world())
        .count();
    assert!(count > 0);
    assert!(
        !app.world()
            .get::<PlantCaptureAnchors>(entity)
            .unwrap()
            .0
            .is_empty()
    );
    app.update();
    assert_eq!(
        count,
        app.world_mut()
            .query_filtered::<Entity, With<Mesh3d>>()
            .iter(app.world())
            .count()
    );
}
