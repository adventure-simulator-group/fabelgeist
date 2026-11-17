//! Read-only selection of authored equipment and reachable attachments.

use super::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReachableTarget {
    Occupied(Entity),
    EmptyAttachment {
        parent: Entity,
        attachment_point_id: String,
        channel: EquipmentChannel,
        capacity_index: u16,
    },
}

impl ReachableTarget {
    pub(super) fn expected_entity(&self) -> Entity {
        match self {
            Self::Occupied(entity) => *entity,
            Self::EmptyAttachment { parent, .. } => *parent,
        }
    }
}

pub(super) fn hand_item(
    actor: Entity,
    hand: EquipmentHand,
    items: &Query<ItemView<'_>>,
) -> Option<Entity> {
    items.iter().find_map(|(entity, _, owner, _, slot, scene)| {
        (!scene
            && owner.is_some_and(|owner| owner.0 == actor)
            && slot.is_some_and(|slot| *slot == EquipSlot::from(hand)))
        .then_some(entity)
    })
}

pub(super) fn ordered_at_location(
    actor: Entity,
    location: EquipmentLocation,
    items: &Query<ItemView<'_>>,
) -> Vec<ReachableTarget> {
    let mut found: Vec<_> = items
        .iter()
        .filter(|(_, _, owner, _, _, scene)| !scene && owner.is_some_and(|owner| owner.0 == actor))
        .filter_map(|(entity, properties, _, topology, _, _)| {
            topology
                .root_order(location, &properties.id)
                .map(|key| (key, entity))
        })
        .collect();
    found.sort_by_key(|(key, _)| *key);
    let mut reachable = Vec::new();
    let mut visited = HashSet::new();
    for (_, entity) in found {
        append_reachable(entity, location, items, &mut visited, &mut reachable);
    }
    reachable
}

fn append_reachable(
    entity: Entity,
    location: EquipmentLocation,
    items: &Query<ItemView<'_>>,
    visited: &mut HashSet<Entity>,
    output: &mut Vec<ReachableTarget>,
) {
    if !visited.insert(entity) {
        return;
    }
    let Ok((_, properties, _, _, _, _)) = items.get(entity) else {
        output.push(ReachableTarget::Occupied(entity));
        return;
    };
    let Some(equipment) = item_catalog::definition(&properties.id)
        .and_then(|definition| definition.equipment.as_ref())
    else {
        output.push(ReachableTarget::Occupied(entity));
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
                items.iter().find_map(|(child, _, _, topology, _, _)| {
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
                append_reachable(child, location, items, visited, output);
            } else {
                output.push(ReachableTarget::EmptyAttachment {
                    parent: entity,
                    attachment_point_id: point.id.clone(),
                    channel: point.channel,
                    capacity_index,
                });
            }
        }
    }
    output.push(ReachableTarget::Occupied(entity));
}

pub(super) fn has_children(entity: Entity, items: &Query<ItemView<'_>>) -> bool {
    items.iter().any(|(_, _, _, topology, _, _)| {
        topology.occupancies.iter().any(|occupancy| {
            matches!(
                occupancy.anchor,
                TacticalEquipmentAnchor::ItemAttachment { parent, .. } if parent == entity
            )
        })
    })
}

#[cfg(test)]
pub(super) fn hand_topology(hand: EquipmentHand) -> EquipmentTopology {
    EquipmentTopology {
        authored_placement: None,
        occupancies: vec![EquipmentTopologyOccupancy {
            occupancy_id: format!("tactical:{}:held", hand.id()),
            anchor: TacticalEquipmentAnchor::CharacterLocation(hand.location()),
            channel: EquipmentChannel::Held,
            order: 0,
            requirement_index: 0,
            capacity_index: 0,
        }],
    }
}

pub(super) fn placement_topology(
    item_id: &str,
    location: EquipmentLocation,
) -> Option<EquipmentTopology> {
    let definition = item_catalog::definition(item_id)?;
    let equipment = definition.equipment.as_ref()?;
    let placement = equipment.placements.iter().find(|placement| {
        placement.parents.is_empty()
            && placement
                .occupancy
                .iter()
                .any(|requirement| requirement.location == location)
    })?;
    if !authored_placement_allowed(item_id) {
        return None;
    }
    Some(EquipmentTopology {
        authored_placement: Some(placement.id.clone().try_into().ok()?),
        occupancies: placement
            .occupancy
            .iter()
            .enumerate()
            .map(|(index, requirement)| EquipmentTopologyOccupancy {
                occupancy_id: format!("tactical:{item_id}:{}:{index}", placement.id),
                anchor: TacticalEquipmentAnchor::CharacterLocation(requirement.location),
                channel: requirement.channel,
                order: requirement.order,
                requirement_index: index as u16,
                capacity_index: 0,
            })
            .collect(),
    })
}

