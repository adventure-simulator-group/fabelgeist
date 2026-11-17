//! Carried load follows live quantities and actual ownership relationships.

use adventuresim_core::prelude::PlayerEquipment;
use adventuresim_tactical_core::inventory::{
    InventoryViewer, ItemOf, ItemProperties, TacticalItemQuantity,
};
use bevy::{ecs::system::SystemState, prelude::*};

fn carried_kilograms(world: &mut World, actor: Entity) -> f32 {
    let mut viewer = SystemState::<InventoryViewer>::new(world);
    viewer.get(world).unwrap().get(actor).inventory_weight()
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
