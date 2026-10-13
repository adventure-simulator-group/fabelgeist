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
            resident_capacity: HomeCapacity::new(std::num::NonZeroU32::MIN.saturating_add(7)),
            supply_role: market.reserve(tier),
            east_metres: index as f32 * 10.0,
            north_metres: 0.0,
            yaw_radians: 0.0,
            width_metres: 8.0,
            depth_metres: 12.0,
        })
        .collect();
    GeneratedHomeCatalog {
        settlement_id: settlement.into(),
        population: ResidentCount::new(20),
        seed: crate::settlement_population::settlement_building_seed(settlement),
        homes,
    }
}

#[test]
fn scope_duplicates_and_invalid_geometry_are_rejected() {
    let mut homes = catalog();
    homes.validate("town", ResidentCount::new(20)).unwrap();
    assert_eq!(
        homes.validate("other", ResidentCount::new(20)),
        Err(PropertyError::SettlementMismatch)
    );
    homes.homes[1].id = homes.homes[0].id.clone();
    assert_eq!(
        homes.validate("town", ResidentCount::new(20)),
        Err(PropertyError::InvalidCatalog)
    );
    homes = catalog();
    homes.homes[0].width_metres = f32::NAN;
    assert_eq!(
        homes.validate("town", ResidentCount::new(20)),
        Err(PropertyError::InvalidCatalog)
    );
}

#[test]
fn market_supply_cannot_be_counted_as_population_housing() {
    let mut homes = catalog();
    homes.population = ResidentCount::new(30);
    assert_eq!(
        homes.validate("town", ResidentCount::new(30)),
        Err(PropertyError::InsufficientCapacity)
    );
}

#[test]
fn exact_family_allocation_preserves_population_and_reserves_empty_homes() {
    let homes = catalog();
    let allocated = homes
        .allocate(&[
            HouseholdRequest {
                household_id: "family-a".into(),
                residents: ResidentCount::new(4),
            },
            HouseholdRequest {
                household_id: "family-b".into(),
                residents: ResidentCount::new(3),
            },
        ])
        .unwrap();
    assert_eq!(
        allocated
            .iter()
            .map(|home| home.residents.get())
            .sum::<u32>(),
        homes.population.get()
    );
    assert!(allocated.len() < homes.population.get() as usize);
    for home in &homes.homes {
        let occupants: u32 = allocated
            .iter()
            .filter(|assignment| assignment.property_id == home.id)
            .map(|assignment| assignment.residents.get())
            .sum();
        assert!(occupants <= home.resident_capacity.get());
        if home.supply_role == HomeSupplyRole::MarketReserve {
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
        homes.allocate(&[HouseholdRequest {
            household_id: "family".into(),
            residents: ResidentCount::new(21)
        }]),
        Err(PropertyError::HouseholdPopulation)
    );
    assert_eq!(
        homes.allocate(&[HouseholdRequest {
            household_id: "family".into(),
            residents: ResidentCount::new(9)
        }]),
        Err(PropertyError::InsufficientCapacity)
    );
}
