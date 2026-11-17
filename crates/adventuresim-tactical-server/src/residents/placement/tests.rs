use super::*;
use adventuresim_building_generator::{BuildingProgram, interior::furnish, settlement_archetype};
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene_coordinates::ScenePlanPoint,
    scene_input::{BuildingOrientation, GeneratedBuildingRecipe, TacticalBuildingPlacement},
};
use adventuresim_world_schema::{
    person_names::RenderedPersonalName, settlement_buildings::BuildingUse,
};

pub(in crate::residents) fn merchant() -> (
    GeneratedBuilding,
    InteriorBuildingLayout,
    SceneEstablishment,
) {
    let usage = BuildingUse::GeneralShop;
    let program =
        BuildingProgram::validated_settlement(settlement_archetype(usage), usage, 0.into(), None)
            .unwrap();
    let recipe = GeneratedBuildingRecipe::generate(program.clone()).unwrap();
    let building = GeneratedBuilding {
        placement: TacticalBuildingPlacement {
            id: 1.into(),
            program,
            centre_metres: ScenePlanPoint::try_from(Vec2::new(12.0, -9.0)).unwrap(),
            base_elevation_metres: SupportElevation::from_metres(4.2).unwrap(),
            orientation: BuildingOrientation::from_radians(0.7).unwrap(),
        },
        plan: recipe.plan,
        collision: recipe.collision,
    };
    let interior = InteriorBuildingLayout {
        building_id: building.placement.id,
        layout: furnish(&building.plan, &building.placement.program).unwrap(),
    };
    let name = RenderedPersonalName::new("Anna the merchant").unwrap();
    let establishment = SceneEstablishment {
        building_id: building.placement.id,
        business_id: BusinessId::new(
            "station-fixture",
            adventuresim_world_schema::settlement_buildings::BusinessKey { usage, ordinal: 0 },
        ),
        operator_character_id: CharacterId(42),
        operator_name: name.clone(),
        shop_name: adventuresim_building_generator::signs::ShopName::for_operator(&name, usage),
    };
    (building, interior, establishment)
}

#[test]
fn occupied_merchant_station_keeps_identity_and_rebuilds_reachable_counter_access() {
    let (building, mut interior, establishment) = merchant();
    let prepare = |interior: &InteriorBuildingLayout| {
        OperatorStations::prepare(
            std::slice::from_ref(&building),
            std::slice::from_ref(interior),
            std::slice::from_ref(&establishment),
        )
        .unwrap()
    };
    let expected = prepare(&interior);
    interior.layout.paths.clear();
    let actual = prepare(&interior);
    assert_eq!(actual.0.len(), 1);
    let station = &actual.0[0];
    assert_eq!(station.character, establishment.operator_character_id);
    assert_eq!(station.business, establishment.business_id);
    assert_eq!(station.building, building.placement.id);
    assert!(matches!(
        station.approaches[0].role,
        StationRole::CounterOperator
    ));
    assert_eq!(
        station.approaches[0].arrival,
        expected.0[0].approaches[0].arrival
    );
    assert_eq!(station.approaches[0].via, expected.0[0].approaches[0].via);
    assert!(!station.approaches[0].via.is_empty());
    let counter_access = adventuresim_building_generator::interior::validate_layout(
        &building.plan,
        &interior.layout,
    )
    .unwrap()
    .into_iter()
    .find(|path| {
        interior.layout.placements[path.placement_index].key.kind()
            == adventuresim_building_generator::furniture::FurnitureKind::DisplayCounter
            && path.face == adventuresim_building_generator::furniture::FurnitureAccessFace::Back
    })
    .unwrap();
    let floor = counter_access
        .points
        .last()
        .unwrap()
        .floor_height(&building.plan)
        .unwrap();
    let expected_y = building.placement.base_elevation_metres.metres() + floor.metres();
    // Recentring a collision datum composes two f32 height additions. Budget
    // two adjacent representable heights rather than requiring an upward bias.
    let height_rounding = expected_y.next_down().next_down()..=expected_y.next_up().next_up();
    assert!(
        height_rounding.contains(&station.approaches[0].arrival.metres().y),
        "standing height={}, physical floor={expected_y}",
        station.approaches[0].arrival.metres().y
    );
}

#[test]
fn distant_businesses_supply_no_live_actor_and_repeated_character_bindings_are_rejected() {
    let (building, interior, establishment) = merchant();
    let mut distant = establishment.clone();
    distant.building_id = 2.into();
    assert!(
        OperatorStations::prepare(
            std::slice::from_ref(&building),
            std::slice::from_ref(&interior),
            &[distant]
        )
        .unwrap()
        .0
        .is_empty()
    );
    assert!(matches!(
        OperatorStations::prepare(
            &[building],
            &[interior],
            &[establishment.clone(), establishment]
        ),
        Err(ResidentPlacementError::DuplicateOperator {
            character: CharacterId(42)
        })
    ));
}
