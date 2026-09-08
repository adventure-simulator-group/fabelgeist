use super::*;
use adventuresim_building_generator::generate;
use adventuresim_tactical_core::prelude::*;

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
    }
}

fn layout(id: &str, population: u32) -> SettlementBuildingLayout {
    place_settlement_buildings(&settlement(id, population), 50.0).unwrap()
}

fn ordered_centres(layout: &SettlementBuildingLayout) -> Vec<(u64, bevy::math::Vec2)> {
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
        building.centre_metres.abs().max_element() <= 50.0 && building.orientation.is_valid()
    }));
    assert!(
        city.distant
            .iter()
            .all(|building| building.centre_metres.abs().max_element() > 50.0)
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
    };
    let population = settlement.effective_population();
    let buildings = place_settlement_buildings(&settlement, 50.0).unwrap();
    let expected = generate_city(
        settlement_seed(&settlement.id),
        population,
        &settlement.economy,
    )
    .lots
    .len();
    assert_eq!(buildings.playable.len() + buildings.distant.len(), expected);
}

#[test]
fn dense_city_layout_passes_tactical_pad_validation() {
    let layout = place_settlement_buildings(&settlement("dense", 40_000), 50.0).unwrap();
    let mut input = TacticalSceneInput {
        schema_version: TACTICAL_SCENE_SCHEMA_VERSION,
        generation_version: TACTICAL_SCENE_GENERATION_VERSION,
        seed: 42,
        scene_key: "city".into(),
        source: SceneSource::SyntheticFixture("city".into()),
        latitude_microdegrees: 53_500_000,
        longitude_microdegrees: 10_000_000,
        absolute_minute: 1,
        lunar_phase_minute: 1,
        absolute_elevation_metres: 0,
        playable: TerrainSampleGrid {
            width: 101,
            depth: 101,
            spacing_metres: 1.0,
            heights_metres: vec![0.0; 101 * 101],
            environment: vec![EnvironmentalSample::default(); 101 * 101],
        },
        landform: None,
        streets: layout.streets,
        yards: layout.yards,
        buildings: layout.playable,
        distant_buildings: Vec::new(),
        vista: VistaSample::default(),
        weather: adventuresim_core::weather::weather_at(42, 1, 53_500_000, 10_000_000, 0),
    };
    input.validate().unwrap();
    let generated = input.generate().unwrap();
    assert_eq!(generated.buildings.len(), input.buildings.len());
    assert!(!generated.buildings.is_empty());
    input.buildings.reverse();
    assert!(input.generate().is_ok());
}

#[test]
fn invalid_initial_recipe_seed_advances_deterministically_to_a_valid_program() {
    let buildings = place_settlement_buildings(&settlement("massive-city-3229", 100_000), 50.0)
        .expect("the deterministic retry sequence should find valid recipes");

    assert_eq!(
        buildings.playable.len() + buildings.distant.len(),
        generate_city(
            settlement_seed("massive-city-3229"),
            100_000,
            &economy(100_000)
        )
        .lots
        .len()
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
        let program = BuildingProgram::fixture(house_class.archetype(), 42);
        let (width_cells, depth_cells) = program.footprint.dimensions();
        assert_eq!(
            bevy::math::Vec2::new(
                f32::from(width_cells) * adventuresim_building_generator::CELL_SIZE_METRES,
                f32::from(depth_cells) * adventuresim_building_generator::CELL_SIZE_METRES,
            ),
            bevy::math::Vec2::new(
                house_class.frontage_width_metres(),
                house_class.depth_metres(),
            )
        );
    }
}
