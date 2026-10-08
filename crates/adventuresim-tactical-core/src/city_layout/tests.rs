use super::*;
pub(super) fn economy() -> SettlementEconomyProfile {
    let mut economy = SettlementEconomyProfile::stage_placeholder();
    economy.services = vec![
        adventuresim_world_schema::SettlementService::Inn,
        adventuresim_world_schema::SettlementService::Temple,
    ];
    economy
}

#[test]
fn city_lots_are_deterministic_and_follow_many_connected_street_segments() {
    let small = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(8_000),
            &economy(),
        )
        .unwrap();
    let large = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(40_000),
            &economy(),
        )
        .unwrap();
    assert_eq!(
        small,
        CitySite::central_german_market_town()
            .unwrap()
            .generate(
                (42).into(),
                adventuresim_core::settlement_property::ResidentCount::new(8_000),
                &economy()
            )
            .unwrap()
    );
    assert!(small.lots.len() < large.lots.len());
    assert!(large.unplaced_services.is_empty());
    assert_eq!(large.unhoused_population, ResidentCount::ZERO);
    let mut headings = large
        .lots
        .iter()
        .map(|lot| (lot.orientation.yaw_radians().to_degrees() / 2.0).round() as i16)
        .collect::<Vec<_>>();
    headings.sort_unstable();
    headings.dedup();
    assert!(headings.len() >= 12, "headings={headings:?}");
}

#[test]
fn population_is_represented_by_physical_house_capacity() {
    for population in [900, 6_500, 40_000] {
        let lots = CitySite::central_german_market_town()
            .unwrap()
            .generate(
                (42).into(),
                adventuresim_core::settlement_property::ResidentCount::new(population),
                &economy(),
            )
            .unwrap()
            .lots;
        let mut market = adventuresim_core::settlement_property::HousingMarketReserve::default();
        let capacity = lots
            .iter()
            .filter(|lot| lot.service.is_none())
            .filter(|lot| {
                market.reserve(lot.house_class.housing_tier()) == HomeSupplyRole::PopulationHousing
            })
            .map(|lot| lot.house_class.resident_capacity().get())
            .sum::<u32>();
        assert!(
            capacity >= population,
            "population={population} capacity={capacity} lots={}",
            lots.len()
        );
        assert!(market.complete());
        assert!(capacity < population + 30);
    }
}

#[test]
fn accepted_building_footprints_do_not_overlap() {
    let lots = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(40_000),
            &economy(),
        )
        .unwrap()
        .lots;
    for (index, lot) in lots.iter().enumerate() {
        assert!(
            lots[index + 1..]
                .iter()
                .all(|other| !lots_overlap(lot.bounds().unwrap(), other.bounds().unwrap()))
        );
    }
}

#[test]
fn building_south_side_faces_out_of_its_block() {
    let block = CitySite::central_german_market_town()
        .unwrap()
        .street_graph(
            (42).into(),
            DevelopmentExtent::for_population(ResidentCount::new(40_000)).unwrap(),
        )
        .unwrap()
        .blocks[0];
    let tangent = (block.corners_metres()[1] - block.corners_metres()[0]).normalize();
    let inward = Vec2::new(-tangent.y, tangent.x);
    let orientation = BuildingOrientation::from_frontage_tangent(tangent).unwrap();
    assert!(orientation.local_to_world(-Vec2::Y).dot(-inward) > 0.999);
}

#[test]
fn street_surfaces_are_mixed_grass_free_patches_from_the_same_graph() {
    let city = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(40_000),
            &economy(),
        )
        .unwrap();
    assert!(city.streets.len() > 100);
    for surface in [
        CityStreetSurface::CompactedEarth,
        CityStreetSurface::Gravel,
        CityStreetSurface::Fieldstone,
    ] {
        assert!(city.streets.iter().any(|patch| patch.surface() == surface));
    }
    assert_eq!(
        city.streets
            .iter()
            .filter(|patch| matches!(patch, CityStreetPatch::Market { .. }))
            .count(),
        1
    );
    for patch in &city.streets {
        let centre = match *patch {
            CityStreetPatch::Corridor {
                start_metres,
                end_metres,
                ..
            } => {
                let start_metres = start_metres.metres();
                let end_metres = end_metres.metres();
                (start_metres + end_metres) * 0.5
            }
            CityStreetPatch::Market { corners_metres, .. } => {
                let corners_metres =
                    corners_metres.map(crate::scene_coordinates::ScenePlanPoint::metres);
                corners_metres.into_iter().sum::<Vec2>() * 0.25
            }
        };
        assert!(patch.contains(centre));
    }
}

#[test]
fn every_selected_lot_belongs_to_a_deterministic_developed_yard() {
    let city = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(40_000),
            &economy(),
        )
        .unwrap();
    assert!(!city.yards.is_empty());
    assert!(city.yards.iter().all(|yard| yard.is_valid()));
    assert!(
        city.yards
            .iter()
            .any(|yard| yard.surface == CityYardSurface::PackedEarth)
    );
    assert!(
        city.yards
            .iter()
            .all(|yard| yard.surface == CityYardSurface::PackedEarth),
        "cultivated beds are compiled only after an owned garden is accepted"
    );
    assert!(city.lots.iter().all(|lot| {
        city.yards
            .iter()
            .any(|yard| yard.contains(lot.centre_metres.metres()))
    }));
}

#[test]
fn every_street_corridor_connects_to_the_market_network() {
    for population in [120, 900, 6_500] {
        let city = CitySite::central_german_market_town()
            .unwrap()
            .generate(
                (42).into(),
                adventuresim_core::settlement_property::ResidentCount::new(population),
                &economy(),
            )
            .unwrap();
        let corridors = city
            .streets
            .iter()
            .filter_map(|patch| match patch {
                CityStreetPatch::Corridor {
                    start_metres,
                    end_metres,
                    ..
                } => {
                    let start_metres = start_metres.metres();
                    let end_metres = end_metres.metres();
                    Some((start_metres, end_metres))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut reached = vec![corridors[0].0];
        let mut previous = 0;
        while previous != reached.len() {
            previous = reached.len();
            for &(a, b) in &corridors {
                if reached.contains(&a) && !reached.contains(&b) {
                    reached.push(b);
                }
                if reached.contains(&b) && !reached.contains(&a) {
                    reached.push(a);
                }
            }
        }
        assert!(
            corridors
                .iter()
                .all(|(a, b)| reached.contains(a) && reached.contains(b))
        );
    }
}

#[test]
fn extreme_population_reports_shortfall_without_claiming_capacity() {
    let city = CitySite::central_german_market_town()
        .unwrap()
        .generate(
            (42).into(),
            adventuresim_core::settlement_property::ResidentCount::new(u32::MAX),
            &economy(),
        )
        .unwrap();
    assert_eq!(city.unhoused_population, ResidentCount::new(u32::MAX));
    assert!(!city.unplaced_services.is_empty());
}
