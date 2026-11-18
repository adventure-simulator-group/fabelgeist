//! Serialize checked equipment intents at the generated SDK boundary.

use adventuresim_core::equipment_command as core;

use crate::{
    AuthoredEquipmentPlacementRequest, EquipmentAttachmentTargetSelection, EquipmentChangeRequest,
    EquipmentChangeSetRequest, EquipmentChangesRequest, EquipmentConflictPolicy,
    EquipmentDestinationRequest, EquipmentExchangeRequest,
};

impl From<core::EquipmentIntent> for EquipmentChangeSetRequest {
    fn from(intent: core::EquipmentIntent) -> Self {
        Self {
            changes: intent.changes.into(),
            conflicts: match intent.conflicts {
                adventuresim_core::inventory_command::EquipmentConflictPolicy::RejectOccupied => {
                    EquipmentConflictPolicy::RejectOccupied
                }
                adventuresim_core::inventory_command::EquipmentConflictPolicy::ReplaceOccupied => {
                    EquipmentConflictPolicy::ReplaceOccupied
                }
            },
        }
    }
}

impl From<core::EquipmentChanges> for EquipmentChangesRequest {
    fn from(changes: core::EquipmentChanges) -> Self {
        match changes {
            core::EquipmentChanges::Single(change) => Self::Single(change.into()),
            core::EquipmentChanges::Exchange(exchange) => {
                Self::Exchange(EquipmentExchangeRequest {
                    source: exchange.source.into(),
                    destination: exchange.destination.into(),
                })
            }
        }
    }
}

impl From<core::EquipmentChange> for EquipmentChangeRequest {
    fn from(change: core::EquipmentChange) -> Self {
        Self {
            inventory_item_id: change.inventory.get(),
            destination: change.destination.into(),
        }
    }
}

impl From<core::EquipmentDestination> for EquipmentDestinationRequest {
    fn from(destination: core::EquipmentDestination) -> Self {
        match destination {
            core::EquipmentDestination::Inventory => Self::Inventory,
            core::EquipmentDestination::Held(hand) => Self::Held(hand.into()),
            core::EquipmentDestination::Authored(authored) => {
                Self::Authored(AuthoredEquipmentPlacementRequest {
                    placement_index: authored.placement.get(),
                    targets: authored
                        .targets
                        .iter()
                        .map(|target| EquipmentAttachmentTargetSelection {
                            requirement_index: target.requirement.get(),
                            parent_inventory_item_id: target.parent.get(),
                            attachment_point_id: target.point.as_str().to_owned(),
                        })
                        .collect(),
                })
            }
        }
    }
}
