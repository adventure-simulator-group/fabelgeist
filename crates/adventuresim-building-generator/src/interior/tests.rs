use super::*;
use crate::furniture::{FurnitureKind, FurnitureVariant};
use crate::{BuildingPlan, BuildingProgram, Cell, Room, RoomKind, generate, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

fn building(usage: BuildingUse) -> (BuildingProgram, BuildingPlan) {
    let program =
        BuildingProgram::validated_settlement(settlement_archetype(usage), usage, 42, None)
            .unwrap();
    let plan = generate(&program).unwrap_or_else(|error| panic!("{usage:?}: {error}"));
    (program, plan)
}

#[test]
fn interior_representative_buildings_have_usable_furniture() {
    for (usage, expected) in [
        (BuildingUse::Dwelling, FurnitureKind::Bed),
        (BuildingUse::Inn, FurnitureKind::Counter),
        (BuildingUse::Hospital, FurnitureKind::WardBed),
        (BuildingUse::Bathhouse, FurnitureKind::BathTub),
        (BuildingUse::GeneralShop, FurnitureKind::Counter),
        (BuildingUse::ParishChurch, FurnitureKind::ChurchBench),
        (BuildingUse::Smithy, FurnitureKind::Workbench),
    ] {
        let (program, plan) = building(usage);
        let layout = furnish(&plan, &program).unwrap_or_else(|e| panic!("{usage:?}: {e:?}"));
        assert!(
            layout.placements.iter().any(|p| p.key.kind == expected),
            "{usage:?} missing {expected:?}: {:?}",
            layout.unmet_budgets
        );
        let paths = validate_layout(&plan, &layout).unwrap();
        assert_eq!(
            paths.len(),
            layout
                .placements
                .iter()
                .map(|p| p.key.interior_spec().unwrap().required_faces.len())
                .sum::<usize>()
        );
        assert!(paths.iter().all(|p| p.points.first().unwrap().storey == 0));
        eprintln!(
            "{usage:?}: {} furniture, {} paths, {} unmet budgets",
            layout.placements.len(),
            paths.len(),
            layout.unmet_budgets.len()
        );
    }
}

#[test]
fn interior_hospital_front_door_is_clear() {
    let (_, plan) = building(BuildingUse::Hospital);
    super::navigation::Navigation::new(&plan).unwrap();
}

#[test]
fn interior_cathedral_rooms_share_continuous_paving_and_clear_doors() {
    let program = BuildingProgram::settlement(
        settlement_archetype(BuildingUse::Cathedral),
        Some(BuildingUse::Cathedral),
        42,
    )
    .with_service_size(crate::ServiceBuildingSize::Medium);
    let plan = generate(&program).unwrap();
    let layout = furnish(&plan, &program).unwrap();
    for (kind, role) in [
        (FurnitureKind::ChurchBench, RoomKind::Nave),
        (FurnitureKind::Altar, RoomKind::Chancel),
    ] {
        assert!(
            layout.placements.iter().any(|p| p.key.kind == kind
                && plan.storeys[0]
                    .rooms
                    .iter()
                    .any(|room| room.id == p.room_id && room.kind == role)),
            "missing {kind:?} in {role:?}: {:?}",
            layout.unmet_budgets
        );
    }
    let floor = super::architecture::Floor::new(&plan, 0);
    for door in plan
        .opening_assemblies
        .iter()
        .filter(|d| d.use_kind == crate::OpeningUse::Door)
    {
        assert!(floor.reserved.iter().any(|r| r.contains(door.frame.origin)));
        for id in [door.frame.inside_room, door.frame.outside_room]
            .into_iter()
            .flatten()
        {
            assert!(plan.storeys[0].rooms.iter().any(|room| room.id == id));
        }
    }
    validate_layout(&plan, &layout).unwrap();
}

#[test]
fn interior_woad_store_door_approaches_clear_structural_posts() {
    let program = BuildingProgram::validated_settlement(
        settlement_archetype(BuildingUse::WoadStore),
        BuildingUse::WoadStore,
        42,
        Some(crate::ServiceBuildingSize::Medium),
    )
    .unwrap();
    let plan = generate(&program).unwrap();
    super::navigation::Navigation::new(&plan).unwrap();
}

#[test]
fn interior_every_use_has_an_exhaustive_room_budget() {
    let room = Room {
        id: 0,
        kind: RoomKind::Workshop,
        cells: (0..8)
            .flat_map(|x| (0..8).map(move |z| Cell::new(x, z)))
            .collect(),
    };
    for usage in BuildingUse::ALL {
        let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42);
        let budgets = furniture_budgets(&program, &room);
        assert!(!budgets.is_empty(), "{usage:?}");
        assert!(budgets.iter().all(|b| b.count > 0));
    }
}

