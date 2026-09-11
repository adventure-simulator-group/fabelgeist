//! Semantic room views selected from proven walking routes and unobstructed sightlines.
use super::*;
use adventuresim_building_generator::{
    BuildingPlan, CollisionCuboid, OpeningUse, Room, RoomKind, SolidRole, StoreyPlan,
    interior::{InteriorLayout, InteriorPlacement, furniture_floor_height},
};
use sightlines::{Blocker, Owner, Subject};

mod sightlines;

#[derive(Clone, Copy)]
pub(super) enum RoomSelection {
    Role(RoomKind),
    UpperKeep,
    Bedroom,
}

pub(super) fn camera(
    building: &GeneratedBuilding,
    furniture: &FurnitureLayout,
    selection: RoomSelection,
) -> BuildingReviewCamera {
    let layout = &furniture
        .interiors
        .iter()
        .find(|interior| interior.building_id == building.placement.id)
        .expect("review building has furnishing proof")
        .layout;
    let (level, floor, room) = match selection {
        RoomSelection::UpperKeep => {
            let landing = building
                .plan
                .stairs
                .iter()
                .enumerate()
                .flat_map(|(index, _)| {
                    adventuresim_building_generator::spiral_stairs::landings(&building.plan, index)
                })
                .filter(|landing| landing.storey > 0)
                .min_by_key(|landing| landing.storey)
                .expect("keep review requires a real upper spiral landing");
            (landing.storey, landing.elevation_metres, None)
        }
        _ => {
            let (storey, room) = select_room(&building.plan.storeys, layout, selection);
            let placement = layout
                .placements
                .iter()
                .find(|placement| placement.storey == storey.level && placement.room_id == room.id)
                .expect("selected room has furniture");
            (
                storey.level,
                furniture_floor_height(&building.plan, placement),
                Some(room),
            )
        }
    };
    let blockers = blockers(building, layout);
    let mut subjects: Vec<_> = layout
        .placements
        .iter()
        .enumerate()
        .filter(|(_, placement)| {
            placement.storey == level && room.is_none_or(|room| placement.room_id == room.id)
        })
        .map(|(index, placement)| furniture_subject(building, placement, index, selection))
        .collect();
    if matches!(selection, RoomSelection::UpperKeep) {
        add_stair_subjects(building, floor, &mut subjects);
    }
    let mut eyes: Vec<_> = layout
        .paths
        .iter()
        .flat_map(|path| &path.points)
        .filter(|point| {
            point.storey == level
                && room.is_none_or(|room| room_contains(room, point.position_metres))
        })
        .map(|point| {
            Vec3::new(
                point.position_metres.x,
                floor + CAMERA_EYE_HEIGHT_METRES,
                point.position_metres.y,
            )
        })
        .collect();
    eyes.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.z.total_cmp(&b.z)));
    eyes.dedup();
    let (eye, target) = sightlines::choose(&eyes, &subjects, &blockers).expect(
        "review room must have a reachable unobstructed view of its signature furniture or stair",
    );
    let transform = super::super::buildings::building_transform(building);
    let origin = building.collision.bounds.centre();
    BuildingReviewCamera {
        position: transform.transform_point(eye - origin),
        target: transform.transform_point(target - origin),
        plaster_raking_light: None,
    }
}

fn select_room<'a>(
    storeys: &'a [StoreyPlan],
    layout: &InteriorLayout,
    selection: RoomSelection,
) -> (&'a StoreyPlan, &'a Room) {
    storeys
        .iter()
        .flat_map(|storey| storey.rooms.iter().map(move |room| (storey, room)))
        .filter(|(storey, room)| match selection {
            RoomSelection::Role(kind) => room.kind == kind,
            RoomSelection::Bedroom => storey.level > 0 && room.kind == RoomKind::Bedchamber,
            RoomSelection::UpperKeep => false,
        })
        .filter_map(|(storey, room)| {
            let population = layout
                .placements
                .iter()
                .filter(|placement| {
                    placement.storey == storey.level && placement.room_id == room.id
                })
                .count();
            (population > 0).then_some((population, storey, room))
        })
        .max_by_key(|(population, _, _)| *population)
        .map(|(_, storey, room)| (storey, room))
        .expect("review fixture must provide furniture in the requested semantic room")
}

