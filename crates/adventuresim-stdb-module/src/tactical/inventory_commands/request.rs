//! Admit framework request values once, then dispatch the shared item effects.

use crate::{
    character::{EquipmentEffectPolicy, EquipmentPlacementChange, EquipmentRemoval},
    inventory_container::{
        ContainedItemRemoval, ContainerInsertion, ContainerWaterDisposal, inventory_object,
    },
    inventory_effects::PersonalInventoryTransfer,
};

use super::*;

pub(super) struct InventoryCommand {
    pub(super) id: InventoryCommandId,
    pub(super) actor: ExpectedActor,
    pub(super) effect: InventoryEffect,
    pub(super) digest: PayloadDigest,
}

pub(super) struct ExpectedActor {
    pub(super) id: CustodyCharacterId,
    pub(super) token: InventorySnapshotToken,
}

pub(super) enum InventoryEffect {
    PlaceEquipment {
        inventory: InventoryRowId,
        placement: EquipmentPlacementIndex,
        targets: Vec<crate::character::EquipmentAttachmentTargetSelection>,
        conflicts: EquipmentConflictPolicy,
    },
    RemoveEquipment(InventoryRowId),
    Transfer {
        inventory: InventoryRowId,
        quantity: ItemQuantity,
        recipient: ExpectedActor,
    },
    InsertIntoContainer {
        child: PhysicalObjectId,
        parent: PhysicalObjectId,
    },
    RemoveFromContainer(PhysicalObjectId),
    DiscardWater {
        container: PhysicalObjectId,
        volume: Milliliters,
    },
}

impl InventoryCommand {
    pub(super) fn actors(&self) -> impl Iterator<Item = &ExpectedActor> {
        std::iter::once(&self.actor).chain(self.effect.peer())
    }
}

impl TryFrom<TacticalInventoryCommand> for InventoryCommand {
    type Error = String;

    fn try_from(request: TacticalInventoryCommand) -> Result<Self, Self::Error> {
        let digest = PayloadDigest::for_request(&request)?;
        Ok(Self {
            id: InventoryCommandId::try_from(request.command_id)
                .map_err(|error| error.to_string())?,
            actor: ExpectedActor::try_from(request.actor)?,
            effect: InventoryEffect::try_from(request.action)?,
            digest,
        })
    }
}

impl TryFrom<TacticalInventoryActorState> for ExpectedActor {
    type Error = String;

    fn try_from(value: TacticalInventoryActorState) -> Result<Self, Self::Error> {
        Ok(Self {
            id: CustodyCharacterId::try_new(value.character_id)
                .map_err(|error| error.to_string())?,
            token: InventorySnapshotToken {
                scope: InventoryScopeId::try_from(value.scope)
                    .map_err(|error| error.to_string())?,
                revision: InventoryRevision::try_from(value.revision)
                    .map_err(|error| error.to_string())?,
            },
        })
    }
}

impl TryFrom<TacticalInventoryAction> for InventoryEffect {
    type Error = String;

    fn try_from(value: TacticalInventoryAction) -> Result<Self, Self::Error> {
        Ok(match value {
            TacticalInventoryAction::PlaceEquipment(selection) => Self::PlaceEquipment {
                inventory: InventoryRowId::try_new(selection.inventory_item_id)
                    .map_err(|error| error.to_string())?,
                placement: EquipmentPlacementIndex::new(selection.placement_index),
                targets: selection.targets,
                conflicts: selection.conflicts,
            },
            TacticalInventoryAction::RemoveEquipment(inventory_item_id) => Self::RemoveEquipment(
                InventoryRowId::try_new(inventory_item_id).map_err(|error| error.to_string())?,
            ),
            TacticalInventoryAction::Transfer(transfer) => Self::Transfer {
                inventory: InventoryRowId::try_new(transfer.inventory_item_id)
                    .map_err(|error| error.to_string())?,
                quantity: ItemQuantity::new(transfer.quantity)
                    .ok_or("Transfer quantity must be positive")?,
                recipient: ExpectedActor::try_from(transfer.recipient)?,
            },
            TacticalInventoryAction::InsertIntoContainer(insertion) => Self::InsertIntoContainer {
                child: PhysicalObjectId::try_new(insertion.child_object_id)
                    .map_err(|error| error.to_string())?,
                parent: PhysicalObjectId::try_new(insertion.parent_object_id)
                    .map_err(|error| error.to_string())?,
            },
            TacticalInventoryAction::RemoveFromContainer(child_object_id) => {
                Self::RemoveFromContainer(
                    PhysicalObjectId::try_new(child_object_id)
                        .map_err(|error| error.to_string())?,
                )
            }
            TacticalInventoryAction::DiscardWater(disposal) => Self::DiscardWater {
                container: PhysicalObjectId::try_new(disposal.container_object_id)
                    .map_err(|error| error.to_string())?,
                volume: Milliliters::new(disposal.amount_ml),
            },
        })
    }
}

impl InventoryEffect {
    pub(super) fn peer(&self) -> Option<&ExpectedActor> {
        match self {
            Self::Transfer { recipient, .. } => Some(recipient),
            _ => None,
        }
    }

    pub(super) fn apply(
        &self,
        ctx: &ReducerContext,
        actor: CustodyCharacterId,
    ) -> CommandResult<()> {
        match self {
            Self::PlaceEquipment {
                inventory,
                placement,
                targets,
                conflicts,
            } => EquipmentPlacementChange {
                actor,
                inventory: *inventory,
                placement: *placement,
                targets: targets.clone(),
                policy: EquipmentEffectPolicy::Gameplay,
                conflicts: *conflicts,
            }
            .apply(ctx),
            Self::RemoveEquipment(inventory) => EquipmentRemoval {
                actor,
                inventory: *inventory,
            }
            .apply(ctx),
            Self::Transfer {
                inventory,
                quantity,
                recipient,
            } => {
                let source = ctx
                    .db
                    .inventory_item()
                    .id()
                    .find(inventory.get())
                    .filter(|row| row.character_id == actor.get())
                    .ok_or("Source inventory item is not carried by this character")?;
                let recipient = ctx
                    .db
                    .character()
                    .id()
                    .find(recipient.id.get())
                    .ok_or("Recipient character is missing")?;
                PersonalInventoryTransfer {
                    source,
                    recipient,
                    quantity: *quantity,
                }
                .apply(ctx)
            }
            Self::InsertIntoContainer { child, parent } => {
                require_personal_object(ctx, actor, *child)?;
                require_personal_object(ctx, actor, *parent)?;
                ContainerInsertion {
                    actor,
                    child: *child,
                    parent: *parent,
                }
                .apply(ctx)
            }
            Self::RemoveFromContainer(child) => {
                require_personal_object(ctx, actor, *child)?;
                ContainedItemRemoval {
                    actor,
                    child: *child,
                }
                .apply(ctx)
            }
            Self::DiscardWater { container, volume } => {
                require_personal_object(ctx, actor, *container)?;
                ContainerWaterDisposal {
                    actor,
                    container: *container,
                    volume: *volume,
                }
                .apply(ctx)
            }
        }
    }
}

fn require_personal_object(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    id: PhysicalObjectId,
) -> CommandResult<()> {
    let object = ctx
        .db
        .inventory_object()
        .id()
        .find(id.get())
        .ok_or("Inventory object is missing")?;
    match object.location {
        InventoryLocation::Personal(location) if location.character_id == actor.get() => Ok(()),
        _ => Err("Live container commands require the actor's personal inventory".into()),
    }
}
