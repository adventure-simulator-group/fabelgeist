//! Producer/consumer regressions use the production city compiler and scene path.
use super::*;
use crate::{city_layout::CitySite, scene_input::tests::fixture};
use adventuresim_world_schema::SettlementEconomyProfile;
use bevy::math::{Vec2, Vec3};

mod matrix;

fn city_input() -> (TacticalSceneInput, CitySceneLayout) {
    city_input_for(42, 900)
}

fn city_input_for(seed: u64, population: u32) -> (TacticalSceneInput, CitySceneLayout) {
    let layout = CitySite::central_german_market_town()
        .generate(
            seed,
            population,
            &SettlementEconomyProfile::stage_placeholder(),
        )
        .compile(seed)
        .unwrap()
        .partition(Some(50.0))
        .unwrap();
    let mut input = fixture();
    input.seed = seed;
    input.playable = TerrainSampleGrid {
        width: 101,
        depth: 101,
        spacing_metres: 1.0,
        heights_metres: vec![0.0; 101 * 101],
        environment: vec![EnvironmentalSample::default(); 101 * 101],
    };
    input.vista = VistaSample {
        lods: vec![VistaLod {
            level: 0,
            width: 41,
            depth: 41,
            spacing_metres: 50.0,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 41 * 41],
            environment: vec![EnvironmentalSample::default(); 41 * 41],
        }],
    };
    input.buildings = layout.playable.clone();
    input.distant_buildings = layout.distant.clone();
    input.compounds = layout.compounds.clone();
    input.gardens = layout.gardens.clone();
    input.parishes = layout.parishes.clone();
    input.streets = layout.streets.clone();
    input.yards = layout.yards.clone();
    (input, layout)
}

#[test]
fn production_support_handoff_preserves_source_identity_members_and_programmes() {
    let (draft, layout) = city_input();
    let before = draft.clone();
    assert!(
        draft.validate().is_err(),
        "occupied drafts are never runtime inputs"
    );
    let input = draft
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap();
    assert_eq!(input.playable, before.playable);
    assert_eq!(input.vista, before.vista);
    assert_eq!(input.compounds, before.compounds);
    assert_eq!(input.gardens, before.gardens);
    let bytes = serde_json::to_vec(&input).unwrap();
    let restored: TacticalSceneInput = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    let prepared = restored
        .prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    let source =
        GeographicSurface::from_presented_scene(&prepared.terrain, &restored.vista).unwrap();
    let generated = restored
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    assert_eq!(generated.repairs.levelled_building_samples, 0);
    assert_eq!(generated.buildings.len(), restored.buildings.len());
    for building in &generated.buildings {
        let floor = building.placement.base_elevation_metres;
        let centre = building.placement.centre_metres;
        assert!(
            (generated
                .terrain
                .surface_below(Vec3::new(centre.x, floor + 0.001, centre.y))
                .unwrap()
                .0
                - floor)
                .abs()
                < 0.001
        );
        let expected = before
            .buildings
            .iter()
            .find(|b| b.id == building.placement.id)
            .unwrap();
        assert_eq!(building.placement.program, expected.program);
        assert_eq!(building.placement.centre_metres, expected.centre_metres);
        assert_eq!(building.placement.orientation, expected.orientation);
    }
    for point in [Vec2::new(900.0, 900.0), Vec2::new(-900.0, -900.0)] {
        assert_eq!(
            generated.terrain.height_at(point),
            source.elevation_at(point).map(|h| h.metres())
        );
    }
}

#[test]
fn occupied_inputs_reject_missing_stale_and_mismatched_support_without_fallback() {
    let (draft, layout) = city_input();
    let input = draft
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap();
    let mut missing = input.clone();
    missing.grounding = None;
    assert!(missing.generate().is_err());
    let mut changed = input.clone();
    changed.buildings[0].base_elevation_metres += 0.01;
    assert!(matches!(
        changed.validate(),
        Err(SceneInputError::GroundingProjection(error))
            if matches!(*error, crate::city_layout::CityGroundingProjectionError::PlacementMismatch)
    ));
    let mut stale = input;
    stale.vista.lods[0].heights_metres[0] += 0.01;
    assert!(matches!(
        stale.generate(),
        Err(SceneInputError::GroundingProjection(error))
            if matches!(*error, crate::city_layout::CityGroundingProjectionError::SourceMismatch)
    ));
    let (draft, mut wrong) = city_input();
    wrong.playable[0].centre_metres.x += 0.01;
    assert!(
        draft
            .ground_generated_city(&wrong, CompoundGradingPolicy::bounded_settlement())
            .is_err()
    );
}

#[test]
fn prepared_support_phase_matches_full_generation_and_rejects_removed_members() {
    let input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/garden-review.json"
    )))
    .unwrap();
    let prepared = input
        .prepare_supported_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    let complete = input
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&prepared.terrain).unwrap(),
        serde_json::to_vec(&complete.terrain).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&prepared.ground).unwrap(),
        serde_json::to_vec(&complete.ground).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&prepared.repairs).unwrap(),
        serde_json::to_vec(&complete.repairs).unwrap()
    );
    assert!(prepared.terrain.property_surface().is_some());
    let mut incomplete = input.clone();
    assert!(incomplete.distant_buildings.pop().is_some());
    let rejection = incomplete.prepare_supported_terrain(&mut GeneratedBuildingRecipes::default());
    assert_eq!(
        rejection.unwrap_err().to_string(),
        "scene input is invalid: garden owner must reference an occupied front building"
    );
    let mut mismatched = input;
    mismatched.distant_buildings[0].base_elevation_metres += 0.01;
    let rejection = mismatched.prepare_supported_terrain(&mut GeneratedBuildingRecipes::default());
    assert!(
        matches!(&rejection,
        Err(SceneInputError::GroundingProjection(error))
            if matches!(**error, crate::city_layout::CityGroundingProjectionError::PlacementMismatch)),
        "specific placement rejection: {rejection:?}"
    );
}
