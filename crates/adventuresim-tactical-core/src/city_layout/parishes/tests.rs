use super::*;
use adventuresim_world_schema::settlement_buildings::{AuthoredParishPolicy, ParishProminence};

#[test]
fn parishes_own_real_housing_and_nearby_support_buildings() {
    for (seed, population) in [(42, 900), (101, 6_500), (47_114, 30_000)] {
        let city = CitySite::central_german_market_town().generate(
            seed,
            population,
            &super::super::tests::economy(),
        );
        assert!(
            city.unplaced_services.is_empty(),
            "{seed}: {:?}",
            city.unplaced_services
        );
        assert_eq!(city.unhoused_population, 0);
        let parishes = city.parish_layout().unwrap();
        assert_eq!(
            parishes
                .iter()
                .map(|p| p.programme.population.0)
                .sum::<u32>(),
            population
        );
        assert_eq!(
            parishes
                .iter()
                .filter(|p| p.programme.prominence == ParishProminence::PrincipalTown)
                .count(),
            1
        );
        let mut owned = BTreeSet::new();
        for parish in parishes {
            let church = city
                .lots
                .iter()
                .find(|lot| lot.id == parish.church_building_id)
                .unwrap();
            for id in [Some(parish.rectory_building_id), parish.school_building_id]
                .into_iter()
                .flatten()
            {
                let dependent = city.lots.iter().find(|lot| lot.id == id).unwrap();
                assert!(church.centre_metres.distance(dependent.centre_metres) <= 90.0);
            }
            assert!(!parish.residences.is_empty());
            for allocation in parish.residences {
                assert!(owned.insert(allocation.building_id));
                let house = city
                    .lots
                    .iter()
                    .find(|lot| lot.id == allocation.building_id)
                    .unwrap();
                assert!(house.service.is_none());
                assert!(allocation.residents.0 <= house.house_class.resident_capacity());
            }
        }
        assert_eq!(
            owned.len(),
            city.lots.iter().filter(|lot| lot.service.is_none()).count()
        );
    }
}

#[test]
fn authored_church_rich_city_keeps_institutions_and_real_catchments() {
    let mut site = CitySite::central_german_market_town();
    site.parish_policy = AuthoredParishPolicy {
        target_population: std::num::NonZeroU32::new(1_000).unwrap(),
    };
    let city = site.generate(47_114, 30_000, &super::super::tests::economy());
    assert!(city.unplaced_services.is_empty());
    let parishes = city.parish_layout().unwrap();
    assert_eq!(parishes.len(), 30);
    assert_eq!(
        parishes
            .iter()
            .map(|p| p.programme.population.0)
            .sum::<u32>(),
        30_000
    );
}

#[test]
fn incomplete_precinct_is_unplaced_as_a_group() {
    let site = CitySite::central_german_market_town();
    let graph = site.street_graph(42, DevelopmentExtent::for_population(900));
    let candidates = graph
        .blocks
        .iter()
        .copied()
        .filter(|block| !block.is_market())
        .flat_map(|block| block_lots(42, block))
        .collect::<Vec<_>>();
    let demand = SettlementBuildingDemand::new(42, 900, &super::super::tests::economy());
    let parish_requests = demand
        .buildings
        .iter()
        .copied()
        .filter(|d| matches!(d, BuildingDemand::Parish { .. }))
        .collect::<Vec<_>>();
    let (placed, _) =
        services::place_services(42, 900, &graph.blocks, &candidates, &parish_requests[..1]);
    let one_plot = candidates
        .iter()
        .find(|candidate| candidate.selection_key == placed[0].selection_key)
        .copied()
        .unwrap();
    let (placed, unplaced) =
        services::place_services(42, 900, &graph.blocks, &[one_plot], &parish_requests);
    assert!(placed.is_empty());
    assert_eq!(unplaced, parish_requests);
}

#[test]
fn precinct_retries_a_later_church_site_when_the_first_cannot_fit_dependents() {
    let site = CitySite::central_german_market_town();
    let graph = site.street_graph(42, DevelopmentExtent::for_population(6_500));
    let candidates = graph
        .blocks
        .iter()
        .copied()
        .filter(|block| !block.is_market())
        .flat_map(|block| block_lots(42, block))
        .collect::<Vec<_>>();
    let demand = SettlementBuildingDemand::new(42, 900, &super::super::tests::economy());
    let requests = demand
        .buildings
        .iter()
        .copied()
        .filter(|d| matches!(d, BuildingDemand::Parish { .. }))
        .collect::<Vec<_>>();
    let (first, _) =
        services::place_services(42, 6_500, &graph.blocks, &candidates, &requests[..1]);
    let isolated = first[0].selection_key;
    let restricted = candidates
        .iter()
        .copied()
        .filter(|candidate| {
            candidate.selection_key == isolated
                || candidate.lot.centre_metres.distance(Vec2::new(150.0, 0.0)) < 45.0
        })
        .collect::<Vec<_>>();
    let (placed, unplaced) =
        services::place_services(42, 6_500, &graph.blocks, &restricted, &requests);
    assert!(unplaced.is_empty());
    assert_eq!(placed.len(), requests.len());
    assert_ne!(placed[0].selection_key, isolated);
}
