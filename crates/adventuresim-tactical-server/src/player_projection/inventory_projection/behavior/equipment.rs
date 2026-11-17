//! Exercise durable equipment transitions against native bodies and item caches.

use super::*;
use adventuresim_stdb_client::{
    AuthoredPlacementId, CatalogItemKind, ConnectedEquipmentOccupancy, EquipmentAnchorKind,
    EquipmentBodyPart, EquipmentChannel, EquipmentHand, EquipmentLocation, EquippedPlacement,
};

#[test]
fn refresh_moves_clothing_between_worn_and_held_without_replacing_body_or_item() {
    let mut world = World::new();
    world.insert_resource(TacticalCombatConfig::default());
    let owner = actor(&mut world, 1);
    let mut tunic = item(11, 1, 2.0, 2.0);
    tunic.item.id = "linen_tunic".into();
    tunic.item.kind = CatalogItemKind::Clothing;
    tunic.physical_object_id = Some(50);
    tunic.selected_placement = Some(EquippedPlacement::Authored(AuthoredPlacementId {
        value: "worn".into(),
    }));
    tunic.protected_body_parts = vec![
        EquipmentBodyPart::Chest,
        EquipmentBodyPart::Stomach,
        EquipmentBodyPart::LeftArm,
        EquipmentBodyPart::RightArm,
    ];
    tunic.occupancies = [
        EquipmentLocation::Chest,
        EquipmentLocation::Stomach,
        EquipmentLocation::LeftArm,
        EquipmentLocation::RightArm,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, location)| ConnectedEquipmentOccupancy {
        id: format!("tunic:{index}"),
        anchor_kind: EquipmentAnchorKind::CharacterLocation,
        location: Some(location),
        parent_inventory_item_id: None,
        attachment_point_id: None,
        channel: EquipmentChannel::BaseClothing,
        order: 0,
        requirement_index: index as u16,
        capacity_index: 0,
    })
    .collect();
    refresh_snapshots(&mut world, vec![snapshot(1, vec![tunic.clone()])]).unwrap();
    let entity = row_entity(&mut world, 11);
    assert!(world.get::<ArmorItem>(entity).is_some());
    let worn = world.get::<EquipmentTopology>(entity).unwrap().clone();
    assert_eq!(worn.authored_placement.as_ref().unwrap().as_str(), "worn");

    for (hand, location, slot) in [
        (
            EquipmentHand::Left,
            EquipmentLocation::LeftHand,
            EquipSlot::HoldingLeft,
        ),
        (
            EquipmentHand::Right,
            EquipmentLocation::RightHand,
            EquipSlot::HoldingRight,
        ),
    ] {
        tunic.selected_placement = Some(EquippedPlacement::Held(hand));
        tunic.protected_body_parts.clear();
        tunic.occupancies = vec![ConnectedEquipmentOccupancy {
            id: "tunic:held".into(),
            anchor_kind: EquipmentAnchorKind::CharacterLocation,
            location: Some(location),
            parent_inventory_item_id: None,
            attachment_point_id: None,
            channel: EquipmentChannel::Held,
            order: 0,
            requirement_index: 0,
            capacity_index: 0,
        }];
        refresh_snapshots(&mut world, vec![snapshot(1, vec![tunic.clone()])]).unwrap();
        assert_eq!(row_entity(&mut world, 11), entity);
        assert!(world.get::<ArmorItem>(entity).is_none());
        assert_eq!(world.get::<EquipSlot>(entity), Some(&slot));
        assert!(
            world
                .get::<EquipmentTopology>(entity)
                .unwrap()
                .authored_placement
                .is_none()
        );
        assert_eq!(load(&mut world, owner), 2.0);
        assert_eq!(
            world.get::<TacticalCombatState>(owner).unwrap().fatigue,
            0.25
        );
        assert_eq!(
            world.get::<Transform>(owner).unwrap().translation,
            Vec3::new(7.0, 3.0, 2.0)
        );
    }
    tunic.selected_placement = None;
    tunic.occupancies.clear();
    refresh_snapshots(&mut world, vec![snapshot(1, vec![tunic])]).unwrap();
    assert_eq!(row_entity(&mut world, 11), entity);
    assert!(world.get::<EquipSlot>(entity).is_none());
    assert!(world.get::<ArmorItem>(entity).is_none());
    assert_eq!(load(&mut world, owner), 2.0);
}
