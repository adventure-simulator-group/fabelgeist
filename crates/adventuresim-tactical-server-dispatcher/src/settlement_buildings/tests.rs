use super::*;
use adventuresim_building_generator::BuildingProgram;
use adventuresim_building_generator::generate;

fn economy(population: u32) -> SettlementEconomyProfile {
    use adventuresim_world_schema::*;
    infer_settlement_economy(
        if population >= 5_000 { 4 } else { 1 },
        population,
        3,
        population >= 5_000,
        &InferredIndustryProfile::new(vec![IndustryEvidence::Fallback(
            FallbackIndustry::CroplandGrain,
        )])
        .unwrap(),
    )
    .unwrap()
}

fn settlement(id: &str, population: u32) -> SettlementSceneProfile {
    SettlementSceneProfile {
        id: id.into(),
        population_level: 1,
        population_estimate: population,
        economy: economy(population),
        operators: Vec::new(),
    }
}

fn layout(id: &str, population: u32) -> CitySceneLayout {
    place_settlement_buildings(&settlement(id, population), 50.0).unwrap()
}

fn ordered_centres(
    layout: &CitySceneLayout,
) -> Vec<(
    adventuresim_tactical_core::scene_input::SceneBuildingId,
    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint,
)> {
    let mut centres = layout
        .playable
        .iter()
        .map(|building| (building.id, building.centre_metres))
        .chain(
            layout
                .distant
                .iter()
                .map(|building| (building.id, building.centre_metres)),
        )
        .collect::<Vec<_>>();
    centres.sort_by_key(|(id, _)| *id);
    centres
}

#[test]
fn layout_is_stable_and_population_increases_occupied_cells() {
    let village = layout("lübeck", 900);
    let town = layout("lübeck", 6_500);
    let city = layout("lübeck", 40_000);
    assert_eq!(village, layout("lübeck", 900));
    assert!(
        village.playable.len() + village.distant.len() < town.playable.len() + town.distant.len()
    );
    assert!(town.playable.len() + town.distant.len() < city.playable.len() + city.distant.len());
    assert!(city.playable.iter().all(|building| {
        building.centre_metres.metres().abs().max_element() <= 50.0
            && building.orientation.is_valid()
    }));
    assert!(
        city.distant
            .iter()
            .all(|building| building.centre_metres.metres().abs().max_element() > 50.0)
    );
}

#[test]
fn playable_boundary_only_changes_representation() {
    let compact = place_settlement_buildings(&settlement("lübeck", 40_000), 35.0).unwrap();
    let broad = place_settlement_buildings(&settlement("lübeck", 40_000), 90.0).unwrap();
    assert_eq!(ordered_centres(&compact), ordered_centres(&broad));
    assert!(compact.playable.len() < broad.playable.len());
}

#[test]
fn missing_estimate_uses_the_shared_population_level_fallback() {
    let settlement = SettlementSceneProfile {
        id: "fallback-city".into(),
        population_level: 4,
        population_estimate: 0,
        economy: economy(6_500),
        operators: Vec::new(),
    };
    let population = settlement.effective_population();
    let buildings = place_settlement_buildings(&settlement, 50.0).unwrap();
    let expected = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            adventuresim_core::settlement_population::settlement_building_seed(&settlement.id),
            adventuresim_core::settlement_property::ResidentCount::new(population),
            &settlement.economy,
        )
        .unwrap()
        .lots
        .len();
    assert_eq!(
        buildings.playable.len() + buildings.distant.len(),
        expected + buildings.compounds.len()
    );
}

