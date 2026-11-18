//! Restore saved equipment through the canonical intent and effect authority.

use adventuresim_core::{
    equipment_command::{
        EquipmentChange, EquipmentChanges, EquipmentDestination, ParentSelection, ParentSelections,
    },
    equipment_placement::EquippedPlacement,
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
    physical_object::CustodyCharacterId,
};

use super::*;

pub(crate) fn restore_equipment_placement(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    inventory_id: InventoryRowId,
    placement: &EquippedPlacement,
    targets: Vec<EquipmentAttachmentTargetSelection>,
) -> Result<(), String> {
    crate::strategic::require_character_no_unresolved_encounter(ctx, actor.get())?;
    let inventory = ctx
        .db
        .inventory_item()
        .id()
        .find(inventory_id.get())
        .ok_or("Inventory item not found while restoring equipment")?;
    let definition = ctx
        .db
        .item()
        .id()
        .find(&inventory.item_id)
        .ok_or("Item definition not found while restoring equipment")?;
    let destination = match placement {
        EquippedPlacement::Authored(id) => {
            let index = definition
                .equipment_placements
                .iter()
                .position(|placement| placement.id == id.as_str())
                .and_then(|index| u16::try_from(index).ok())
                .ok_or("Saved equipment placement is no longer authored")?;
            EquipmentDestination::authored(
                EquipmentPlacementIndex::new(index),
                ParentSelections::try_from(
                    targets
                        .into_iter()
                        .map(ParentSelection::try_from)
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|error| error.to_string())?,
            )
        }
        EquippedPlacement::Held(hand) => {
            if !targets.is_empty() {
                return Err("A held item cannot have saved attachment targets".into());
            }
            EquipmentDestination::Held(*hand)
        }
    };
    EquipmentChangeSet::new(
        actor,
        EquipmentChanges::Single(EquipmentChange {
            inventory: inventory_id,
            destination,
        }),
        EquipmentEffectPolicy::Gameplay,
        EquipmentConflictPolicy::RejectOccupied,
    )
    .apply(ctx)
}
