use super::ItemView;
use adventuresim_core::item_catalog::{self, OccupancyRequirement};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

fn authored_requirement(
    item_id: &str,
    topology: &EquipmentTopology,
    occupancy: &EquipmentTopologyOccupancy,
) -> OccupancyRequirement {
    let TacticalEquipmentAnchor::CharacterLocation(location) = occupancy.anchor else {
        unreachable!("only character anchors have fit zones")
    };
    let fit_zone = item_catalog::definition(item_id)
        .and_then(|definition| definition.equipment.as_ref())
        .and_then(|equipment| {
            equipment
                .placements
                .iter()
                .find(|placement| Some(&placement.id) == topology.placement_id.as_ref())
        })
        .and_then(|placement| {
            placement
                .occupancy
                .get(usize::from(occupancy.requirement_index))
        })
        .filter(|requirement| {
            requirement.location == location && requirement.channel == occupancy.channel
        })
        .and_then(|requirement| requirement.fit_zone);
    OccupancyRequirement {
        location,
        channel: occupancy.channel,
        order: occupancy.order,
        fit_zone,
    }
}

pub(super) fn occupancies_conflict(
    candidate_item_id: &str,
    candidate_topology: &EquipmentTopology,
    candidate: &EquipmentTopologyOccupancy,
    current_item_id: &str,
    current_topology: &EquipmentTopology,
    current: &EquipmentTopologyOccupancy,
) -> bool {
    if matches!(
        candidate.anchor,
        TacticalEquipmentAnchor::CharacterLocation(_)
    ) && matches!(
        current.anchor,
        TacticalEquipmentAnchor::CharacterLocation(_)
    ) {
        authored_requirement(candidate_item_id, candidate_topology, candidate).conflicts_with(
            authored_requirement(current_item_id, current_topology, current),
        )
    } else {
        super::attachment_occupancies_conflict(candidate, current)
    }
}

pub(super) fn topology_conflicts(
    actor: Entity,
    proposed: &EquipmentTopology,
    proposed_item_id: &str,
    ignored: &[Entity],
    items: &Query<ItemView<'_>>,
) -> bool {
    items
        .iter()
        .any(|(entity, properties, owner, topology, _, _, scene, _)| {
            !scene
                && !ignored.contains(&entity)
                && owner.is_some_and(|owner| owner.0 == actor)
                && proposed.occupancies.iter().any(|candidate| {
                    topology.occupancies.iter().any(|current| {
                        occupancies_conflict(
                            proposed_item_id,
                            proposed,
                            candidate,
                            &properties.id,
                            topology,
                            current,
                        )
                    })
                })
        })
}

/// A body slot may already display a neighboring plate in the same channel.
/// Only an intersecting reservation is displaced by a transfer.
pub(super) fn target_conflicts(
    target: &super::ReachableTarget,
    proposed_item_id: &str,
    proposed: &EquipmentTopology,
    items: &Query<ItemView<'_>>,
) -> bool {
    let super::ReachableTarget::Occupied(entity) = target else {
        return true;
    };
    items
        .get(*entity)
        .is_ok_and(|(_, properties, _, topology, _, _, _, _)| {
            proposed.occupancies.iter().any(|candidate| {
                topology.occupancies.iter().any(|current| {
                    occupancies_conflict(
                        proposed_item_id,
                        proposed,
                        candidate,
                        &properties.id,
                        topology,
                        current,
                    )
                })
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tactical_harness_reservations_accept_neighbors_and_reject_duplicate_armor() {
        let mut harness: Vec<(&str, EquipmentTopology)> = Vec::new();
        for item in [
            "morion",
            "gorget",
            "cuirass",
            "fauld",
            "spaulder",
            "rerebrace",
            "couter",
            "vambrace",
            "mitten_gauntlet",
            "cuisse",
            "poleyn",
            "greave",
            "sabaton",
        ] {
            for placement in &item_catalog::definition(item)
                .unwrap()
                .equipment
                .as_ref()
                .unwrap()
                .placements
            {
                let topology =
                    super::super::placement_topology(item, placement.occupancy[0].location)
                        .unwrap();
                for (other_item, other_topology) in &harness {
                    assert!(
                        !topology.occupancies.iter().any(|candidate| other_topology
                            .occupancies
                            .iter()
                            .any(|current| occupancies_conflict(
                                item,
                                &topology,
                                candidate,
                                other_item,
                                other_topology,
                                current
                            ))),
                        "{item} conflicts with {other_item}"
                    );
                }
                assert!(occupancies_conflict(
                    item,
                    &topology,
                    &topology.occupancies[0],
                    item,
                    &topology,
                    &topology.occupancies[0]
                ));
                harness.push((item, topology));
            }
        }
        assert_eq!(harness.len(), 22);
    }
    #[test]
    fn slot_transfers_build_a_harness_without_swapping_neighbors_and_can_wear_gauntlets() {
        use super::super::{hand_topology, placement_topology, transfer_slot};
        use adventuresim_tactical_netcode::prelude::EquipmentHand;
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let actor = world.spawn_empty().id();
        let mut equipped = Vec::new();
        for (item, location) in [
            "morion",
            "gorget",
            "cuirass",
            "fauld",
            "spaulder",
            "rerebrace",
            "couter",
            "vambrace",
            "mitten_gauntlet",
            "cuisse",
            "poleyn",
            "greave",
            "sabaton",
        ]
        .into_iter()
        .flat_map(|item| {
            item_catalog::definition(item)
                .unwrap()
                .equipment
                .as_ref()
                .unwrap()
                .placements
                .iter()
                .map(move |placement| (item, placement.occupancy[0].location))
        }) {
            let entity = world
                .spawn((
                    ItemOf(actor),
                    ItemProperties {
                        id: item.into(),
                        weight: 1.0,
                    },
                    hand_topology(EquipmentHand::Right),
                    EquipmentHand::Right.slot(),
                ))
                .id();
            let transferred = world
                .run_system_once(move |mut commands: Commands, items: Query<ItemView<'_>>| {
                    transfer_slot(
                        &mut commands,
                        actor,
                        EquipmentHand::Right,
                        location,
                        0,
                        &items,
                    )
                })
                .unwrap();
            assert!(transferred, "{item}");
            assert_eq!(
                world.get::<EquipmentTopology>(entity),
                placement_topology(item, location).as_ref()
            );
            equipped.push(entity);
            for worn in &equipped {
                assert!(
                    world.get::<EquipSlot>(*worn).is_none(),
                    "neighbor was swapped while equipping {item}"
                );
            }
        }
        assert_eq!(equipped.len(), 22);
    }
}
