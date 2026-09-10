use super::*;
use crate::scene_input::{TacticalBuildingPlacement, buildings::prepare_buildings};
use adventuresim_building_generator::{BuildingProgram, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::math::Vec3Swizzles;

#[test]
fn interior_instances_follow_building_rotation_elevation_and_room_identity() {
    let usage = BuildingUse::Dwelling;
    let input = TacticalBuildingPlacement {
        id: 891,
        program: BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 47_101),
        centre_metres: Vec2::new(12.0, -19.0),
        orientation: BuildingOrientation::from_radians(0.73).unwrap(),
    };
    let mut buildings = prepare_buildings(&[input]).unwrap();
    buildings[0].pad_elevation_metres = 4.2;
    let mut furniture = FurnitureLayout::default();
    append(&mut furniture, &buildings).unwrap();
    let building = &buildings[0];
    let proof = &furniture.interiors[0];
    assert_eq!(proof.building_id, building.placement.id);
    assert!(!proof.layout.placements.is_empty());
    assert!(proof.layout.placements.iter().any(|p| p.storey > 0));
    assert_eq!(furniture.instances.len(), proof.layout.placements.len());
    adventuresim_building_generator::interior::validate_layout(&building.plan, &proof.layout)
        .unwrap();
    let origin = building.collision.bounds.centre();
    for (instance, placement) in furniture.instances.iter().zip(&proof.layout.placements) {
        let local = building
            .placement
            .orientation
            .world_to_local(instance.position_metres.xz() - building.placement.centre_metres)
            + Vec2::new(origin.x, origin.z);
        assert!(local.distance(placement.centre_metres) < 0.0001);
        assert!(
            (instance.position_metres.y - 4.2 + building.collision.bounds.min.y
                - furniture_floor_height(&building.plan, placement))
            .abs()
                < 0.0001
        );
        let world_front = instance.orientation.local_to_world(-Vec2::Y);
        let expected_front = building.placement.orientation.local_to_world(
            BuildingOrientation::from_radians(placement.yaw_radians())
                .unwrap()
                .local_to_world(-Vec2::Y),
        );
        assert!(world_front.distance(expected_front) < 0.0001);
        assert_eq!(
            instance.scene.location,
            FurnitureLocation::Interior {
                building_id: building.placement.id,
                room_id: placement.room_id,
                storey: placement.storey,
            }
        );
    }
    assert_eq!(
        furniture
            .instances
            .iter()
            .map(|i| i.scene.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        furniture.instances.len()
    );
    let mut repeated = FurnitureLayout::default();
    append(&mut repeated, &buildings).unwrap();
    assert_eq!(repeated, furniture);
}
