use super::*;

fn catalog() -> GeneratedHomeCatalog {
    let settlement = "town";
    let mut market = HousingMarketReserve::default();
    let homes = HousingTier::ALL
        .into_iter()
        .chain(HousingTier::ALL)
        .enumerate()
        .map(|(index, tier)| GeneratedHome {
            id: PropertyId::new(settlement, index as u64 + 1).unwrap(),
            building_id: index as u64 + 1,
            tier,
            resident_capacity: 8,
            market_reserve: market.reserve(tier),
            east_metres: index as f32 * 10.0,
            north_metres: 0.0,
            yaw_radians: 0.0,
            width_metres: 8.0,
            depth_metres: 12.0,
        })
        .collect();
    GeneratedHomeCatalog {
        settlement_id: settlement.into(),
        population: 20,
        seed: crate::settlement_population::settlement_building_seed(settlement),
        homes,
    }
}

#[test]
fn scope_duplicates_and_invalid_geometry_are_rejected() {
    let mut homes = catalog();
    homes.validate("town", 20).unwrap();
    assert_eq!(
        homes.validate("other", 20),
        Err(PropertyError::SettlementMismatch)
    );
    homes.homes[1].id = homes.homes[0].id.clone();
    assert_eq!(
        homes.validate("town", 20),
        Err(PropertyError::InvalidCatalog)
    );
    homes = catalog();
    homes.homes[0].width_metres = f32::NAN;
    assert_eq!(
        homes.validate("town", 20),
        Err(PropertyError::InvalidCatalog)
    );
}

#[test]
fn market_supply_cannot_be_counted_as_population_housing() {
    let mut homes = catalog();
    homes.population = 30;
    assert_eq!(
        homes.validate("town", 30),
        Err(PropertyError::InsufficientCapacity)
    );
}

#[test]
fn exact_family_allocation_preserves_population_and_reserves_empty_homes() {
    let homes = catalog();
    let allocated = homes
        .allocate(&[("family-a".into(), 4), ("family-b".into(), 3)])
        .unwrap();
    assert_eq!(
        allocated.iter().map(|home| home.residents).sum::<u32>(),
        homes.population
    );
    assert!(allocated.len() < homes.population as usize);
    for home in &homes.homes {
        let occupants: u32 = allocated
            .iter()
            .filter(|assignment| assignment.property_id == home.id)
            .map(|assignment| assignment.residents)
            .sum();
        assert!(occupants <= home.resident_capacity);
        if home.market_reserve {
            assert_eq!(occupants, 0);
        }
    }
    assert_eq!(allocated[0].household_id, "family-a");
    assert_eq!(allocated[1].household_id, "family-b");
}

#[test]
fn impossible_households_fail_instead_of_creating_homes() {
    let homes = catalog();
    assert_eq!(
        homes.allocate(&[("family".into(), 21)]),
        Err(PropertyError::HouseholdPopulation)
    );
    assert_eq!(
        homes.allocate(&[("family".into(), 9)]),
        Err(PropertyError::InsufficientCapacity)
    );
}
