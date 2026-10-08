//! Geometry mutations exercise proof failures independently of furniture meshes.
use super::*;
use crate::furniture::{FurnitureKind, FurnitureVariant};
use crate::{
    BuildingArchetype, BuildingPlan, BuildingProgram, Cell, ResolvedItemId, Room, RoomKind,
    SolidRole,
};
use bevy::math::Vec3;

fn empty_room() -> BuildingPlan {
    let mut plan = crate::generate(&BuildingProgram::fixture(
        BuildingArchetype::TownHouse,
        fabelgeist_determinism::Seed::from_u64(42),
    ))
    .unwrap();
    plan.storeys.truncate(1);
    plan.storeys[0].rooms = vec![Room {
        id: 0,
        kind: RoomKind::Storage,
        cells: (0..6)
            .flat_map(|x| (0..10).map(move |z| Cell::new(x, z)))
            .collect(),
    }];
    plan.storeys[0].walls.clear();
    plan.storeys[0].openings.clear();
    plan.wall_assemblies.clear();
    plan.stairs.clear();
    plan.opening_assemblies.retain(|o| {
        o.use_kind == crate::OpeningUse::Door
            && o.frame.outside_room.is_none()
            && o.sill_elevation_metres == 0.0
    });
    for door in &mut plan.opening_assemblies {
        door.frame.inside_room = Some(0);
    }
    plan.resolved_geometry.solids.retain(|s| {
        s.role == SolidRole::FrameFloor
            && (s.centre.metres().y + s.size.metres().y * 0.5).abs() < 0.001
    });
    plan
}
fn placement(kind: FurnitureKind, variant: FurnitureVariant) -> InteriorPlacement {
    InteriorPlacement {
        key: FurnitureKey::natural(kind, variant),
        room_id: crate::RoomIndex::from_serialized(0),
        storey: crate::StoreyIndex::from_serialized(0),
        centre_metres: crate::plan_geometry::ArchitecturalPlanPoint::try_from(Vec2::new(4.5, 7.5))
            .unwrap(),
        facing: Direction::South,
    }
}
fn add_solid(plan: &mut BuildingPlan, centre: Vec3, size: Vec3) {
    let mut solid = plan.resolved_geometry.solids[0].clone();
    solid.id = ResolvedItemId(9_000_000 + plan.resolved_geometry.solids.len() as u64);
    solid.centre =
        crate::spatial_geometry::Position::<crate::Architectural>::from_metres(centre).unwrap();
    solid.size = crate::spatial_geometry::CuboidDimensions::from_metres(size).unwrap();
    solid.role = SolidRole::FramePost;
    plan.resolved_geometry.solids.push(solid);
}

#[test]
fn interior_rejects_clear_usable_face_inside_a_sealed_enclosure() {
    let mut plan = empty_room();
    let layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::Shelving,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    assert!(validate_layout(&plan, &layout).is_ok());
    for (centre, size) in [
        (Vec3::new(3.0, 1.2, 7.0), Vec3::new(0.1, 2.4, 4.0)),
        (Vec3::new(6.0, 1.2, 7.0), Vec3::new(0.1, 2.4, 4.0)),
        (Vec3::new(4.5, 1.2, 5.0), Vec3::new(3.0, 2.4, 0.1)),
        (Vec3::new(4.5, 1.2, 9.0), Vec3::new(3.0, 2.4, 0.1)),
    ] {
        add_solid(&mut plan, centre, size);
    }
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::InaccessibleFurniture { index: 0 })
    ));
}

#[test]
fn interior_does_not_cut_a_diagonal_gap_between_touching_obstacles() {
    let mut plan = empty_room();
    let layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::Shelving,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    // Two halves meet at a single corner. A point-agent diagonal flood could cross it.
    add_solid(
        &mut plan,
        Vec3::new(2.25, 1.2, 5.0),
        Vec3::new(4.5, 2.4, 0.1),
    );
    add_solid(
        &mut plan,
        Vec3::new(6.75, 1.2, 5.1),
        Vec3::new(4.5, 2.4, 0.1),
    );
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::InaccessibleFurniture { index: 0 })
    ));
}

