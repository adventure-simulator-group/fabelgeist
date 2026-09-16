use super::*;
use crate::city_layout::CitySite;

fn fixture() -> TacticalSceneInput {
    let mut economy = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
    economy.services = vec![adventuresim_world_schema::SettlementService::Temple];
    let city = CitySite::central_german_market_town().generate(42, 6_500, &economy);
    let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/flat-dry-grassland.json"
    ))
    .unwrap();
    input.buildings.clear();
    input.compounds.clear();
    input.parishes = city.parish_layout().unwrap();
    input.distant_buildings = city
        .lots
        .iter()
        .map(|lot| DistantBuildingPlacement {
            id: lot.id,
            archetype: lot.archetype(),
            usage: Some(lot.building_use().unwrap_or(BuildingUse::Dwelling)),
            service_size: lot.service_size(),
            seed: 42,
            centre_metres: lot.centre_metres,
            base_elevation_metres: 0.0,
            orientation: lot.orientation,
        })
        .collect();
    input
}

#[test]
fn parish_scene_round_trip_preserves_population_and_near_far_ownership() {
    let mut input = fixture();
    validate(&input).unwrap();
    let church_index = input
        .distant_buildings
        .iter()
        .position(|b| b.id == input.parishes[0].church_building_id)
        .unwrap();
    let church = input.distant_buildings.remove(church_index);
    input.buildings.push(TacticalBuildingPlacement {
        id: church.id,
        program: church.program(),
        centre_metres: church.centre_metres,
        orientation: church.orientation,
    });
    validate(&input).unwrap();
    let encoded = serde_json::to_string(&input).unwrap();
    let decoded: TacticalSceneInput = serde_json::from_str(&encoded).unwrap();
    assert_eq!(input.parishes, decoded.parishes);
    validate(&decoded).unwrap();
}

#[test]
fn parish_scene_rejects_missing_shared_overfilled_and_distant_members() {
    let input = fixture();
    let mutations: [fn(&mut TacticalSceneInput); 8] = [
        |scene| {
            scene.parishes[0].rectory_building_id = u64::MAX;
        },
        |scene| {
            scene.parishes[1].rectory_building_id = scene.parishes[0].rectory_building_id;
        },
        |scene| {
            scene.parishes[0].residences.truncate(1);
            scene.parishes[0].residences[0].residents.0 = u32::MAX;
            scene.parishes[0].programme.population.0 = u32::MAX;
        },
        |scene| {
            let shared = scene.parishes[0].residences[0];
            scene.parishes[1].residences.push(shared);
        },
        |scene| {
            let id = scene.parishes[0].rectory_building_id;
            scene
                .distant_buildings
                .iter_mut()
                .find(|b| b.id == id)
                .unwrap()
                .centre_metres += bevy::math::Vec2::splat(500.0);
        },
        |scene| {
            scene.parishes[0].programme.population.0 += 1;
        },
        |scene| {
            let id = scene.parishes[0].church_building_id;
            scene
                .distant_buildings
                .iter_mut()
                .find(|b| b.id == id)
                .unwrap()
                .service_size = None;
        },
        |scene| {
            scene.parishes.pop();
        },
    ];
    for mutate in mutations {
        let mut broken = input.clone();
        mutate(&mut broken);
        assert!(validate(&broken).is_err());
    }
}
