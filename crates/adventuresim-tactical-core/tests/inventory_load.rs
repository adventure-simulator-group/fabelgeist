//! Carried load follows live quantities and actual ownership relationships.

use adventuresim_core::inventory_measurement::InventoryMassKg;
use adventuresim_core::prelude::PlayerEquipment;
use adventuresim_tactical_core::inventory::{
    InventoryViewer, ItemOf, ItemProperties, TacticalInventoryMass, TacticalItemQuantity,
};
use bevy::{ecs::system::SystemState, prelude::*};

fn carried_kilograms(world: &mut World, actor: Entity) -> f32 {
    let mut viewer = SystemState::<InventoryViewer>::new(world);
    viewer.get(world).unwrap().get(actor).inventory_weight()
}

#[test]
fn exact_remaining_load_is_not_multiplied_by_stack_count_or_used_as_weapon_unit_mass() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let lot = world
        .spawn((
            ItemOf(owner),
            ItemProperties {
                id: "food-lot".into(),
                weight: 2.0,
            },
            TacticalItemQuantity::new(3).unwrap(),
            TacticalInventoryMass::Measured(InventoryMassKg::try_from(1.5).unwrap()),
        ))
        .id();
    let container = world
        .spawn((
            ItemOf(owner),
            ItemProperties {
                id: "filled-container".into(),
                weight: 1.0,
            },
            TacticalInventoryMass::Measured(InventoryMassKg::try_from(4.0).unwrap()),
        ))
        .id();
    assert_eq!(carried_kilograms(&mut world, owner), 5.5);
    world
        .entity_mut(lot)
        .insert(TacticalInventoryMass::Measured(
            InventoryMassKg::try_from(0.5).unwrap(),
        ));
    assert_eq!(carried_kilograms(&mut world, owner), 4.5);
    assert_eq!(world.get::<ItemProperties>(container).unwrap().weight, 1.0);
    world.entity_mut(container).remove::<ItemOf>();
    assert_eq!(carried_kilograms(&mut world, owner), 0.5);
}

#[test]
fn stack_quantities_and_custody_changes_update_the_load_seen_by_combat() {
    let mut world = World::new();
    let first = world.spawn_empty().id();
    let second = world.spawn_empty().id();
    let stack = world
        .spawn((
            ItemOf(first),
            ItemProperties {
                id: "stock".into(),
                weight: 2.0,
            },
            TacticalItemQuantity::new(3).unwrap(),
        ))
        .id();
    let other = world
        .spawn((
            ItemOf(second),
            ItemProperties {
                id: "other-stock".into(),
                weight: 5.0,
            },
        ))
        .id();
    assert_eq!(carried_kilograms(&mut world, first), 6.0);
    assert_eq!(carried_kilograms(&mut world, second), 5.0);

    world
        .entity_mut(stack)
        .insert(TacticalItemQuantity::new(2).unwrap());
    assert_eq!(carried_kilograms(&mut world, first), 4.0);
    world.entity_mut(stack).insert(ItemOf(second));
    assert_eq!(carried_kilograms(&mut world, first), 0.0);
    assert_eq!(carried_kilograms(&mut world, second), 9.0);

    world.entity_mut(stack).remove::<ItemOf>();
    assert_eq!(carried_kilograms(&mut world, second), 5.0);
    world.despawn(other);
    assert_eq!(carried_kilograms(&mut world, second), 0.0);
}