#[test]
fn interior_checks_tall_furniture_above_person_head_height() {
    let mut plan = empty_room();
    add_solid(
        &mut plan,
        Vec3::new(4.5, 1.92, 7.5),
        Vec3::new(2.0, 0.2, 1.0),
    );
    let mut layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::Cupboard,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    validate_layout(&plan, &layout).unwrap();
    layout.placements[0].key =
        FurnitureKey::natural(layout.placements[0].key.kind(), FurnitureVariant::Broad);
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::InvalidPlacement { index: 0 })
    ));
}

#[test]
fn interior_routes_start_at_the_door_and_reject_a_blocked_threshold() {
    let mut plan = empty_room();
    let layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::Shelving,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    let door = plan.opening_assemblies[0].frame.origin;
    let paths = validate_layout(&plan, &layout).unwrap();
    assert_eq!(paths[0].points[0].position_metres.metres(), door);
    // The inside approach remains free, but the actual threshold is sealed.
    add_solid(
        &mut plan,
        Vec3::new(door.x, 1.2, door.y),
        Vec3::new(0.2, 2.4, 0.2),
    );
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::MissingFrontDoor)
    ));
}

#[test]
fn interior_requires_floor_under_the_whole_footprint() {
    let mut plan = empty_room();
    let layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::DiningTable,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    validate_layout(&plan, &layout).unwrap();
    let source = plan.resolved_geometry.solids[0].clone();
    plan.resolved_geometry.solids.clear();
    // Remove only a central floor patch, leaving every object corner supported.
    for (index, (centre, size)) in [
        (Vec3::new(2.175, -0.08, 7.5), Vec3::new(4.05, 0.16, 14.7)),
        (Vec3::new(6.825, -0.08, 7.5), Vec3::new(4.05, 0.16, 14.7)),
        (Vec3::new(4.5, -0.08, 3.675), Vec3::new(0.6, 0.16, 7.05)),
        (Vec3::new(4.5, -0.08, 11.325), Vec3::new(0.6, 0.16, 7.05)),
    ]
    .into_iter()
    .enumerate()
    {
        let mut floor = source.clone();
        floor.id = ResolvedItemId(8_000_000 + index as u64);
        floor.centre =
            crate::spatial_geometry::Position::<crate::Architectural>::from_metres(centre).unwrap();
        floor.size = crate::spatial_geometry::CuboidDimensions::from_metres(size).unwrap();
        plan.resolved_geometry.solids.push(floor);
    }
    assert!(matches!(
        validate_layout(&plan, &layout),
        Err(InteriorLayoutError::InvalidPlacement { index: 0 })
    ));
}

#[test]
fn interior_floor_support_follows_rotated_landing_footprint() {
    let mut plan = empty_room();
    plan.resolved_geometry.solids.truncate(1);
    let landing = &mut plan.resolved_geometry.solids[0];
    landing.centre = crate::spatial_geometry::Position::<crate::Architectural>::from_metres(
        Vec3::new(4.5, -0.08, 7.5),
    )
    .unwrap();
    landing.size =
        crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(3.0, 0.16, 0.8)).unwrap();
    landing.yaw_radians =
        crate::spatial_geometry::Radians::new(std::f32::consts::FRAC_PI_4).unwrap();
    let floor = super::architecture::Floor::new(&plan, crate::StoreyIndex::GROUND).unwrap();
    assert!(floor.contains(
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(5.2, 6.8)).unwrap()
    ));
    assert!(!floor.contains(
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(5.7, 7.5)).unwrap()
    ));
}

#[test]
fn interior_requires_physical_floors_without_archetype_descriptors() {
    let mut plan = empty_room();
    let layout = InteriorLayout {
        placements: vec![placement(
            FurnitureKind::DiningTable,
            FurnitureVariant::Compact,
        )],
        ..Default::default()
    };
    plan.timber_frame = None;
    plan.church = None;
    plan.small_church = None;
    plan.workplace = None;
    validate_layout(&plan, &layout).unwrap();
    plan.resolved_geometry.solids.clear();
    let floor = super::architecture::Floor::new(&plan, crate::StoreyIndex::GROUND).unwrap();
    assert!(!floor.contains(layout.placements[0].centre_metres));
    assert!(validate_layout(&plan, &layout).is_err());
    assert!(
        furnish(
            &plan,
            &BuildingProgram::fixture(
                BuildingArchetype::TownHouse,
                fabelgeist_determinism::Seed::from_u64(42)
            )
        )
        .is_err()
    );
}