#[test]
fn dense_city_layout_passes_tactical_pad_validation() {
    let layout = place_settlement_buildings(&settlement("dense", 40_000), 50.0).unwrap();
    // Keep complete property metadata while exercising only playable pads.
    let mut input = TacticalSceneInput {
        grounding: None,
        properties: None,
        schema_version: TACTICAL_SCENE_SCHEMA_VERSION,
        generation_version: TACTICAL_SCENE_GENERATION_VERSION,
        seed: 42.into(),
        scene_key: "city".into(),
        source: SceneSource::SyntheticFixture("city".into()),
        latitude_microdegrees: const {
            match adventuresim_world_schema::coordinates::LatitudeMicrodegrees::new(53_500_000) {
                Some(value) => value,
                None => panic!("invalid authored latitude"),
            }
        },
        longitude_microdegrees: const {
            match adventuresim_world_schema::coordinates::LongitudeMicrodegrees::new(10_000_000) {
                Some(value) => value,
                None => panic!("invalid authored longitude"),
            }
        },
        absolute_minute: adventuresim_world_schema::calendar::StrategicMinute::new(1),
        lunar_phase_minute: adventuresim_world_schema::calendar::StrategicMinute::new(1),
        absolute_elevation_metres: adventuresim_world_schema::ElevationMeters::new(0).unwrap(),
        playable: TerrainSampleGrid {
            width: 101,
            depth: 101,
            spacing_metres: 1.0,
            heights_metres: vec![0.0; 101 * 101],
            environment: vec![EnvironmentalSample::default(); 101 * 101],
        },
        landform: None,
        streets: layout.streets.clone(),
        yards: layout.yards.clone(),
        parishes: layout.parishes.clone(),
        compounds: layout.compounds.clone(),
        gardens: layout.gardens.clone(),
        buildings: layout.playable.clone(),
        distant_buildings: layout.distant.clone(),
        establishments: Vec::new(),
        vista: VistaSample {
            lods: vec![VistaLod {
                level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(1),
                spacing_metres: 1_000.0,
                width: 41,
                depth: 41,
                origin_east_metres: 0.0,
                origin_north_metres: 0.0,
                heights_metres: vec![0.0; 41 * 41],
                environment: vec![EnvironmentalSample::default(); 41 * 41],
            }],
        },
        weather: adventuresim_core::weather::weather_at(
            fabelgeist_determinism::Seed::from_u64(42),
            adventuresim_world_schema::calendar::StrategicMinute::new(1),
            53_500_000,
            10_000_000,
            0,
        ),
    };
    input = input
        .ground_generated_city(
            &layout,
            adventuresim_tactical_core::city_layout::CompoundGradingPolicy::bounded_settlement(),
        )
        .expect("complete flat city retains accepted property support");
    input.validate().unwrap();
    let generated = input.generate().unwrap();
    assert_eq!(generated.buildings.len(), input.buildings.len());
    assert!(!generated.buildings.is_empty());
    input.buildings.reverse();
    assert!(input.generate().is_ok());
}

#[test]
fn large_city_uses_valid_deterministic_recipes_and_preserves_all_plots() {
    let buildings = place_settlement_buildings(&settlement("massive-city-3229", 100_000), 50.0)
        .expect("the city should fit its population and produce valid recipes");

    assert_eq!(
        buildings.playable.len() + buildings.distant.len(),
        CitySite::central_german_market_town()
            .unwrap()
            .generate(
                adventuresim_core::settlement_population::settlement_building_seed(
                    "massive-city-3229",
                ),
                adventuresim_core::settlement_property::ResidentCount::new(100_000),
                &economy(100_000)
            )
            .unwrap()
            .lots
            .len()
            + buildings.compounds.len()
    );
    assert!(
        buildings
            .playable
            .iter()
            .all(|building| generate(&building.program).is_ok())
    );
    assert_eq!(
        buildings,
        place_settlement_buildings(&settlement("massive-city-3229", 100_000), 50.0).unwrap()
    );
}

#[test]
fn city_house_class_dimensions_match_generated_programmes() {
    for house_class in CityHouseClass::ALL {
        let program = BuildingProgram::fixture(
            house_class.archetype(),
            fabelgeist_determinism::Seed::from_u64(42),
        );
        let (width_cells, depth_cells) = program.footprint.dimensions();
        assert_eq!(
            bevy::math::Vec2::new(
                f32::from(width_cells) * adventuresim_building_generator::CELL_SIZE_METRES,
                f32::from(depth_cells) * adventuresim_building_generator::CELL_SIZE_METRES,
            ),
            house_class.footprint().unwrap().metres()
        );
    }
}
