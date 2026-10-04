//! Equipment attachment topology and atomic occupancy admission.
use crate::identity::InventoryItemId;
use crate::item_catalog_schema::OccupancyRequirement;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentGraphError {
    HasChildren,
    DuplicateBodyOccupancy,
    DuplicateAttachmentCapacity,
    BodyOccupancyConflict,
    AttachmentCapacityConflict,
    ParentNotEquipped,
    AttachmentCycle,
}
impl std::fmt::Display for EquipmentGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::HasChildren => "item has equipped children",
            Self::DuplicateBodyOccupancy => "duplicate body occupancy",
            Self::DuplicateAttachmentCapacity => "duplicate attachment capacity",
            Self::BodyOccupancyConflict => "body occupancy conflict",
            Self::AttachmentCapacityConflict => "attachment capacity conflict",
            Self::ParentNotEquipped => "parent is not equipped",
            Self::AttachmentCycle => "attachment cycle",
        })
    }
}
impl std::error::Error for EquipmentGraphError {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EquipmentGraph {
    pub nodes: BTreeMap<InventoryItemId, EquipmentGraphPlacement>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EquipmentGraphPlacement {
    pub body: Vec<OccupancyRequirement>,
    pub parents: Vec<EquipmentGraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EquipmentGraphEdge {
    pub parent_inventory_item_id: InventoryItemId,
    pub attachment_point_id: String,
    pub capacity_index: u16,
}

impl EquipmentGraph {
    pub fn equip(
        &mut self,
        inventory_item_id: InventoryItemId,
        mut placement: EquipmentGraphPlacement,
    ) -> Result<(), EquipmentGraphError> {
        if self.has_children(inventory_item_id) {
            return Err(EquipmentGraphError::HasChildren);
        }
        for requirement in &mut placement.body {
            if requirement.channel.singleton_per_location() {
                requirement.order = 0;
            }
        }
        if placement
            .body
            .iter()
            .enumerate()
            .any(|(index, requirement)| {
                placement.body[..index]
                    .iter()
                    .any(|other| requirement.conflicts_with(*other))
            })
        {
            return Err(EquipmentGraphError::DuplicateBodyOccupancy);
        }
        let edge_keys = placement.parents.iter().cloned().collect::<BTreeSet<_>>();
        if edge_keys.len() != placement.parents.len() {
            return Err(EquipmentGraphError::DuplicateAttachmentCapacity);
        }
        for (other_id, other) in &self.nodes {
            if *other_id == inventory_item_id {
                continue;
            }
            if other.body.iter().any(|cell| {
                placement
                    .body
                    .iter()
                    .any(|requirement| requirement.conflicts_with(*cell))
            }) {
                return Err(EquipmentGraphError::BodyOccupancyConflict);
            }
            if other.parents.iter().any(|edge| edge_keys.contains(edge)) {
                return Err(EquipmentGraphError::AttachmentCapacityConflict);
            }
        }
        if placement
            .parents
            .iter()
            .any(|edge| !self.nodes.contains_key(&edge.parent_inventory_item_id))
        {
            return Err(EquipmentGraphError::ParentNotEquipped);
        }
        if self.would_cycle(
            inventory_item_id,
            placement
                .parents
                .iter()
                .map(|edge| edge.parent_inventory_item_id),
        ) {
            return Err(EquipmentGraphError::AttachmentCycle);
        }
        self.nodes.insert(inventory_item_id, placement);
        Ok(())
    }

    pub fn unequip(
        &mut self,
        inventory_item_id: InventoryItemId,
    ) -> Result<(), EquipmentGraphError> {
        if self.has_children(inventory_item_id) {
            return Err(EquipmentGraphError::HasChildren);
        }
        self.nodes.remove(&inventory_item_id);
        Ok(())
    }

    pub fn has_children(&self, inventory_item_id: InventoryItemId) -> bool {
        self.nodes.values().any(|placement| {
            placement
                .parents
                .iter()
                .any(|edge| edge.parent_inventory_item_id == inventory_item_id)
        })
    }

    fn would_cycle(
        &self,
        inventory_item_id: InventoryItemId,
        parents: impl IntoIterator<Item = InventoryItemId>,
    ) -> bool {
        let mut ancestors = parents.into_iter().collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        while let Some(ancestor) = ancestors.pop() {
            if ancestor == inventory_item_id {
                return true;
            }
            if visited.insert(ancestor)
                && let Some(placement) = self.nodes.get(&ancestor)
            {
                ancestors.extend(
                    placement
                        .parents
                        .iter()
                        .map(|edge| edge.parent_inventory_item_id),
                );
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_catalog_schema::{EquipmentChannel, EquipmentLocation};
    #[test]
    fn graph_supports_belt_sheath_sword_and_body_bag_contents() {
        let edge = |parent, point: &str, capacity_index| EquipmentGraphEdge {
            parent_inventory_item_id: InventoryItemId::new(parent),
            attachment_point_id: point.into(),
            capacity_index,
        };
        let mut graph = EquipmentGraph::default();
        graph
            .equip(
                InventoryItemId::new(1),
                EquipmentGraphPlacement {
                    body: vec![OccupancyRequirement {
                        location: EquipmentLocation::LeftBelt,
                        channel: EquipmentChannel::Accessory,
                        order: 0,
                        fit_zone: None,
                    }],
                    parents: vec![],
                },
            )
            .unwrap();
        graph
            .equip(
                InventoryItemId::new(2),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(1, "left", 0), edge(1, "right", 0)],
                },
            )
            .unwrap();
        graph
            .equip(
                InventoryItemId::new(3),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(2, "blade", 0)],
                },
            )
            .unwrap();
        graph
            .equip(
                InventoryItemId::new(4),
                EquipmentGraphPlacement {
                    body: vec![OccupancyRequirement {
                        location: EquipmentLocation::LeftShoulder,
                        channel: EquipmentChannel::Accessory,
                        order: 0,
                        fit_zone: None,
                    }],
                    parents: vec![],
                },
            )
            .unwrap();
        graph
            .equip(
                InventoryItemId::new(5),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(4, "contents", 0)],
                },
            )
            .unwrap();
        assert_eq!(graph.nodes.len(), 5);
        assert!(graph.unequip(InventoryItemId::new(1)).is_err());
        graph.unequip(InventoryItemId::new(3)).unwrap();
        graph.unequip(InventoryItemId::new(2)).unwrap();
        graph.unequip(InventoryItemId::new(1)).unwrap();
    }

