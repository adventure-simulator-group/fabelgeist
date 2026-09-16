use super::*;

#[test]
fn principal_parish_retains_street_access_and_exact_recipe_across_city_partitions() {
    use adventuresim_building_generator::ServiceBuildingSize;
    for (seed, population) in [(42, 900), (47_114, 30_000)] {
        let city = CitySite::central_german_market_town().generate(
            seed,
            population,
            &super::super::tests::economy(),
        );
        let lots = city.lots.clone();
        let compiled = city.compile(seed).unwrap();
        let principals = compiled
            .buildings
            .iter()
            .filter(|b| b.program.church_program.is_some())
            .collect::<Vec<_>>();
        assert_eq!(principals.len(), usize::from(population > 1500));
        for principal in principals {
            assert_eq!(principal.program.usage, Some(BuildingUse::ParishChurch));
            assert_eq!(
                principal.program.service_size,
                Some(ServiceBuildingSize::Large)
            );
            let lot = lots.iter().find(|lot| lot.id == principal.id).unwrap();
            assert!(
                principal
                    .orientation
                    .local_to_world(-Vec2::X)
                    .distance(lot.orientation.local_to_world(-Vec2::Y))
                    < 0.001
            );
            let distant = compiled
                .clone()
                .partition(None)
                .unwrap()
                .distant
                .into_iter()
                .find(|building| building.id == principal.id)
                .unwrap();
            assert_eq!(distant.program(), principal.program);
            assert_eq!(distant.orientation, principal.orientation);
        }
    }
}

#[test]
fn compiled_compounds_preserve_capacity_identity_and_exact_distant_recipes() {
    let city =
        CitySite::central_german_market_town().generate(42, 900, &super::super::tests::economy());
    let front_count = city.lots.len();
    let merchant_count = city.lots.iter().filter(|lot| lot.has_rear_range()).count();
    let compiled = city.compile(42).unwrap();
    assert!(merchant_count > 0);
    assert_eq!(compiled.compounds.len(), merchant_count);
    assert_eq!(compiled.buildings.len(), front_count + merchant_count);
    let ids = compiled
        .buildings
        .iter()
        .map(|b| b.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), compiled.buildings.len());
    for compound in &compiled.compounds {
        assert_ne!(compound.front_building_id, compound.rear_building_id);
        let rear = compiled
            .buildings
            .iter()
            .find(|b| b.id == compound.rear_building_id)
            .unwrap();
        assert_eq!(rear.program.archetype, BuildingArchetype::StorageRange);
        assert_eq!(rear.program.usage, None);
        let gate = compound.boundary.gate.door(compound.id);
        assert!(gate.opening.0 > u64::from(u32::MAX));
    }
    let partition = compiled.clone().partition(None).unwrap();
    assert_eq!(partition.parishes, compiled.parishes);
    assert!(!partition.parishes.is_empty());
    assert!(partition.playable.is_empty());
    for distant in partition.distant {
        let original = compiled
            .buildings
            .iter()
            .find(|b| b.id == distant.id)
            .unwrap();
        assert_eq!(distant.program(), original.program);
    }
    let partition = compiled.clone().partition(Some(50.0)).unwrap();
    for compound in compiled.compounds {
        let front = partition
            .playable
            .iter()
            .any(|b| b.id == compound.front_building_id);
        let rear = partition
            .playable
            .iter()
            .any(|b| b.id == compound.rear_building_id);
        assert_eq!(
            front, rear,
            "a compound must not split across simulation authority"
        );
        if compound
            .plot
            .corners()
            .iter()
            .any(|p| p.abs().max_element() < 50.0)
        {
            assert!(
                front,
                "intersecting property must retain authoritative collision"
            );
        }
    }
}