fn blockers(building: &GeneratedBuilding, layout: &InteriorLayout) -> Vec<Blocker> {
    let mut result: Vec<_> = building
        .collision
        .cuboids
        .iter()
        .map(|solid| Blocker {
            owner: Owner::Architecture(solid.source),
            solid: *solid,
        })
        .collect();
    result.extend(closed_door_blockers(&building.plan));
    for (index, placement) in layout.placements.iter().enumerate() {
        let rotation = Quat::from_rotation_y(placement.yaw_radians());
        let translation = Vec3::new(
            placement.centre_metres.x,
            furniture_floor_height(&building.plan, placement),
            placement.centre_metres.y,
        );
        result.extend(placement.key.recipe().colliders.iter().map(|solid| {
            let mut solid = *solid;
            solid.centre = translation + rotation * solid.centre;
            solid.yaw_radians += placement.yaw_radians();
            Blocker {
                owner: Owner::Furniture(index),
                solid,
            }
        }));
    }
    result
}

/// Interior capture fixtures keep authored leaves closed, including internal
/// doors omitted from static physics so navigation can pass through openings.
fn closed_door_blockers(plan: &BuildingPlan) -> Vec<Blocker> {
    let door_solids = plan
        .opening_assemblies
        .iter()
        .filter(|opening| opening.use_kind == OpeningUse::Door)
        .flat_map(|opening| opening.closure_solids.iter().copied())
        .collect::<std::collections::BTreeSet<_>>();
    plan.resolved_geometry
        .solids
        .iter()
        .filter(|solid| door_solids.contains(&solid.id))
        .map(|solid| Blocker {
            owner: Owner::Architecture(solid.id),
            solid: CollisionCuboid {
                source: solid.id,
                centre: solid.centre,
                size: solid.size,
                yaw_radians: solid.yaw_radians,
                crossfall_radians: solid.crossfall_radians,
                longfall_radians: solid.longfall_radians,
            },
        })
        .collect()
}

fn furniture_subject(
    building: &GeneratedBuilding,
    placement: &InteriorPlacement,
    index: usize,
    selection: RoomSelection,
) -> Subject {
    let size = placement.key.interior_spec().unwrap().size_metres;
    let rotation = Quat::from_rotation_y(placement.yaw_radians());
    let translation = Vec3::new(
        placement.centre_metres.x,
        furniture_floor_height(&building.plan, placement),
        placement.centre_metres.y,
    );
    let points = [
        Vec3::new(0.0, size.y * 0.65, 0.0),
        Vec3::new(-size.x * 0.4, size.y * 0.65, -size.z * 0.4),
        Vec3::new(size.x * 0.4, size.y * 0.65, -size.z * 0.4),
        Vec3::new(-size.x * 0.4, size.y * 0.65, size.z * 0.4),
        Vec3::new(size.x * 0.4, size.y * 0.65, size.z * 0.4),
    ]
    .map(|point| translation + rotation * point)
    .to_vec();
    let signature = signature_kind(selection, placement.key.kind);
    Subject {
        owner: Owner::Furniture(index),
        points,
        importance: if signature { 4.0 } else { 1.0 },
        signature,
    }
}

fn signature_kind(selection: RoomSelection, kind: FurnitureKind) -> bool {
    use FurnitureKind::*;
    match selection {
        RoomSelection::Role(RoomKind::Shop) => matches!(
            kind,
            Counter | CounterLeftEnd | CounterRightEnd | CounterCorner | DisplayCounter
        ),
        RoomSelection::Role(RoomKind::CommonRoom | RoomKind::GreatHall) => matches!(
            kind,
            DiningTable | Counter | CounterLeftEnd | CounterRightEnd
        ),
        RoomSelection::Role(RoomKind::Ward) => kind == WardBed,
        RoomSelection::Role(RoomKind::Guardroom) => matches!(kind, BunkBed | WeaponRack),
        RoomSelection::Role(RoomKind::Workshop) => kind == Workbench,
        RoomSelection::Role(RoomKind::Nave) => kind == ChurchBench,
        RoomSelection::Role(RoomKind::Storage) => matches!(kind, StorageCrate | GrainBin),
        RoomSelection::Bedroom => kind == Bed,
        _ => false,
    }
}