    #[test]
    fn graph_multi_point_move_is_atomic_and_cycle_safe() {
        let edge = |parent, point: &str, capacity_index| EquipmentGraphEdge {
            parent_inventory_item_id: InventoryItemId::new(parent),
            attachment_point_id: point.into(),
            capacity_index,
        };
        let mut graph = EquipmentGraph::default();
        for id in [10, 11] {
            graph
                .equip(
                    InventoryItemId::new(id),
                    EquipmentGraphPlacement {
                        body: vec![OccupancyRequirement {
                            location: if id == 10 {
                                EquipmentLocation::LeftShoulder
                            } else {
                                EquipmentLocation::RightShoulder
                            },
                            channel: EquipmentChannel::Mount,
                            order: 0,
                            fit_zone: None,
                        }],
                        parents: vec![],
                    },
                )
                .unwrap();
        }
        graph
            .equip(
                InventoryItemId::new(20),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(10, "strap", 0), edge(11, "strap", 0)],
                },
            )
            .unwrap();
        let before = graph.clone();
        assert_eq!(
            graph.equip(
                InventoryItemId::new(21),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(10, "strap", 0), edge(11, "strap", 1)],
                },
            ),
            Err(EquipmentGraphError::AttachmentCapacityConflict)
        );
        assert_eq!(graph, before, "failed preflight never partially mutates");
        assert_eq!(
            graph.equip(
                InventoryItemId::new(10),
                EquipmentGraphPlacement {
                    body: vec![],
                    parents: vec![edge(20, "loop", 0)],
                },
            ),
            Err(EquipmentGraphError::HasChildren)
        );
    }

    #[test]
    fn singleton_wearable_orders_conflict_but_accessory_coexists() {
        let mut graph = EquipmentGraph::default();
        graph
            .equip(
                InventoryItemId::new(1),
                EquipmentGraphPlacement {
                    body: vec![OccupancyRequirement {
                        location: EquipmentLocation::Chest,
                        channel: EquipmentChannel::RigidArmor,
                        order: 0,
                        fit_zone: None,
                    }],
                    parents: vec![],
                },
            )
            .unwrap();
        assert_eq!(
            graph.equip(
                InventoryItemId::new(2),
                EquipmentGraphPlacement {
                    body: vec![OccupancyRequirement {
                        location: EquipmentLocation::Chest,
                        channel: EquipmentChannel::RigidArmor,
                        order: 1,
                        fit_zone: None
                    }],
                    parents: vec![],
                },
            ),
            Err(EquipmentGraphError::BodyOccupancyConflict)
        );
        graph
            .equip(
                InventoryItemId::new(3),
                EquipmentGraphPlacement {
                    body: vec![OccupancyRequirement {
                        location: EquipmentLocation::Chest,
                        channel: EquipmentChannel::Accessory,
                        order: 0,
                        fit_zone: None,
                    }],
                    parents: vec![],
                },
            )
            .unwrap();
    }
}
