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
    for hinge in [PropertySide::Left, PropertySide::Right] {
        assert!(
            compiled
                .compounds
                .iter()
                .any(|property| property.boundary.gate.hinge == hinge),
            "production packing must retain both passage orientations"
        );
    }
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

#[test]
fn business_keys_survive_compilation_and_playable_partitioning() {
    let city =
        CitySite::central_german_market_town().generate(42, 6_500, &super::super::tests::economy());
    let expected = city
        .lots
        .iter()
        .filter_map(|lot| {
            lot.service
                .and_then(BuildingDemand::business_key)
                .map(|key| (lot.id, key))
        })
        .collect::<Vec<_>>();
    let compiled = city.compile(42).unwrap();
    assert_eq!(
        compiled
            .businesses
            .iter()
            .map(|site| (site.building_id, site.key))
            .collect::<Vec<_>>(),
        expected
    );
    for extent in [None, Some(35.0), Some(90.0)] {
        assert_eq!(
            compiled.clone().partition(extent).unwrap().businesses,
            compiled.businesses
        );
    }
}

#[test]
fn gardens_are_owned_connected_and_preserve_accepted_plants_across_partition() {
    use crate::city_layout::gardens::GardenIssue;
    let city =
        CitySite::central_german_market_town().generate(42, 900, &super::super::tests::economy());
    let compiled = city.compile(42).unwrap();
    assert!(
        !compiled.gardens.is_empty(),
        "fixture needs an accepted owned garden"
    );
    assert!(
        compiled
            .yards
            .iter()
            .filter(|yard| yard.surface == CityYardSurface::KitchenGarden)
            .all(|yard| compiled.gardens.iter().any(|garden| garden
                .beds
                .iter()
                .any(|bed| bed.corners() == yard.corners_metres)))
    );
    let mut owners = BTreeSet::new();
    for garden in &compiled.gardens {
        assert!(owners.insert(garden.owner));
        assert!(
            compiled
                .buildings
                .iter()
                .any(|building| building.id == garden.front_building_id)
        );
        assert_eq!(garden.validate_geometry(&compiled.streets), Ok(()));
        assert!(!garden.plants.is_empty());
        let mut escaped = garden.clone();
        escaped.plants[0].centre_metres += Vec2::splat(100.0);
        assert_eq!(
            escaped.validate_geometry(&compiled.streets),
            Err(GardenIssue::PlantOutsidePlot)
        );
        let mut blocked = garden.clone();
        blocked.beds[0].centre_metres =
            (blocked.access[1].start_metres + blocked.access[1].end_metres) * 0.5;
        assert!(blocked.validate_geometry(&compiled.streets).is_err());
        let mut disconnected = garden.clone();
        disconnected.access[2].start_metres += Vec2::splat(100.0);
        disconnected.access[2].end_metres += Vec2::splat(100.0);
        assert_eq!(
            disconnected.validate_geometry(&compiled.streets),
            Err(GardenIssue::DisconnectedTendingLane)
        );
    }
    for extent in [None, Some(96.0)] {
        assert_eq!(
            compiled.clone().partition(extent).unwrap().gardens,
            compiled.gardens
        );
    }
}

#[test]
fn garden_crossing_playable_edge_promotes_its_owner_without_changing_plants() {
    let mut compiled = CitySite::central_german_market_town()
        .generate(42, 900, &super::super::tests::economy())
        .compile(42)
        .unwrap();
    let (garden, extent) = compiled
        .gardens
        .iter()
        .find_map(|g| {
            let centre = compiled
                .buildings
                .iter()
                .find(|b| b.id == g.front_building_id)?
                .centre_metres
                .abs()
                .max_element();
            let inner = g
                .plot
                .corners()
                .into_iter()
                .map(|p| p.abs().max_element())
                .min_by(f32::total_cmp)?;
            (inner < centre).then_some((g, (inner + centre) * 0.5))
        })
        .expect("generated garden crossing precedes its owner centre");
    let owner = garden.front_building_id;
    // Isolate this complete accepted property from the unrelated city capacity.
    compiled.buildings.retain(|b| b.id == owner);
    compiled.gardens.retain(|g| g.front_building_id == owner);
    compiled.compounds.clear();
    compiled.parishes.clear();
    let result = compiled.clone().partition(Some(extent)).unwrap();
    assert!(result.playable.iter().any(|b| b.id == owner));
    assert_eq!(result.gardens, compiled.gardens);
}
