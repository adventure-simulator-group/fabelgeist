//! Ordered equipment occupancy and attachment capacity for grab/HUD previews.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct PreviewTarget {
    pub(super) entity: Entity,
    pub(super) occupancy: PreviewOccupancy,
    pub(super) attachment: PreviewAttachment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreviewOccupancy {
    Empty,
    Occupied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PreviewAttachment {
    Detached,
    Attached,
}

/// The held item at catalogue admission, retaining an unknown item separately
/// from an empty hand so an unavailable definition cannot select occupied slots.
#[derive(Clone, Copy)]
pub(super) enum HeldPreviewItem {
    Empty,
    Unknown,
    Defined(&'static item_catalog::ItemDefinition),
}

impl From<Option<&ItemProperties>> for HeldPreviewItem {
    fn from(properties: Option<&ItemProperties>) -> Self {
        match properties {
            None => Self::Empty,
            Some(properties) => {
                item_catalog::definition(&properties.id).map_or(Self::Unknown, Self::Defined)
            }
        }
    }
}

impl PreviewAttachment {
    fn from_predicate(attached: bool) -> Self {
        if attached {
            Self::Attached
        } else {
            Self::Detached
        }
    }
}

pub(super) fn outermost_occupied_depth(layers: &[PreviewTarget]) -> Option<usize> {
    layers
        .iter()
        .position(|target| target.occupancy == PreviewOccupancy::Occupied)
}

pub(super) fn eligible_slot_depth(
    item: &item_catalog::ItemDefinition,
    location: EquipmentLocation,
    layers: &[PreviewTarget],
) -> Option<usize> {
    let equipment = item.equipment.as_ref()?;
    if equipment.placements.iter().any(|placement| {
        placement
            .occupancy
            .iter()
            .any(|occupancy| occupancy.location == location)
    }) {
        return Some(0);
    }
    equipment
        .placements
        .iter()
        .any(|placement| !placement.parents.is_empty())
        .then(|| {
            layers
                .iter()
                .position(|target| target.attachment == PreviewAttachment::Attached)
        })?
}

pub(super) fn ordered_preview_at_location(
    actor: Entity,
    location: EquipmentLocation,
    topologies: &Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>,
) -> Vec<PreviewTarget> {
    let mut found: Vec<_> = topologies
        .iter()
        .filter(|(_, owner, _, _)| owner.0 == actor)
        .filter_map(|(entity, _, topology, properties)| {
            topology
                .root_order(location, &properties.id)
                .map(|key| (key, entity))
        })
        .collect();
    found.sort_by_key(|(key, _)| *key);
    let mut output = Vec::new();
    let mut visited = std::collections::HashSet::new();
    for (_, entity) in found {
        append_preview(entity, location, topologies, &mut visited, &mut output);
    }
    output
}

pub(super) fn hud_layers(
    actor: Entity,
    location: EquipmentLocation,
    items: &Query<(
        Entity,
        &ItemOf,
        Option<&EquipSlot>,
        &ItemProperties,
        &EquipmentTopology,
    )>,
) -> Vec<PreviewTarget> {
    let mut roots: Vec<_> = items
        .iter()
        .filter(|(_, owner, _, _, _)| owner.0 == actor)
        .filter_map(|(entity, _, _, properties, topology)| {
            topology
                .root_order(location, &properties.id)
                .map(|key| (key, entity))
        })
        .collect();
    roots.sort_by_key(|(key, _)| *key);
    let mut output = Vec::new();
    let mut visited = std::collections::HashSet::new();
    for (_, root) in roots {
        append_hud_preview(root, location, items, &mut visited, &mut output);
    }
    output
}

fn append_preview(
    entity: Entity,
    location: EquipmentLocation,
    items: &Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>,
    visited: &mut std::collections::HashSet<Entity>,
    output: &mut Vec<PreviewTarget>,
) {
    if !visited.insert(entity) {
        return;
    }
    let Ok((_, _, _, properties)) = items.get(entity) else {
        output.push(PreviewTarget {
            entity,
            occupancy: PreviewOccupancy::Occupied,
            attachment: PreviewAttachment::Detached,
        });
        return;
    };
    let Some(equipment) = item_catalog::definition(&properties.id)
        .and_then(|definition| definition.equipment.as_ref())
    else {
        output.push(PreviewTarget {
            entity,
            occupancy: PreviewOccupancy::Occupied,
            attachment: PreviewAttachment::Detached,
        });
        return;
    };
    let mut points: Vec<_> = equipment.attachment_points.iter().collect();
    points.sort_by_key(|point| (point.order, point.id.as_str()));
    for point in points
        .into_iter()
        .filter(|point| point.locations.is_empty() || point.locations.contains(&location))
    {
        for capacity_index in 0..point.capacity {
            let child =
                items.iter().find_map(|(child, _, topology, _)| {
                    topology.occupancies.iter().any(|occupancy| {
                    matches!(
                        &occupancy.anchor,
                        TacticalEquipmentAnchor::ItemAttachment { parent, attachment_point_id }
                            if *parent == entity
                                && attachment_point_id == &point.id
                                && occupancy.capacity_index == capacity_index
                    )
                }).then_some(child)
                });
            if let Some(child) = child {
                append_preview(child, location, items, visited, output);
            } else {
                // Empty attachment points address their mapped parent. Depth
                // disambiguates the exact authored point/capacity on server.
                output.push(PreviewTarget {
                    entity,
                    occupancy: PreviewOccupancy::Empty,
                    attachment: PreviewAttachment::Attached,
                });
            }
        }
    }
    output.push(PreviewTarget {
        entity,
        occupancy: PreviewOccupancy::Occupied,
        attachment: PreviewAttachment::from_predicate(items.get(entity).is_ok_and(
            |(_, _, topology, _)| {
                topology.occupancies.iter().any(|occupancy| {
                    matches!(
                        occupancy.anchor,
                        TacticalEquipmentAnchor::ItemAttachment { .. }
                    )
                })
            },
        )),
    });
}

fn append_hud_preview(
    entity: Entity,
    location: EquipmentLocation,
    items: &Query<(
        Entity,
        &ItemOf,
        Option<&EquipSlot>,
        &ItemProperties,
        &EquipmentTopology,
    )>,
    visited: &mut std::collections::HashSet<Entity>,
    output: &mut Vec<PreviewTarget>,
) {
    if !visited.insert(entity) {
        return;
    }
    let Ok((_, _, _, properties, _)) = items.get(entity) else {
        output.push(PreviewTarget {
            entity,
            occupancy: PreviewOccupancy::Occupied,
            attachment: PreviewAttachment::Detached,
        });
        return;
    };
    let Some(equipment) = item_catalog::definition(&properties.id)
        .and_then(|definition| definition.equipment.as_ref())
    else {
        output.push(PreviewTarget {
            entity,
            occupancy: PreviewOccupancy::Occupied,
            attachment: PreviewAttachment::Detached,
        });
        return;
    };
    let mut points: Vec<_> = equipment.attachment_points.iter().collect();
    points.sort_by_key(|point| (point.order, point.id.as_str()));
    for point in points
        .into_iter()
        .filter(|point| point.locations.is_empty() || point.locations.contains(&location))
    {
        for capacity_index in 0..point.capacity {
            let child =
                items.iter().find_map(|(child, _, _, _, topology)| {
                    topology.occupancies.iter().any(|occupancy| {
                    matches!(
                        &occupancy.anchor,
                        TacticalEquipmentAnchor::ItemAttachment { parent, attachment_point_id }
                            if *parent == entity
                                && attachment_point_id == &point.id
                                && occupancy.capacity_index == capacity_index
                    )
                }).then_some(child)
                });
            if let Some(child) = child {
                append_hud_preview(child, location, items, visited, output);
            } else {
                output.push(PreviewTarget {
                    entity,
                    occupancy: PreviewOccupancy::Empty,
                    attachment: PreviewAttachment::Attached,
                });
            }
        }
    }
    output.push(PreviewTarget {
        entity,
        occupancy: PreviewOccupancy::Occupied,
        attachment: PreviewAttachment::from_predicate(items.get(entity).is_ok_and(
            |(_, _, _, _, topology)| {
                topology.occupancies.iter().any(|occupancy| {
                    matches!(
                        occupancy.anchor,
                        TacticalEquipmentAnchor::ItemAttachment { .. }
                    )
                })
            },
        )),
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn outermost_display_skips_empty_attachment_capacity() {
        let parent = Entity::from_bits(1);
        let child = Entity::from_bits(2);
        let layers = [
            PreviewTarget {
                entity: parent,
                occupancy: PreviewOccupancy::Empty,
                attachment: PreviewAttachment::Attached,
            },
            PreviewTarget {
                entity: child,
                occupancy: PreviewOccupancy::Occupied,
                attachment: PreviewAttachment::Attached,
            },
            PreviewTarget {
                entity: parent,
                occupancy: PreviewOccupancy::Occupied,
                attachment: PreviewAttachment::Detached,
            },
        ];
        assert_eq!(outermost_occupied_depth(&layers), Some(1));
    }

    #[test]
    fn preview_traversal_keeps_belt_attachment_points_in_their_slots() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let belt = world
            .spawn((
                ItemOf(actor),
                ItemProperties {
                    id: "belt".into(),
                    weight: 0.35,
                },
                EquipmentTopology {
                    placement_id: Some("worn".into()),
                    occupancies: [
                        EquipmentLocation::LeftBelt,
                        EquipmentLocation::RightBelt,
                        EquipmentLocation::FrontBelt,
                        EquipmentLocation::BackBelt,
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, location)| EquipmentTopologyOccupancy {
                        occupancy_id: format!("belt-{index}"),
                        anchor: TacticalEquipmentAnchor::CharacterLocation(location),
                        channel: EquipmentChannel::Accessory,
                        order: 0,
                        requirement_index: index as u16,
                        capacity_index: 0,
                    })
                    .collect(),
                },
            ))
            .id();
        let right_sheath = world
            .spawn((
                ItemOf(actor),
                ItemProperties {
                    id: "scabbard".into(),
                    weight: 0.3,
                },
                EquipmentTopology {
                    placement_id: Some("right".into()),
                    occupancies: vec![EquipmentTopologyOccupancy {
                        occupancy_id: "right-sheath".into(),
                        anchor: TacticalEquipmentAnchor::ItemAttachment {
                            parent: belt,
                            attachment_point_id: "right".into(),
                        },
                        channel: EquipmentChannel::Mount,
                        order: 1,
                        requirement_index: 0,
                        capacity_index: 0,
                    }],
                },
            ))
            .id();
        let sheath = world
            .spawn((
                ItemOf(actor),
                ItemProperties {
                    id: "sword_sheath".into(),
                    weight: 0.3,
                },
                EquipmentTopology {
                    placement_id: Some("attached".into()),
                    occupancies: vec![EquipmentTopologyOccupancy {
                        occupancy_id: "sheath".into(),
                        anchor: TacticalEquipmentAnchor::ItemAttachment {
                            parent: belt,
                            attachment_point_id: "left".into(),
                        },
                        channel: EquipmentChannel::Mount,
                        order: 0,
                        requirement_index: 0,
                        capacity_index: 0,
                    }],
                },
            ))
            .id();
        let preview = world
            .run_system_once(
                move |items: Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>| {
                    ordered_preview_at_location(actor, EquipmentLocation::LeftBelt, &items)
                },
            )
            .unwrap();
        assert_eq!(preview.first().map(|target| target.entity), Some(sheath));
        assert!(
            preview.first().unwrap().occupancy == PreviewOccupancy::Empty,
            "deepest empty blade is first"
        );
        assert_eq!(preview.last().map(|target| target.entity), Some(belt));
        let right_preview = world
            .run_system_once(
                move |items: Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>| {
                    ordered_preview_at_location(actor, EquipmentLocation::RightBelt, &items)
                },
            )
            .unwrap();
        assert_eq!(
            right_preview.first().map(|target| target.entity),
            Some(right_sheath)
        );
        assert!(
            right_preview.iter().all(|target| target.entity != sheath),
            "the left sheath must not appear in the right belt slot"
        );
    }
}
