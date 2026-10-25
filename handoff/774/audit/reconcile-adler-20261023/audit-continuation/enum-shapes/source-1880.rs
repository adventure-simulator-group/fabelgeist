//! Replication-stable ordering of body-root equipment. Entity allocation order
//! is deliberately absent: a client remaps every server entity independently.
use super::{EquipmentTopology, TacticalEquipmentAnchor};
use adventuresim_core::item_catalog::{self, EquipmentFitZone, EquipmentLocation};
use std::cmp::Reverse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquipmentRootOrder<'a> {
    channel: Reverse<u8>,
    layer: Reverse<u16>,
    fit_zone: Option<EquipmentFitZone>,
    item_id: &'a str,
    placement_id: Option<&'a str>,
}

impl EquipmentTopology {
    /// Outermost layers first, then anatomical fit zone and authored identity.
    /// Valid equipped roots cannot share a location/channel/order/fit zone;
    /// catalog identity supplies a stable secondary key without network IDs.
    pub fn root_order<'a>(
        &'a self,
        location: EquipmentLocation,
        item_id: &'a str,
    ) -> Option<EquipmentRootOrder<'a>> {
        let placement = item_catalog::definition(item_id)
            .and_then(|definition| definition.equipment.as_ref())
            .and_then(|equipment| {
                equipment
                    .placements
                    .iter()
                    .find(|placement| Some(placement.id.as_str()) == self.placement_id.as_deref())
            });
        self.occupancies
            .iter()
            .filter_map(|occupancy| {
                if occupancy.anchor != TacticalEquipmentAnchor::CharacterLocation(location) {
                    return None;
                }
                let fit_zone = placement
                    .and_then(|placement| {
                        placement
                            .occupancy
                            .get(usize::from(occupancy.requirement_index))
                    })
                    .filter(|requirement| {
                        requirement.location == location && requirement.channel == occupancy.channel
                    })
                    .and_then(|requirement| requirement.fit_zone);
                Some(EquipmentRootOrder {
                    channel: Reverse(occupancy.channel.order()),
                    layer: Reverse(occupancy.order),
                    fit_zone,
                    item_id,
                    placement_id: self.placement_id.as_deref(),
                })
            })
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::EquipmentTopologyOccupancy;
    use bevy::prelude::Entity;

    #[test]
    fn anatomical_depth_survives_reversed_network_entity_remapping() {
        let location = EquipmentLocation::LeftArm;
        let roots = ["rerebrace", "couter", "vambrace"].map(|item_id| {
            let placement = &item_catalog::definition(item_id)
                .unwrap()
                .equipment
                .as_ref()
                .unwrap()
                .placements[0];
            let topology = EquipmentTopology {
                placement_id: Some(placement.id.clone()),
                occupancies: placement
                    .occupancy
                    .iter()
                    .enumerate()
                    .map(|(index, requirement)| EquipmentTopologyOccupancy {
                        occupancy_id: format!("{item_id}:{index}"),
                        anchor: TacticalEquipmentAnchor::CharacterLocation(requirement.location),
                        channel: requirement.channel,
                        order: requirement.order,
                        requirement_index: index as u16,
                        capacity_index: 0,
                    })
                    .collect(),
            };
            (item_id, topology)
        });
        let server_entities = [
            Entity::from_bits(10),
            Entity::from_bits(20),
            Entity::from_bits(30),
        ];
        let client_entities = [
            Entity::from_bits(300),
            Entity::from_bits(200),
            Entity::from_bits(100),
        ];
        let order = |entities: [Entity; 3]| {
            let mut local_roots = roots.iter().zip(entities).collect::<Vec<_>>();
            local_roots
                .sort_by_key(|((item_id, topology), _)| topology.root_order(location, item_id));
            local_roots
                .into_iter()
                .map(|(_, entity)| entity)
                .collect::<Vec<_>>()
        };
        let mapped_server_order = order(server_entities)
            .into_iter()
            .map(|entity| {
                client_entities[server_entities
                    .iter()
                    .position(|server| *server == entity)
                    .unwrap()]
            })
            .collect::<Vec<_>>();
        assert_eq!(order(client_entities), mapped_server_order);
        assert_eq!(
            order(client_entities),
            client_entities,
            "fit zones traverse upper arm, elbow, forearm"
        );
    }
}
