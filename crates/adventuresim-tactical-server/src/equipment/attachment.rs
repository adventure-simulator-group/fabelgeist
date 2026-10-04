//! Resolve compatible supporting garments and their free attachment capacity.
use super::*;

#[derive(Clone)]
struct AttachmentTarget {
    parent: Entity,
    attachment_point_id: String,
    channel: EquipmentChannel,
    capacity_index: u16,
}

fn attachment_target_accepts(
    moving_tags: &[String],
    target: &AttachmentTarget,
    items: &Query<ItemView<'_>>,
    requirement: &item_catalog::ParentRequirement,
    actor: Entity,
) -> bool {
    if requirement.location.is_some_and(|location| {
        !ordered_at_location(actor, location, items)
            .iter()
            .any(|reachable| reachable.expected_entity() == target.parent)
    }) {
        return false;
    }
    let Ok((_, parent_properties, _, _, _, _, _, _)) = items.get(target.parent) else {
        return false;
    };
    item_catalog::definition(&parent_properties.id)
        .and_then(|definition| definition.equipment.as_ref())
        .and_then(|equipment| {
            equipment
                .attachment_points
                .iter()
                .find(|point| point.id == target.attachment_point_id)
        })
        .is_some_and(|point| {
            point.channel == target.channel
                && point.order == requirement.order
                && (point.accepts_tags.is_empty()
                    || point
                        .accepts_tags
                        .iter()
                        .any(|accepted| moving_tags.contains(accepted)))
        })
}

pub(super) fn attachment_topology(
    item_id: &str,
    selected: &ReachableTarget,
    actor: Entity,
    items: &Query<ItemView<'_>>,
) -> Option<EquipmentTopology> {
    if !parent_placement_allowed(item_id) {
        return None;
    }
    let moving = item_catalog::definition(item_id)?.equipment.as_ref()?;
    let mut available = Vec::<AttachmentTarget>::new();
    let mut explicitly_selected = Vec::<(Entity, String, u16)>::new();
    match selected {
        ReachableTarget::EmptyAttachment {
            parent,
            attachment_point_id,
            channel,
            capacity_index,
        } => {
            explicitly_selected.push((*parent, attachment_point_id.clone(), *capacity_index));
            available.push(AttachmentTarget {
                parent: *parent,
                attachment_point_id: attachment_point_id.clone(),
                channel: *channel,
                capacity_index: *capacity_index,
            });
        }
        ReachableTarget::Occupied(entity) => {
            let (_, _, _, topology, _, _, _, _) = items.get(*entity).ok()?;
            available.extend(topology.occupancies.iter().filter_map(|occupancy| {
                match &occupancy.anchor {
                    TacticalEquipmentAnchor::ItemAttachment {
                        parent,
                        attachment_point_id,
                    } => {
                        explicitly_selected.push((
                            *parent,
                            attachment_point_id.clone(),
                            occupancy.capacity_index,
                        ));
                        Some(AttachmentTarget {
                            parent: *parent,
                            attachment_point_id: attachment_point_id.clone(),
                            channel: occupancy.channel,
                            capacity_index: occupancy.capacity_index,
                        })
                    }
                    _ => None,
                }
            }));
        }
    }
    append_empty_targets(actor, items, &mut available);
    available.sort_by(|left, right| {
        let left_selected = explicitly_selected.iter().any(|selected| {
            selected
                == &(
                    left.parent,
                    left.attachment_point_id.clone(),
                    left.capacity_index,
                )
        });
        let right_selected = explicitly_selected.iter().any(|selected| {
            selected
                == &(
                    right.parent,
                    right.attachment_point_id.clone(),
                    right.capacity_index,
                )
        });
        right_selected.cmp(&left_selected).then(
            left.parent
                .to_bits()
                .cmp(&right.parent.to_bits())
                .then(left.attachment_point_id.cmp(&right.attachment_point_id))
                .then(left.capacity_index.cmp(&right.capacity_index)),
        )
    });
    select_placement(moving, actor, items, &available, &explicitly_selected)
}

fn append_empty_targets(
    actor: Entity,
    items: &Query<ItemView<'_>>,
    available: &mut Vec<AttachmentTarget>,
) {
    // Additional empty points are selected deterministically for multi-parent
    // placements. Existing occupied capacities are never silently displaced.
    for (parent, parent_properties, owner, _, _, _, scene, _) in items.iter() {
        if scene || owner.is_none_or(|owner| owner.0 != actor) {
            continue;
        }
        let Some(parent_equipment) = item_catalog::definition(&parent_properties.id)
            .and_then(|definition| definition.equipment.as_ref())
        else {
            continue;
        };
        for point in &parent_equipment.attachment_points {
            for capacity_index in 0..point.capacity {
                let occupied = items.iter().any(|(_, _, _, topology, _, _, _, _)| {
                    topology.occupancies.iter().any(|occupancy| {
                        matches!(
                            &occupancy.anchor,
                            TacticalEquipmentAnchor::ItemAttachment { parent: found, attachment_point_id }
                                if *found == parent
                                    && attachment_point_id == &point.id
                                    && occupancy.capacity_index == capacity_index
                        )
                    })
                });
                if !occupied
                    && !available.iter().any(|target| {
                        target.parent == parent
                            && target.attachment_point_id == point.id
                            && target.capacity_index == capacity_index
                    })
                {
                    available.push(AttachmentTarget {
                        parent,
                        attachment_point_id: point.id.clone(),
                        channel: point.channel,
                        capacity_index,
                    });
                }
            }
        }
    }
}

fn select_placement(
    moving: &item_catalog::EquipmentDefinition,
    actor: Entity,
    items: &Query<ItemView<'_>>,
    available: &[AttachmentTarget],
    explicitly_selected: &[(Entity, String, u16)],
) -> Option<EquipmentTopology> {
    for placement in &moving.placements {
        if placement.occupancy.is_empty() && !placement.parents.is_empty() {
            let mut chosen = Vec::new();
            for requirement in &placement.parents {
                let Some(target) = available.iter().find(|target| {
                    target.channel == requirement.channel
                        && attachment_target_accepts(
                            &moving.attachment_tags,
                            target,
                            items,
                            requirement,
                            actor,
                        )
                        && !chosen.iter().any(|chosen: &&AttachmentTarget| {
                            chosen.parent == target.parent
                                && chosen.attachment_point_id == target.attachment_point_id
                                && chosen.capacity_index == target.capacity_index
                        })
                }) else {
                    break;
                };
                chosen.push(target);
            }
            if chosen.len() != placement.parents.len() {
                continue;
            }
            if !explicitly_selected.iter().all(|selected| {
                chosen.iter().any(|target| {
                    selected
                        == &(
                            target.parent,
                            target.attachment_point_id.clone(),
                            target.capacity_index,
                        )
                })
            }) {
                continue;
            }
            return Some(EquipmentTopology {
                placement_id: Some(placement.id.clone()),
                occupancies: chosen
                    .into_iter()
                    .enumerate()
                    .map(|(index, target)| EquipmentTopologyOccupancy {
                        occupancy_id: format!(
                            "tactical:{}:{}:{}",
                            target.parent.to_bits(),
                            target.attachment_point_id,
                            target.capacity_index
                        ),
                        anchor: TacticalEquipmentAnchor::ItemAttachment {
                            parent: target.parent,
                            attachment_point_id: target.attachment_point_id.clone(),
                        },
                        channel: target.channel,
                        order: placement.parents[index].order,
                        requirement_index: index as u16,
                        capacity_index: target.capacity_index,
                    })
                    .collect(),
            });
        }
    }
    None
}