fn add_stair_subjects(building: &GeneratedBuilding, floor: f32, subjects: &mut Vec<Subject>) {
    let treads: Vec<_> = building
        .plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| solid.role == SolidRole::StairTread)
        .filter(|solid| {
            let top = solid.centre.y + solid.size.y * 0.5;
            top <= floor + 0.02 && top >= floor - 0.8
        })
        .collect();
    assert!(
        !treads.is_empty(),
        "upper keep review requires actual stair treads beside its landing"
    );
    for tread in treads {
        let rotation = Quat::from_rotation_y(tread.yaw_radians);
        let top = tread.centre + Vec3::Y * (tread.size.y * 0.5 + 0.02);
        subjects.push(Subject {
            owner: Owner::Architecture(tread.id),
            points: vec![
                top,
                top + rotation * Vec3::X * tread.size.x * 0.25,
                top - rotation * Vec3::X * tread.size.x * 0.25,
            ],
            importance: 6.0,
            signature: true,
        });
    }
}

fn room_contains(room: &Room, point: Vec2) -> bool {
    room.cells.iter().any(|cell| {
        (point - cell.centre()).abs().max_element()
            <= adventuresim_building_generator::CELL_SIZE_METRES * 0.5
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_keep_door_blocks_a_camera_ray_that_static_navigation_allows() {
        use adventuresim_building_generator::{
            BuildingArchetype, BuildingProgram, compile_building_collision, generate,
        };
        let program = BuildingProgram::settlement(
            BuildingArchetype::WalledKeep,
            Some(BuildingUse::Castle),
            42,
        );
        let plan = generate(&program).unwrap();
        let collision = compile_building_collision(&plan);
        let mut blockers: Vec<_> = collision
            .cuboids
            .iter()
            .map(|solid| Blocker {
                owner: Owner::Architecture(solid.source),
                solid: *solid,
            })
            .collect();
        // Exact failed keep-upper capture ray, expressed in the generated plan's frame.
        let eye = Vec3::new(3.25, 4.95, 6.75);
        let target = Vec3::new(6.548733, 2.697778, 6.68088);
        let subjects = [Subject {
            owner: Owner::Furniture(0),
            points: vec![target],
            importance: 1.0,
            signature: true,
        }];
        assert!(sightlines::choose(&[eye], &subjects, &blockers).is_some());
        let doors = closed_door_blockers(&plan);
        assert!(
            doors
                .iter()
                .any(|door| door.solid.source.0 == 1153050796007358484)
        );
        blockers.extend(doors);
        assert!(
            sightlines::choose(&[eye], &subjects, &blockers).is_none(),
            "a route through a door does not make its closed rendered leaf transparent"
        );
    }

    #[test]
    fn semantic_room_selection_does_not_choose_a_denser_shop_for_a_house_view() {
        let storeys = [
            StoreyPlan {
                level: 0,
                rooms: vec![Room {
                    id: 0,
                    kind: RoomKind::Shop,
                    cells: vec![],
                }],
                walls: vec![],
                openings: vec![],
            },
            StoreyPlan {
                level: 1,
                rooms: vec![Room {
                    id: 0,
                    kind: RoomKind::CommonRoom,
                    cells: vec![],
                }],
                walls: vec![],
                openings: vec![],
            },
        ];
        let mut layout = InteriorLayout::default();
        for storey in [0, 0, 0, 1] {
            layout.placements.push(InteriorPlacement {
                key: FurnitureKey {
                    kind: FurnitureKind::DiningTable,
                    variant: FurnitureVariant::Compact,
                },
                room_id: 0,
                storey,
                centre_metres: Vec2::ZERO,
                facing: adventuresim_building_generator::Direction::South,
            });
        }
        let (storey, room) =
            select_room(&storeys, &layout, RoomSelection::Role(RoomKind::CommonRoom));
        assert_eq!(storey.level, 1);
        assert_eq!(room.kind, RoomKind::CommonRoom);
    }

    #[test]
    fn semantic_room_signatures_prioritize_work_and_living_furniture_over_storage() {
        assert!(signature_kind(
            RoomSelection::Role(RoomKind::Shop),
            FurnitureKind::CounterLeftEnd
        ));
        assert!(!signature_kind(
            RoomSelection::Role(RoomKind::Shop),
            FurnitureKind::Shelving
        ));
        assert!(signature_kind(
            RoomSelection::Role(RoomKind::CommonRoom),
            FurnitureKind::DiningTable
        ));
        assert!(!signature_kind(
            RoomSelection::Role(RoomKind::CommonRoom),
            FurnitureKind::StorageChest
        ));
        assert!(signature_kind(
            RoomSelection::Role(RoomKind::Ward),
            FurnitureKind::WardBed
        ));
        assert!(signature_kind(
            RoomSelection::Role(RoomKind::Guardroom),
            FurnitureKind::BunkBed
        ));
    }
}