pub(super) fn authored_placement_allowed(item_id: &str) -> bool {
    item_catalog::weapon_carry(item_id) != Some(item_catalog::WeaponCarry::HandOnly)
}

pub(super) fn attachment_occupancies_conflict(
    candidate: &EquipmentTopologyOccupancy,
    current: &EquipmentTopologyOccupancy,
) -> bool {
    match (&candidate.anchor, &current.anchor) {
        (
            TacticalEquipmentAnchor::ItemAttachment {
                parent: left,
                attachment_point_id: left_point,
            },
            TacticalEquipmentAnchor::ItemAttachment {
                parent: right,
                attachment_point_id: right_point,
            },
        ) => {
            left == right
                && left_point == right_point
                && candidate.capacity_index == current.capacity_index
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn spawn_test_item(
        world: &mut World,
        actor: Entity,
        id: &str,
        topology: EquipmentTopology,
    ) -> Entity {
        world
            .spawn((
                ItemProperties {
                    id: id.into(),
                    weight: 1.0,
                },
                ItemOf(actor),
                topology,
                Transform::default(),
            ))
            .id()
    }

    #[test]
    fn multi_location_catalog_placement_is_planned_as_one_topology() {
        let topology = placement_topology("linen_tunic", EquipmentLocation::Chest)
            .expect("tunic chest placement");
        assert!(topology.occupancies.len() > 1);
        assert!(topology.occupancies.iter().any(|occupancy| {
            occupancy.anchor
                == TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftArm)
        }));
        assert!(topology.occupancies.iter().any(|occupancy| {
            occupancy.anchor
                == TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::RightArm)
        }));
    }

    #[test]
    fn hand_topology_contains_only_the_selected_mapped_hand() {
        let topology = hand_topology(EquipmentHand::Left);
        assert_eq!(topology.occupancies.len(), 1);
        assert_eq!(
            topology.occupancies[0].anchor,
            TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftHand)
        );
    }

    #[test]
    fn parent_only_stack_destination_fails_closed_in_body_planner() {
        assert!(placement_topology("arming_sword", EquipmentLocation::Chest).is_none());
    }

    #[test]
    fn hand_only_weapons_fail_closed_for_parent_placement() {
        for item_id in ["halberd", "hunting_spear", "military_pike", "spear"] {
            assert!(!authored_placement_allowed(item_id), "{item_id}");
            let equipment = item_catalog::definition(item_id)
                .unwrap()
                .equipment
                .as_ref()
                .unwrap();
            assert!(
                equipment
                    .placements
                    .iter()
                    .all(|placement| placement.parents.is_empty())
            );
        }
        assert!(authored_placement_allowed("arming_sword"));
    }

    #[test]
    fn attachment_capacity_indices_are_distinct_even_with_same_requirement_order() {
        let parent = Entity::from_bits(11);
        let occupancy = |capacity_index| EquipmentTopologyOccupancy {
            occupancy_id: format!("capacity-{capacity_index}"),
            anchor: TacticalEquipmentAnchor::ItemAttachment {
                parent,
                attachment_point_id: "loops".into(),
            },
            channel: EquipmentChannel::Mount,
            order: 0,
            requirement_index: 0,
            capacity_index,
        };
        assert!(attachment_occupancies_conflict(
            &occupancy(0),
            &occupancy(0)
        ));
        assert!(!attachment_occupancies_conflict(
            &occupancy(0),
            &occupancy(1)
        ));
    }

    #[test]
    fn belt_locations_traverse_only_their_authored_attachment_points() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let belt = spawn_test_item(
            &mut world,
            actor,
            "belt",
            EquipmentTopology {
                authored_placement: Some("worn".to_owned().try_into().unwrap()),
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
        );
        for (location, expected_point) in [
            (EquipmentLocation::LeftBelt, "left"),
            (EquipmentLocation::RightBelt, "right"),
            (EquipmentLocation::FrontBelt, "front"),
            (EquipmentLocation::BackBelt, "back"),
        ] {
            let reachable = world
                .run_system_once(move |items: Query<ItemView<'_>>| {
                    ordered_at_location(actor, location, &items)
                })
                .unwrap();
            assert!(
                matches!(reachable.first(), Some(ReachableTarget::EmptyAttachment { attachment_point_id, .. }) if attachment_point_id == expected_point)
            );
            assert_eq!(reachable.len(), 2);
            assert_eq!(reachable.last(), Some(&ReachableTarget::Occupied(belt)));
        }
    }

    #[test]
    fn multi_parent_attachment_is_atomic_and_capacity_fail_closed() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let belt = spawn_test_item(
            &mut world,
            actor,
            "belt",
            EquipmentTopology {
                authored_placement: Some("worn".to_owned().try_into().unwrap()),
                occupancies: vec![EquipmentTopologyOccupancy {
                    occupancy_id: "belt".into(),
                    anchor: TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftBelt),
                    channel: EquipmentChannel::Accessory,
                    order: 0,
                    requirement_index: 0,
                    capacity_index: 0,
                }],
            },
        );
        let selected = ReachableTarget::EmptyAttachment {
            parent: belt,
            attachment_point_id: "left".into(),
            channel: EquipmentChannel::Mount,
            capacity_index: 0,
        };
        let planned = world
            .run_system_once(move |items: Query<ItemView<'_>>| {
                attachment_topology("sword_sheath", &selected, actor, &items)
            })
            .unwrap()
            .expect("two belt mounts are available");
        assert_eq!(planned.occupancies.len(), 2);
        assert!(planned.occupancies.iter().all(|occupancy| {
            matches!(occupancy.anchor, TacticalEquipmentAnchor::ItemAttachment { parent, .. } if parent == belt)
        }));
        for (index, point) in ["left", "right"].into_iter().enumerate() {
            spawn_test_item(
                &mut world,
                actor,
                "leather_satchel",
                EquipmentTopology {
                    authored_placement: Some("occupied".to_owned().try_into().unwrap()),
                    occupancies: vec![EquipmentTopologyOccupancy {
                        occupancy_id: format!("occupied-{point}"),
                        anchor: TacticalEquipmentAnchor::ItemAttachment {
                            parent: belt,
                            attachment_point_id: point.into(),
                        },
                        channel: EquipmentChannel::Mount,
                        order: index as u16,
                        requirement_index: 0,
                        capacity_index: 0,
                    }],
                },
            );
        }
        let incompatible = ReachableTarget::EmptyAttachment {
            parent: belt,
            attachment_point_id: "front".into(),
            channel: EquipmentChannel::Mount,
            capacity_index: 0,
        };
        assert!(
            world
                .run_system_once(move |items: Query<ItemView<'_>>| {
                    attachment_topology("sword_sheath", &incompatible, actor, &items)
                })
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn descendants_are_visible_to_the_reparenting_guard() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let parent = spawn_test_item(&mut world, actor, "belt", EquipmentTopology::default());
        spawn_test_item(
            &mut world,
            actor,
            "leather_satchel",
            EquipmentTopology {
                authored_placement: Some("belt_left".to_owned().try_into().unwrap()),
                occupancies: vec![EquipmentTopologyOccupancy {
                    occupancy_id: "edge".into(),
                    anchor: TacticalEquipmentAnchor::ItemAttachment {
                        parent,
                        attachment_point_id: "left".into(),
                    },
                    channel: EquipmentChannel::Mount,
                    order: 0,
                    requirement_index: 0,
                    capacity_index: 0,
                }],
            },
        );
        assert!(
            world
                .run_system_once(move |items: Query<ItemView<'_>>| has_children(parent, &items))
                .unwrap()
        );
    }

    #[test]
    fn item_and_terrain_layers_are_isolated() {
        let item = CollisionLayers::new(TACTICAL_ITEM_LAYER, TACTICAL_TERRAIN_LAYER);
        let terrain = CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL);
        assert!(item.interacts_with(terrain));
        assert!(!item.interacts_with(CollisionLayers::DEFAULT));
    }

    #[test]
    fn voiders_require_doublet_and_can_be_removed_and_laced_on_again() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        assert!(placement_topology("mail_voiders", EquipmentLocation::Chest).is_none());
        let doublet = spawn_test_item(
            &mut world,
            actor,
            "arming_doublet",
            placement_topology("arming_doublet", EquipmentLocation::Chest).unwrap(),
        );
        let target = ReachableTarget::EmptyAttachment {
            parent: doublet,
            attachment_point_id: "mail_voiders".into(),
            channel: EquipmentChannel::FlexibleArmor,
            capacity_index: 0,
        };
        let selected = target.clone();
        let topology = world
            .run_system_once(move |items: Query<ItemView<'_>>| {
                attachment_topology("mail_voiders", &selected, actor, &items)
            })
            .unwrap()
            .expect("doublet has lacing points");
        assert_eq!(topology.occupancies.len(), 1);
        let voiders = spawn_test_item(&mut world, actor, "mail_voiders", topology);
        assert!(
            world
                .run_system_once(move |items: Query<ItemView<'_>>| has_children(doublet, &items))
                .unwrap()
        );
        for location in [
            EquipmentLocation::Chest,
            EquipmentLocation::LeftArm,
            EquipmentLocation::RightArm,
        ] {
            let order = world
                .run_system_once(move |items: Query<ItemView<'_>>| {
                    ordered_at_location(actor, location, &items)
                })
                .unwrap();
            assert_eq!(order.first(), Some(&ReachableTarget::Occupied(voiders)));
        }
        // Taking off the detachable panels clears the edge, preserving the doublet.
        world
            .entity_mut(voiders)
            .insert(EquipmentTopology::default());
        assert!(
            !world
                .run_system_once(move |items: Query<ItemView<'_>>| has_children(doublet, &items))
                .unwrap()
        );
        let selected = target.clone();
        let topology = world
            .run_system_once(move |items: Query<ItemView<'_>>| {
                attachment_topology("mail_voiders", &selected, actor, &items)
            })
            .unwrap()
            .expect("removed panels can be laced on again");
        world.entity_mut(voiders).insert(topology);
        assert!(
            world
                .run_system_once(move |items: Query<ItemView<'_>>| has_children(doublet, &items))
                .unwrap()
        );
        world
            .entity_mut(voiders)
            .insert(EquipmentTopology::default());
        world.entity_mut(doublet).despawn();
        assert!(
            world
                .run_system_once(move |items: Query<ItemView<'_>>| {
                    attachment_topology("mail_voiders", &target, actor, &items)
                })
                .unwrap()
                .is_none(),
            "panels cannot be attached without the supporting garment"
        );
    }

    #[test]
    fn knee_voiders_follow_the_selected_hose_side_and_require_equipped_padding() {
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        for (location, placement) in [
            (EquipmentLocation::LeftLeg, "left"),
            (EquipmentLocation::RightLeg, "right"),
        ] {
            let hose = spawn_test_item(
                &mut world,
                actor,
                "padded_chausses",
                placement_topology("padded_chausses", location).unwrap(),
            );
            let selected = ReachableTarget::EmptyAttachment {
                parent: hose,
                attachment_point_id: "mail_knee_voider".into(),
                channel: EquipmentChannel::FlexibleArmor,
                capacity_index: 0,
            };
            let target = selected.clone();
            let topology = world
                .run_system_once(move |items: Query<ItemView<'_>>| {
                    attachment_topology("mail_knee_voider", &target, actor, &items)
                })
                .unwrap()
                .expect("matching hose supports a knee panel");
            assert_eq!(
                topology.authored_placement.as_ref().map(|id| id.as_str()),
                Some(placement)
            );
            assert!(matches!(&topology.occupancies[0].anchor,
                TacticalEquipmentAnchor::ItemAttachment { parent, .. } if *parent == hose));
            let knee = spawn_test_item(&mut world, actor, "mail_knee_voider", topology.clone());
            assert!(
                world
                    .run_system_once(move |items: Query<ItemView<'_>>| has_children(hose, &items))
                    .unwrap()
            );
            world.entity_mut(knee).insert(EquipmentTopology::default());
            let target = selected.clone();
            assert!(
                world
                    .run_system_once(move |items: Query<ItemView<'_>>| {
                        attachment_topology("mail_knee_voider", &target, actor, &items)
                    })
                    .unwrap()
                    .is_some()
            );
            world.entity_mut(hose).insert(EquipmentTopology::default());
            assert!(
                world
                    .run_system_once(move |items: Query<ItemView<'_>>| {
                        attachment_topology("mail_knee_voider", &selected, actor, &items)
                    })
                    .unwrap()
                    .is_none()
            );
        }
    }
}