#[test]
fn interior_all_settlement_uses_generate_accessible_layouts() {
    let mut failures = Vec::new();
    for usage in BuildingUse::ALL {
        let (program, plan) = building(usage);
        match furnish(&plan, &program) {
            Ok(layout) => {
                assert!(!layout.placements.is_empty());
                eprintln!("{usage:?}: {} placements", layout.placements.len());
            }
            Err(error) => failures.push((usage, error)),
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn interior_civilian_seed_matrix() {
    for seed in [42, 47, 101] {
        for usage in BuildingUse::ALL
            .into_iter()
            .filter(|u| !matches!(u, BuildingUse::Castle | BuildingUse::Arsenal))
        {
            let program = BuildingProgram::validated_settlement(
                settlement_archetype(usage),
                usage,
                seed,
                Some(crate::ServiceBuildingSize::Medium),
            )
            .unwrap_or_else(|error| panic!("{usage:?} seed {seed}: {error}"));
            let plan =
                generate(&program).unwrap_or_else(|error| panic!("{usage:?} seed {seed}: {error}"));
            let layout = furnish(&plan, &program)
                .unwrap_or_else(|error| panic!("{usage:?} seed {seed}: {error}"));
            assert!(!layout.placements.is_empty());
        }
        eprintln!("64 civilian uses accepted at seed {seed}");
    }
}

#[test]
fn interior_civilian_sizes_and_fixture_programs() {
    for usage in [
        BuildingUse::Dwelling,
        BuildingUse::Inn,
        BuildingUse::GeneralShop,
        BuildingUse::Hospital,
        BuildingUse::ParishChurch,
        BuildingUse::Smithy,
    ] {
        for size in [
            crate::ServiceBuildingSize::Small,
            crate::ServiceBuildingSize::Large,
        ] {
            let program = BuildingProgram::validated_settlement(
                settlement_archetype(usage),
                usage,
                42,
                Some(size),
            )
            .unwrap_or_else(|error| panic!("{usage:?} {size:?}: {error}"));
            let plan = generate(&program).unwrap();
            furnish(&plan, &program).unwrap_or_else(|error| panic!("{usage:?} {size:?}: {error}"));
        }
    }
    for archetype in [
        crate::BuildingArchetype::TownHouse,
        crate::BuildingArchetype::HallHouse,
        crate::BuildingArchetype::FachwerkCottage,
        crate::BuildingArchetype::FachwerkMerchantHouse,
        crate::BuildingArchetype::RenaissanceTownHall,
        crate::BuildingArchetype::Cathedral,
        crate::BuildingArchetype::ParishChurch,
    ] {
        let program = BuildingProgram::fixture(archetype, 42);
        let plan = generate(&program).unwrap_or_else(|error| panic!("{archetype:?}: {error}"));
        furnish(&plan, &program).unwrap_or_else(|error| panic!("{archetype:?}: {error}"));
    }
}

#[test]
fn interior_rejects_obstructed_door_and_overlapping_furniture() {
    let (program, plan) = building(BuildingUse::Dwelling);
    let mut layout = furnish(&plan, &program).unwrap();
    let entrance = plan
        .opening_assemblies
        .iter()
        .find(|o| o.use_kind == crate::OpeningUse::Door && o.frame.outside_room.is_none())
        .unwrap();
    layout.placements.push(InteriorPlacement {
        key: crate::furniture::FurnitureKey {
            kind: FurnitureKind::StorageCrate,
            variant: FurnitureVariant::Compact,
        },
        room_id: entrance.frame.inside_room.unwrap(),
        storey: 0,
        centre_metres: entrance.frame.origin - entrance.frame.outward * 0.5,
        facing: Direction::South,
    });
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::InvalidPlacement { .. })
    ));
    layout.placements.pop();
    layout.placements.push(layout.placements[0].clone());
    assert!(validate_layout(&plan, &layout).is_err());
}

#[test]
fn interior_layout_is_deterministic_and_paths_use_stairs() {
    let (program, plan) = building(BuildingUse::Dwelling);
    let a = furnish(&plan, &program).unwrap();
    let b = furnish(&plan, &program).unwrap();
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
    assert!(a.placements.iter().any(|p| p.storey > 0));
    assert!(
        a.paths
            .iter()
            .any(|p| p.points.windows(2).any(|s| s[0].storey != s[1].storey))
    );
}

#[test]
fn interior_jetty_beams_bear_below_the_finished_floor() {
    let (_, plan) = building(BuildingUse::Dwelling);
    let frame = plan.timber_frame.as_ref().unwrap();
    let mut checked = 0;
    for storey in frame
        .facades
        .iter()
        .flat_map(|f| &f.lines)
        .flat_map(|l| &l.storeys)
    {
        let Some(jetty) = &storey.jetty else {
            continue;
        };
        let floor = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == jetty.floor_solid)
            .unwrap();
        let underside = floor.centre.y - floor.size.y * 0.5;
        for beam in &jetty.jetty_beams {
            let member = frame.members.iter().find(|m| m.id == *beam).unwrap();
            assert!(member.start.y < underside && member.end.y < underside);
        }
        for id in &jetty.floor_bearing_interfaces {
            let bearing = plan
                .resolved_geometry
                .support_interfaces
                .iter()
                .find(|i| i.id == *id)
                .unwrap();
            assert!(bearing.bounds.min.y <= underside && bearing.bounds.max.y >= underside);
        }
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn interior_counter_modules_are_contiguous_with_two_sided_access() {
    let (program, plan) = building(BuildingUse::GeneralShop);
    let layout = furnish(&plan, &program).unwrap();
    let left = layout
        .placements
        .iter()
        .find(|p| p.key.kind == FurnitureKind::CounterLeftEnd)
        .unwrap();
    let centre = layout
        .placements
        .iter()
        .find(|p| p.key.kind == FurnitureKind::Counter)
        .unwrap();
    let right = layout
        .placements
        .iter()
        .find(|p| p.key.kind == FurnitureKind::CounterRightEnd)
        .unwrap();
    let width = centre.key.interior_spec().unwrap().size_metres.x;
    assert!((left.centre_metres.distance(centre.centre_metres) - width).abs() < 0.001);
    assert!((right.centre_metres.distance(centre.centre_metres) - width).abs() < 0.001);
    assert_eq!(left.facing, centre.facing);
    assert_eq!(right.facing, centre.facing);
    validate_layout(&plan, &layout).unwrap();
}
