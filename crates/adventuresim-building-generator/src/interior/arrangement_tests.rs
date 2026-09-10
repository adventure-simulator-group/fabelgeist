//! Service rooms must read as deliberate ensembles while preserving access proofs.
use super::*;
use crate::furniture::FurnitureKind;
use crate::{BuildingProgram, RoomKind, ServiceBuildingSize, generate, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

#[test]
fn interior_ward_and_inn_arrangements_preserve_service_and_access() {
    for seed in [42, 47, 101] {
        for size in [None, Some(ServiceBuildingSize::Medium)] {
            for usage in [BuildingUse::Inn, BuildingUse::Hospital] {
                let program = BuildingProgram::validated_settlement(
                    settlement_archetype(usage),
                    usage,
                    seed,
                    size,
                )
                .unwrap();
                let plan = generate(&program).unwrap();
                let layout = furnish(&plan, &program).unwrap();
                if usage == BuildingUse::Inn {
                    assert!(
                        plan.storeys
                            .iter()
                            .any(|storey| storey.rooms.iter().any(|room| {
                                if room.kind != RoomKind::CommonRoom {
                                    return false;
                                }
                                let pieces = layout
                                    .placements
                                    .iter()
                                    .filter(|p| p.storey == storey.level && p.room_id == room.id)
                                    .collect::<Vec<_>>();
                                pieces.iter().any(|p| p.key.kind == FurnitureKind::Counter)
                                    && pieces
                                        .iter()
                                        .any(|p| p.key.kind == FurnitureKind::DiningTable)
                            })),
                        "{seed} {size:?}: no viable common room has dining and service counter"
                    );
                } else {
                    for room in plan.storeys[0]
                        .rooms
                        .iter()
                        .filter(|r| r.kind == RoomKind::Ward)
                    {
                        let beds = layout
                            .placements
                            .iter()
                            .filter(|p| {
                                p.storey == 0
                                    && p.room_id == room.id
                                    && p.key.kind == FurnitureKind::WardBed
                            })
                            .collect::<Vec<_>>();
                        assert!(beds.len() >= 4, "{seed} {size:?}: too few ward beds");
                        let axis = beds[0].facing.offset().as_vec2().abs();
                        assert!(
                            beds.iter()
                                .all(|p| p.facing.offset().as_vec2().abs() == axis)
                        );
                    }
                }
                validate_layout(&plan, &layout).unwrap();
                eprintln!(
                    "{usage:?} {seed} {size:?}: {} pieces",
                    layout.placements.len()
                );
            }
        }
    }
}

#[test]
fn interior_narrow_inn_ground_room_cannot_combine_full_counter_and_dining_group() {
    let program = BuildingProgram::validated_settlement(
        settlement_archetype(BuildingUse::Inn),
        BuildingUse::Inn,
        42,
        None,
    )
    .unwrap();
    let plan = generate(&program).unwrap();
    let nav = super::navigation::Navigation::new(&plan).unwrap();
    let room = plan.storeys[0]
        .rooms
        .iter()
        .find(|r| r.kind == RoomKind::CommonRoom)
        .unwrap();
    let eligible = |kind, position| {
        super::placement::candidates(
            &plan,
            &program,
            room,
            0,
            FurnitureBudget {
                kind,
                count: 1,
                position,
            },
        )
        .into_iter()
        .filter(|group| super::placement::footprints_valid(&plan, &nav, group).is_ok())
        .collect::<Vec<_>>()
    };
    let tables = eligible(FurnitureKind::DiningTable, FurniturePosition::Centre);
    let counters = eligible(FurnitureKind::Counter, FurniturePosition::CounterRun);
    assert!(!tables.is_empty() && !counters.is_empty());
    for table in &tables {
        for counter in &counters {
            let mut pair = table.clone();
            pair.extend_from_slice(counter);
            assert!(super::placement::footprints_valid(&plan, &nav, &pair).is_err());
        }
    }
    let layout = furnish(&plan, &program).unwrap();
    assert!(layout.unmet_budgets.iter().any(|b| b.storey == 0
        && b.room_id == room.id
        && b.kind == FurnitureKind::DiningTable
        && b.placed == 0));
    validate_layout(&plan, &layout).unwrap();
}
