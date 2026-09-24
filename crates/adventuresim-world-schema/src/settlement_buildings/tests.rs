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
                .filter(|b| b.usage() == usage)
                .collect::<Vec<_>>();
            assert_eq!(
                !buildings.is_empty(),
                definition.eligible(population, &economy)
            );
            if buildings.is_empty() {
                continue;
            }
            if matches!(
                definition.demand,
                BuildingDemandPolicy::SettlementInstitution(_)
            ) {
                assert_eq!(buildings.len(), 1);
                continue;
            }
            let BuildingDemandPolicy::ServiceCatchment(range) = definition.demand else {
                continue;
            };
            let capacities = buildings
                .iter()
                .map(|b| match b {
                    BuildingDemand::Service { capacity, .. } => capacity.0,
                    _ => panic!("service policy emitted an institutional request"),
                })
                .collect::<Vec<_>>();
            let capacity: u32 = capacities.iter().sum();
            assert!(capacity >= population);
            assert!(capacity - capacities.last().unwrap() < population);
            assert!(
                capacities
                    .iter()
                    .all(|capacity| *capacity >= range.minimum.0 && *capacity <= range.maximum.0)
            );
        }
    }
}

#[test]
fn business_ordinals_are_stable_coordinates_not_iteration_positions() {
    let economy = economy(40_000, 4);
    let plan = SettlementBuildingDemand::new(42, 40_000, &economy);
    let assigned = plan
        .buildings
        .iter()
        .filter_map(|demand| demand.business_key())
        .collect::<Vec<_>>();
    let repeated = SettlementBuildingDemand::new(42, 40_000, &economy)
        .buildings
        .iter()
        .filter_map(|demand| demand.business_key())
        .collect::<Vec<_>>();
    assert_eq!(assigned, repeated);

    let mut reverse_consumption = plan.buildings.iter().rev().collect::<Vec<_>>();
    reverse_consumption.sort_by_key(|demand| demand.business_key());
    let mut sorted_assigned = assigned.clone();
    sorted_assigned.sort();
    assert_eq!(
        reverse_consumption
            .into_iter()
            .filter_map(|demand| demand.business_key())
            .collect::<Vec<_>>(),
        sorted_assigned
    );

    for usage in BuildingUse::ALL {
        let ordinals = assigned
            .iter()
            .filter(|key| key.usage == usage)
            .map(|key| key.ordinal)
            .collect::<Vec<_>>();
        assert_eq!(ordinals, (0..ordinals.len() as u32).collect::<Vec<_>>());
    }
}

#[test]
fn global_business_identity_includes_its_settlement_scope() {
    let key = BusinessKey {
        usage: BuildingUse::Inn,
        ordinal: 0,
    };
    assert_ne!(
        BusinessId::new("lübeck", key),
        BusinessId::new("hamburg", key)
    );
}

#[test]
fn specialist_presence_is_owned_by_the_economy() {
    let village = economy(120, 1);
    let village_plan = SettlementBuildingDemand::new(42, 120, &village);
    assert!(
        !village_plan
            .buildings
            .iter()
            .any(|b| b.usage() == BuildingUse::Weaponsmith)
    );
    let mut town = economy(8_000, 4);
    assert!(town.has_service(SettlementService::Weaponsmith));
    assert!(
        SettlementBuildingDemand::new(42, 8_000, &town)
            .buildings
            .iter()
            .any(|b| b.usage() == BuildingUse::Weaponsmith)
    );
    town.services
        .retain(|s| *s != SettlementService::Weaponsmith);
    assert!(
        !SettlementBuildingDemand::new(42, 8_000, &town)
            .buildings
            .iter()
            .any(|b| b.usage() == BuildingUse::Weaponsmith)
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
            .filter(|b| b.usage() == BuildingUse::ParishChurch)
            .copied()
            .collect::<Vec<_>>()
    };
    let smaller = churches(&small);
    let larger = churches(&large);
    assert!(smaller.len() > 1);
    assert!(larger.len() > smaller.len());
    assert_eq!(
        large.parishes.iter().map(|p| p.population.0).sum::<u32>(),
        8_000
    );
    assert!(
        !large
            .buildings
            .iter()
            .any(|b| b.usage() == BuildingUse::Cathedral)
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

#[test]
fn parish_programmes_keep_population_roles_and_physical_scale_separate() {
    let economy = economy(30_000, 4);
    for population in [0, 250, 900, 3_001, 30_000] {
        let plan = SettlementBuildingDemand::new(42, population, &economy);
        assert_eq!(
            plan.parishes.iter().map(|p| p.population.0).sum::<u32>(),
            population
        );
        for parish in &plan.parishes {
            let roles = plan
                .buildings
                .iter()
                .filter_map(|request| match request {
                    BuildingDemand::Parish {
                        parish: owner,
                        role,
                    } if *owner == parish.id => Some(*role),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                roles
                    .iter()
                    .filter(|role| matches!(role, ParishBuildingRole::Church(_)))
                    .count(),
                1
            );
            assert_eq!(
                roles
                    .iter()
                    .filter(|role| **role == ParishBuildingRole::Residence)
                    .count(),
                1
            );
            assert_eq!(
                roles.contains(&ParishBuildingRole::TownSchool),
                parish.id.0 == 0 && population >= 800
            );
            if population < 1_500 {
                assert_eq!(parish.church_scale, ChurchBuildingScale::Village);
            }
        }
    }
    let normal = SettlementBuildingDemand::new(42, 30_000, &economy);
    let church_rich = SettlementBuildingDemand::with_parish_policy(
        42,
        30_000,
        &economy,
        AuthoredParishPolicy {
            target_population: std::num::NonZeroU32::new(1_000).unwrap(),
        },
    );
    assert_eq!(normal.parishes.len(), 10);
    assert_eq!(church_rich.parishes.len(), 30);
    assert_eq!(
        church_rich
            .parishes
            .iter()
            .map(|p| p.population.0)
            .sum::<u32>(),
        30_000
    );
    let mut secular = economy;
    secular
        .services
        .retain(|service| *service != SettlementService::Temple);
    let plan = SettlementBuildingDemand::new(42, 30_000, &secular);
    assert!(plan.parishes.is_empty());
    assert!(
        !plan
            .buildings
            .iter()
            .any(|b| matches!(b, BuildingDemand::Parish { .. }))
    );
}
