//! Atomic transfers between a held item and authored wearable or attachment roots.
use super::*;

pub(super) fn transfer_slot(
    commands: &mut Commands,
    actor: Entity,
    hand: EquipmentHand,
    location: EquipmentLocation,
    depth: u16,
    items: &Query<ItemView<'_>>,
) -> bool {
    let held = hand_item(actor, hand, items);
    let destination = ordered_at_location(actor, location, items)
        .get(depth as usize)
        .cloned();
    match held {
        None => {
            let Some(ReachableTarget::Occupied(item)) = destination else {
                return false;
            };
            if has_children(item, items) {
                return false;
            }
            commands
                .entity(item)
                .insert((hand_topology(hand), hand.slot()));
            true
        }
        Some(moving) => {
            let Ok((_, properties, _, _, _, _, _, _)) = items.get(moving) else {
                return false;
            };
            if has_children(moving, items) {
                return false;
            }
            let proposed = match destination.as_ref() {
                Some(ReachableTarget::EmptyAttachment { .. }) => {
                    attachment_topology(&properties.id, destination.as_ref().unwrap(), actor, items)
                }
                Some(ReachableTarget::Occupied(item))
                    if items
                        .get(*item)
                        .is_ok_and(|(_, _, _, topology, _, _, _, _)| {
                            topology.occupancies.iter().any(|occupancy| {
                                matches!(
                                    occupancy.anchor,
                                    TacticalEquipmentAnchor::ItemAttachment { .. }
                                )
                            })
                        }) =>
                {
                    attachment_topology(&properties.id, destination.as_ref().unwrap(), actor, items)
                }
                _ => placement_topology(&properties.id, location),
            };
            let Some(proposed) = proposed else {
                return false;
            };
            if proposed
                .occupancies
                .iter()
                .any(|occupancy| occupancy.channel == EquipmentChannel::Held)
            {
                return false;
            }
            let destination = destination
                .filter(|target| fit::target_conflicts(target, &properties.id, &proposed, items));
            let selected_entity = destination.as_ref().map(ReachableTarget::expected_entity);
            if destination.as_ref().is_some_and(|target| match target {
                ReachableTarget::Occupied(item) => has_children(*item, items),
                ReachableTarget::EmptyAttachment { .. } => false,
            }) || fit::topology_conflicts(
                actor,
                &proposed,
                &properties.id,
                &selected_entity.map_or(vec![moving], |item| vec![moving, item]),
                items,
            ) {
                return false;
            }
            // Every condition is proven before these deferred mutations: a
            // multi-location placement or occupied swap commits as one batch.
            commands
                .entity(moving)
                .insert(proposed)
                .remove::<EquipSlot>();
            if let Some(ReachableTarget::Occupied(swapped)) = destination {
                commands
                    .entity(swapped)
                    .insert((hand_topology(hand), hand.slot()));
            }
            true
        }
    }
}
