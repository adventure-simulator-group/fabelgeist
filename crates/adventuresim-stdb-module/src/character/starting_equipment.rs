//! Assemble a complete character loadout before its owner refreshes derived state.

use adventuresim_core::{
    equipment::LoadoutSlot,
    equipment_placement::EquipmentHand,
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
    physical_object::CustodyCharacterId,
    starting_character::StartingCharacterSpec,
};
use spacetimedb::ReducerContext;

use super::{
    EquipmentChange, EquipmentChangeSet, EquipmentChanges, EquipmentDestination,
    EquipmentEffectPolicy, equipped_wearable_ids, refresh_equipment_dependents, unequip_wearable,
};
use crate::{add_inventory_item, inventory_item, item::item};

pub(super) fn assemble_starting_inventory(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    starting: Option<&StartingCharacterSpec>,
    newborn: bool,
) -> Result<(), String> {
    if let Some(spec) = starting {
        for item in &spec.inventory {
            if let Some(slot) = item.equipped {
                add_and_equip_starting_item(ctx, actor, &item.item_id, slot)?;
                if item.quantity > 1 {
                    add_inventory_item(ctx, actor.get(), &item.item_id, item.quantity - 1);
                }
            } else {
                add_inventory_item(ctx, actor.get(), &item.item_id, item.quantity);
            }
        }
    } else if !newborn {
        add_inventory_item(ctx, actor.get(), "torch", 1);
        add_inventory_item(ctx, actor.get(), "bandage", 3);
        assemble_basic_clothing(ctx, actor)?;
        for (item, slot) in [
            ("buckler", LoadoutSlot::LeftHand),
            ("katzbalger", LoadoutSlot::RightHand),
            ("arming_cap", LoadoutSlot::Head),
            ("arming_doublet", LoadoutSlot::Chest),
            ("padded_chausses", LoadoutSlot::LeftLeg),
            ("padded_chausses", LoadoutSlot::RightLeg),
        ] {
            add_and_equip_starting_item(ctx, actor, item, slot)?;
        }
    }
    Ok(())
}

pub(crate) fn add_and_equip_basic_clothing(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<(), String> {
    let actor = CustodyCharacterId::try_new(character_id).map_err(|error| error.to_string())?;
    assemble_basic_clothing(ctx, actor)?;
    refresh_equipment_dependents(ctx, actor.get())
}

/// Replace a development character's equipped roots without losing its objects.
pub(crate) fn replace_development_loadout(
    ctx: &ReducerContext,
    character_id: u64,
    loadout: &[(&str, LoadoutSlot)],
) -> Result<(), String> {
    let actor = CustodyCharacterId::try_new(character_id).map_err(|error| error.to_string())?;
    crate::strategic::require_character_no_unresolved_encounter(ctx, actor.get())?;
    for inventory in equipped_wearable_ids(ctx, actor.get()) {
        unequip_wearable(ctx, inventory);
    }
    let mut selected = std::collections::BTreeSet::new();
    for (item, slot) in loadout {
        let inventory = ctx
            .db
            .inventory_item()
            .character_and_item_id()
            .filter((actor.get(), *item))
            .find(|row| !selected.contains(&row.id))
            .map(|row| row.id)
            .or_else(|| add_inventory_item(ctx, actor.get(), item, 1))
            .ok_or_else(|| format!("Failed to add {item} to development loadout"))?;
        selected.insert(inventory);
        EquipmentChangeSet::new(
            actor,
            EquipmentChanges::Single(EquipmentChange {
                inventory: InventoryRowId::try_new(inventory).map_err(|error| error.to_string())?,
                destination: starting_item_destination(ctx, item, *slot)?,
            }),
            EquipmentEffectPolicy::LoadoutAssembly,
            EquipmentConflictPolicy::RejectOccupied,
        )
        .apply(ctx)?;
    }
    refresh_equipment_dependents(ctx, actor.get())
}

fn starting_item_destination(
    ctx: &ReducerContext,
    item: &str,
    slot: LoadoutSlot,
) -> Result<EquipmentDestination, String> {
    use adventuresim_core::item_catalog::EquipmentLocation;

    match slot {
        LoadoutSlot::LeftHand => return Ok(EquipmentDestination::Held(EquipmentHand::Left)),
        LoadoutSlot::RightHand => return Ok(EquipmentDestination::Held(EquipmentHand::Right)),
        _ => {}
    }
    let location = EquipmentLocation::from(slot);
    let definition = ctx
        .db
        .item()
        .id()
        .find(item.to_owned())
        .ok_or_else(|| format!("Starting item {item} is missing"))?;
    let index = definition
        .equipment_placements
        .iter()
        .position(|placement| {
            placement.parents.is_empty()
                && placement
                    .occupancy
                    .iter()
                    .any(|root| root.location == location)
        })
        .and_then(|index| u16::try_from(index).ok())
        .ok_or_else(|| format!("Starting item {item} lacks a root placement for {slot:?}"))?;
    EquipmentDestination::authored(EquipmentPlacementIndex::new(index), Vec::new())
}

fn add_and_equip_starting_item(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    item: &str,
    slot: LoadoutSlot,
) -> Result<(), String> {
    crate::strategic::require_character_no_unresolved_encounter(ctx, actor.get())?;
    let row = add_inventory_item(ctx, actor.get(), item, 1)
        .ok_or("Could not add starting inventory item")?;
    EquipmentChangeSet::new(
        actor,
        EquipmentChanges::Single(EquipmentChange {
            inventory: InventoryRowId::try_new(row).map_err(|error| error.to_string())?,
            destination: starting_item_destination(ctx, item, slot)?,
        }),
        EquipmentEffectPolicy::LoadoutAssembly,
        EquipmentConflictPolicy::RejectOccupied,
    )
    .apply(ctx)
}

fn assemble_basic_clothing(ctx: &ReducerContext, actor: CustodyCharacterId) -> Result<(), String> {
    for (item, slot) in [
        ("linen_tunic", LoadoutSlot::Chest),
        ("linen_breeches", LoadoutSlot::LeftLeg),
        ("leather_boot", LoadoutSlot::LeftFoot),
        ("leather_boot", LoadoutSlot::RightFoot),
    ] {
        add_and_equip_starting_item(ctx, actor, item, slot)?;
    }
    Ok(())
}
