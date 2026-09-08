use super::*;
use crate::{
    FallbackIndustry, IndustryEvidence, InferredIndustryProfile, SettlementService,
    infer_settlement_economy,
};

fn economy(population: u32, level: i32) -> SettlementEconomyProfile {
    infer_settlement_economy(
        level,
        population,
        3,
        level >= 3,
        &InferredIndustryProfile::new(vec![IndustryEvidence::Fallback(
            FallbackIndustry::CroplandGrain,
        )])
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn capacities_cover_each_eligible_catchment_without_an_unnecessary_last_building() {
    for population in [1, 120, 900, 6_500, 40_000] {
        let economy = economy(population, 4);
        let demand = SettlementBuildingDemand::new(42, population, &economy);
        assert!(demand.shortfalls.is_empty());
        for usage in BuildingUse::ALL {
            let definition = usage.definition();
            let buildings = demand
                .buildings
                .iter()
                .filter(|b| b.usage == usage)
                .collect::<Vec<_>>();
            assert_eq!(
                !buildings.is_empty(),
                definition.eligible(population, &economy)
            );
            if buildings.is_empty() {
                continue;
            }
            if definition.singleton {
                assert_eq!(buildings.len(), 1);
                continue;
            }
            let capacity: u32 = buildings.iter().map(|b| b.capacity.0).sum();
            assert!(capacity >= population);
            assert!(capacity - buildings.last().unwrap().capacity.0 < population);
            assert!(
                buildings
                    .iter()
                    .all(|b| b.capacity.0 >= definition.capacity.minimum.0
                        && b.capacity.0 <= definition.capacity.maximum.0)
            );
        }
    }
}

#[test]
fn specialist_presence_is_owned_by_the_economy() {
    let village = economy(120, 1);
    let village_plan = SettlementBuildingDemand::new(42, 120, &village);
    assert!(
        !village_plan
            .buildings
            .iter()
            .any(|b| b.usage == BuildingUse::Weaponsmith)
    );
    let mut town = economy(8_000, 4);
    assert!(town.has_service(SettlementService::Weaponsmith));
    assert!(
        SettlementBuildingDemand::new(42, 8_000, &town)
            .buildings
            .iter()
            .any(|b| b.usage == BuildingUse::Weaponsmith)
    );
    town.services
        .retain(|s| *s != SettlementService::Weaponsmith);
    assert!(
        !SettlementBuildingDemand::new(42, 8_000, &town)
            .buildings
            .iter()
            .any(|b| b.usage == BuildingUse::Weaponsmith)
    );
}

#[test]
fn parish_growth_is_deterministic_and_does_not_duplicate_cathedrals() {
    let economy = economy(8_000, 4);
    let small = SettlementBuildingDemand::new(42, 4_000, &economy);
    let large = SettlementBuildingDemand::new(42, 8_000, &economy);
    assert_eq!(large, SettlementBuildingDemand::new(42, 8_000, &economy));
    assert_ne!(large, SettlementBuildingDemand::new(43, 8_000, &economy));
    let churches = |plan: &SettlementBuildingDemand| {
        plan.buildings
            .iter()
            .filter(|b| b.usage == BuildingUse::ParishChurch)
            .copied()
            .collect::<Vec<_>>()
    };
    let smaller = churches(&small);
    let larger = churches(&large);
    assert!(smaller.len() > 1);
    assert_eq!(smaller, larger[..smaller.len()]);
    assert!(
        !large
            .buildings
            .iter()
            .any(|b| b.usage == BuildingUse::Cathedral)
    );
}

#[test]
fn empty_and_extreme_populations_are_bounded_and_shortfalls_are_explicit() {
    let economy = economy(40_000, 4);
    assert!(
        SettlementBuildingDemand::new(42, 0, &economy)
            .buildings
            .is_empty()
    );
    let extreme = SettlementBuildingDemand::new(42, u32::MAX, &economy);
    assert_eq!(extreme.buildings.len(), MAX_SERVICE_BUILDINGS);
    assert!(!extreme.shortfalls.is_empty());
}
