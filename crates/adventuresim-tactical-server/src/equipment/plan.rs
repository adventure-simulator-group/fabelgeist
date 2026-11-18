//! Translate an authoritative selection into one atomic durable change set.

use adventuresim_core::{
    equipment_command::{
        EquipmentChange, EquipmentChanges, EquipmentDestination, EquipmentExchange,
        EquipmentIntent, ParentSelection,
    },
    equipment_placement::{EquipmentAttachmentPointId, EquipmentParentRequirementIndex},
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
};

use super::*;

type PlanResult<T> = Result<T, EquipmentRejection>;

pub(super) fn plan_slot(
    actor: Entity,
    hand: EquipmentHand,
    location: EquipmentLocation,
    depth: u16,
    items: &Query<ItemView<'_>>,
    rows: &Query<&TacticalInventoryItemId>,
) -> PlanResult<EquipmentIntent> {
    let destination = ordered_at_location(actor, location, items)
        .get(usize::from(depth))
        .cloned();
    let Some(moving) = hand_item(actor, hand, items) else {
        let Some(ReachableTarget::Occupied(item)) = destination else {
            return Err(EquipmentRejection::InvalidPlacement);
        };
        if has_children(item, items) {
            return Err(EquipmentRejection::InvalidPlacement);
        }
        return Ok(change_set(change(item, held(hand), rows)?, None));
    };
    if has_children(moving, items) {
        return Err(EquipmentRejection::InvalidPlacement);
    }
    let (_, properties, _, _, _, _) = items.get(moving).map_err(|_| EquipmentRejection::Stale)?;
    let proposed = proposed_topology(&properties.id, destination.as_ref(), actor, location, items)
        .ok_or(EquipmentRejection::InvalidPlacement)?;
    let destination = destination
        .filter(|target| fit::target_conflicts(target, &properties.id, &proposed, items));
    let swapped = match destination {
        Some(ReachableTarget::Occupied(item)) => Some(item),
        _ => None,
    };
    if swapped.is_some_and(|item| has_children(item, items))
        || fit::topology_conflicts(
            actor,
            &proposed,
            &properties.id,
            &swapped.map_or(vec![moving], |item| vec![moving, item]),
            items,
        )
    {
        return Err(EquipmentRejection::InvalidPlacement);
    }
    let target = authored(&properties.id, proposed, rows)?;
    Ok(change_set(
        change(moving, target, rows)?,
        swapped
            .map(|item| change(item, held(hand), rows))
            .transpose()?,
    ))
}

pub(super) fn plan_hand(
    actor: Entity,
    source: EquipmentHand,
    destination: EquipmentHand,
    items: &Query<ItemView<'_>>,
    rows: &Query<&TacticalInventoryItemId>,
) -> PlanResult<EquipmentIntent> {
    if source == destination {
        return Err(EquipmentRejection::InvalidPlacement);
    }
    let moving = hand_item(actor, source, items)
        .map(|item| change(item, held(destination), rows))
        .transpose()?;
    let swapped = hand_item(actor, destination, items)
        .map(|item| change(item, held(source), rows))
        .transpose()?;
    match (moving, swapped) {
        (Some(first), second) => Ok(change_set(first, second)),
        (None, Some(first)) => Ok(change_set(first, None)),
        (None, None) => Err(EquipmentRejection::InvalidPlacement),
    }
}

fn proposed_topology(
    item_id: &str,
    selected: Option<&ReachableTarget>,
    actor: Entity,
    location: EquipmentLocation,
    items: &Query<ItemView<'_>>,
) -> Option<EquipmentTopology> {
    match selected {
        Some(target @ ReachableTarget::EmptyAttachment { .. }) => {
            attachment_topology(item_id, target, actor, items)
        }
        Some(target @ ReachableTarget::Occupied(item))
            if items.get(*item).is_ok_and(|(_, _, _, topology, _, _)| {
                topology.occupancies.iter().any(|occupancy| {
                    matches!(
                        occupancy.anchor,
                        TacticalEquipmentAnchor::ItemAttachment { .. }
                    )
                })
            }) =>
        {
            attachment_topology(item_id, target, actor, items)
        }
        _ => placement_topology(item_id, location),
    }
}

fn row_id(entity: Entity, rows: &Query<&TacticalInventoryItemId>) -> PlanResult<InventoryRowId> {
    rows.get(entity)
        .ok()
        .and_then(|row| InventoryRowId::try_new(row.0).ok())
        .ok_or(EquipmentRejection::Unavailable)
}

fn authored(
    item_id: &str,
    topology: EquipmentTopology,
    rows: &Query<&TacticalInventoryItemId>,
) -> PlanResult<EquipmentDestination> {
    let placement_id = topology
        .authored_placement
        .ok_or(EquipmentRejection::InvalidPlacement)?;
    let placement_index = item_catalog::definition(item_id)
        .and_then(|definition| definition.equipment.as_ref())
        .and_then(|equipment| {
            equipment
                .placements
                .iter()
                .position(|placement| placement.id == placement_id.as_str())
        })
        .and_then(|index| u16::try_from(index).ok())
        .ok_or(EquipmentRejection::InvalidPlacement)?;
    let targets = topology
        .occupancies
        .into_iter()
        .filter_map(|occupancy| match occupancy.anchor {
            TacticalEquipmentAnchor::ItemAttachment {
                parent,
                attachment_point_id,
            } => Some(row_id(parent, rows).and_then(|parent| {
                Ok(ParentSelection {
                    requirement: EquipmentParentRequirementIndex::new(occupancy.requirement_index),
                    parent,
                    point: EquipmentAttachmentPointId::try_from(attachment_point_id)
                        .map_err(|_| EquipmentRejection::InvalidPlacement)?,
                })
            })),
            TacticalEquipmentAnchor::CharacterLocation(_) => None,
        })
        .collect::<PlanResult<Vec<_>>>()?;
    Ok(EquipmentDestination::authored(
        EquipmentPlacementIndex::new(placement_index),
        targets
            .try_into()
            .map_err(|_| EquipmentRejection::InvalidPlacement)?,
    ))
}

fn held(hand: EquipmentHand) -> EquipmentDestination {
    EquipmentDestination::Held(hand)
}

fn change(
    item: Entity,
    destination: EquipmentDestination,
    rows: &Query<&TacticalInventoryItemId>,
) -> PlanResult<EquipmentChange> {
    Ok(EquipmentChange {
        inventory: row_id(item, rows)?,
        destination,
    })
}

fn change_set(first: EquipmentChange, second: Option<EquipmentChange>) -> EquipmentIntent {
    EquipmentIntent {
        changes: match second {
            Some(second) => EquipmentChanges::Exchange(EquipmentExchange {
                source: first,
                destination: second,
            }),
            None => EquipmentChanges::Single(first),
        },
        conflicts: EquipmentConflictPolicy::RejectOccupied,
    }
}
