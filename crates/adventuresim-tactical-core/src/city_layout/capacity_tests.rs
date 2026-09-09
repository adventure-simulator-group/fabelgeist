use super::tests::economy;
use super::*;
use adventuresim_building_generator::{BuildingProgram, ServiceBuildingSize, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::ServiceCapacity;

#[test]
fn service_capacity_bands_reserve_church_and_workplace_plots_before_siting() {
    let nodes = street_nodes(42);
    let candidates = city_blocks(nodes)
        .filter(|block| block_is_inside_city(*block) && !block.is_market())
        .flat_map(|block| block_lots(42, block))
        .collect::<Vec<_>>();
    let mut demand = Vec::new();
    for usage in [
        BuildingUse::Chapel,
        BuildingUse::ParishChurch,
        BuildingUse::Stable,
        BuildingUse::Dyer,
    ] {
        let range = usage.definition().capacity;
        for (ordinal, capacity) in [
            range.minimum,
            ServiceCapacity((range.minimum.0 + range.maximum.0) / 2),
            range.maximum,
        ]
        .into_iter()
        .enumerate()
        {
            demand.push(BuildingDemand {
                usage,
                ordinal: ordinal as u32,
                capacity,
            });
        }
    }
    let (placed, unplaced) = services::place_services(42, 6_500, nodes, &candidates, &demand);
    assert!(
        unplaced.is_empty(),
        "mixed service requests were lost: {unplaced:?}"
    );
    assert_eq!(placed.len(), demand.len());
    for (index, candidate) in placed.iter().enumerate() {
        let lot = candidate.lot;
        let request = lot.service.unwrap();
        let size = [
            ServiceBuildingSize::Small,
            ServiceBuildingSize::Medium,
            ServiceBuildingSize::Large,
        ][request.ordinal as usize];
        assert_eq!(lot.service_size(), Some(size));
        let program = BuildingProgram::settlement(
            settlement_archetype(request.usage),
            Some(request.usage),
            42,
        )
        .with_service_size(size);
        assert_eq!(
            lot.dimensions_metres(),
            program.plot_dimensions_metres(),
            "{:?} {size:?} reserved the wrong plot",
            request.usage
        );
        assert!(
            placed[index + 1..]
                .iter()
                .all(|other| !lots_overlap(lot, other.lot))
        );
    }
    for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
        let footprints = demand
            .iter()
            .filter(|request| request.usage == usage)
            .map(|request| {
                placed
                    .iter()
                    .find(|candidate| candidate.lot.service == Some(*request))
                    .unwrap()
                    .lot
                    .dimensions_metres()
            })
            .collect::<Vec<_>>();
        assert!(
            footprints
                .windows(2)
                .all(|pair| pair[0].element_product() < pair[1].element_product()),
            "{usage:?} capacity does not change reserved area"
        );
    }
}

#[test]
fn generated_neighbourhoods_preserve_requested_churches_and_workplaces_with_resident_lots() {
    for (seed, population) in [(42, 900), (101, 6_500)] {
        let demand = SettlementBuildingDemand::new(seed, population, &economy());
        let city = generate_city(seed, population, &economy());
        assert!(city.unplaced_services.is_empty());
        assert_eq!(city.unhoused_population, 0);
        let services = city
            .lots
            .iter()
            .filter_map(|lot| lot.service)
            .collect::<Vec<_>>();
        assert_eq!(services.len(), demand.buildings.len());
        for request in demand.buildings {
            assert!(
                services.contains(&request),
                "lost {request:?} after residential placement"
            );
        }
        assert!(
            services
                .iter()
                .any(|request| request.usage == BuildingUse::ParishChurch)
        );
        assert!(services.iter().any(|request| {
            adventuresim_building_generator::WorkplaceKind::from_use(request.usage).is_some()
        }));
        for (index, lot) in city.lots.iter().enumerate() {
            assert!(
                city.lots[index + 1..]
                    .iter()
                    .all(|other| !lots_overlap(*lot, *other))
            );
        }
    }
}
