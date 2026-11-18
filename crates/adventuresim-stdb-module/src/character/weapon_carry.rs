//! Physical carry-kit assembly for a generated character loadout.

use adventuresim_core::{
    equipment_command::{
        EquipmentChange, EquipmentChanges, EquipmentDestination, ParentSelection, ParentSelections,
    },
    equipment_placement::{EquipmentAttachmentPointId, EquipmentParentRequirementIndex},
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
};

use super::*;
use adventuresim_core::item_references::{
    BELT_ID, BLADE_ATTACHMENT_POINT_ID, BLADE_SHEATH_ID, HAFT_LOOP_ID, WEAPON_CARRY_HIP_PLACEMENTS,
    WORN_PLACEMENT_ID,
};

/// Adds the physical carry kit implied by a generated loadout. A suitable
/// weapon already authored in a hand stays there; a suitable sidearm that was
/// not initially equipped is placed inside the generated sheath.
pub(super) fn provision_generated_weapon_carry(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<(), String> {
    let sheathable: Vec<_> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter(|inventory| {
            adventuresim_core::item_catalog::is_sheathable_weapon(&inventory.item_id)
        })
        .collect();
    if sheathable.is_empty() {
        return Ok(());
    }
    if sheathable.len() > 2 {
        return Err(format!(
            "Generated loadout has {} sheathable weapons but the authored belt kit carries at most two",
            sheathable.len()
        ));
    }
    crate::strategic::require_character_no_unresolved_encounter(ctx, character_id)?;
    let actor = CustodyCharacterId::try_new(character_id).map_err(|error| error.to_string())?;

    let belt = InventoryRowId::try_new(
        add_inventory_item(ctx, character_id, BELT_ID, 1).ok_or("Could not add generated belt")?,
    )
    .map_err(|error| error.to_string())?;
    let belt_placement = authored_placement_index(ctx, BELT_ID, WORN_PLACEMENT_ID)?;
    EquipmentChangeSet::new(
        actor,
        EquipmentChanges::Single(EquipmentChange {
            inventory: belt,
            destination: EquipmentDestination::authored(belt_placement, ParentSelections::empty()),
        }),
        EquipmentEffectPolicy::LoadoutAssembly,
        EquipmentConflictPolicy::RejectOccupied,
    )
    .apply(ctx)?;

    for (weapon, hip) in sheathable.iter().zip(WEAPON_CARRY_HIP_PLACEMENTS) {
        let holder_id = match adventuresim_weapon_model::recommended_holder(&weapon.item_id) {
            Some(adventuresim_weapon_model::WeaponHolderKind::BladeSheath) => BLADE_SHEATH_ID,
            Some(adventuresim_weapon_model::WeaponHolderKind::HaftLoop) => HAFT_LOOP_ID,
            None => {
                return Err(format!(
                    "Sheathable weapon {} has no procedural holder",
                    weapon.item_id
                ));
            }
        };
        let holder = InventoryRowId::try_new(
            add_inventory_item(ctx, character_id, holder_id, 1)
                .ok_or_else(|| format!("Could not add generated {holder_id}"))?,
        )
        .map_err(|error| error.to_string())?;
        crate::weapon_instance::fit_personal_holder(ctx, character_id, holder.get(), weapon.id)?;
        let holder_placement = authored_placement_index(ctx, holder_id, hip)?;
        EquipmentChangeSet::new(
            actor,
            EquipmentChanges::Single(EquipmentChange {
                inventory: holder,
                destination: EquipmentDestination::authored(
                    holder_placement,
                    ParentSelection {
                        requirement: EquipmentParentRequirementIndex::new(0),
                        parent: belt,
                        point: EquipmentAttachmentPointId::try_from(hip.to_owned())
                            .map_err(|error| error.to_string())?,
                    }
                    .into(),
                ),
            }),
            EquipmentEffectPolicy::LoadoutAssembly,
            EquipmentConflictPolicy::RejectOccupied,
        )
        .apply(ctx)?;
        place_unheld_weapon_in_sheath(ctx, actor, weapon, holder)?;
    }
    Ok(())
}

pub(super) fn select_sheath_compatible_parent_placement(
    placements: &[crate::item::PersistedEquipmentPlacement],
) -> Option<EquipmentPlacementIndex> {
    placements
        .iter()
        .position(|placement| {
            placement.parents.len() == 1
                && placement.parents[0].channel == EquipmentChannel::Containment
                && placement.parents[0].order == 0
        })
        .and_then(|index| u16::try_from(index).ok())
        .map(EquipmentPlacementIndex::new)
}

fn place_unheld_weapon_in_sheath(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    weapon: &crate::item::InventoryItem,
    sheath: InventoryRowId,
) -> Result<(), String> {
    if ctx
        .db
        .character_equipped_item()
        .inventory_item_id()
        .find(weapon.id)
        .is_some()
    {
        return Ok(());
    }
    let placement = sheath_compatible_parent_placement_index(ctx, &weapon.item_id)?;
    EquipmentChangeSet::new(
        actor,
        EquipmentChanges::Single(EquipmentChange {
            inventory: InventoryRowId::try_new(weapon.id).map_err(|error| error.to_string())?,
            destination: EquipmentDestination::authored(
                placement,
                ParentSelection {
                    requirement: EquipmentParentRequirementIndex::new(0),
                    parent: sheath,
                    point: EquipmentAttachmentPointId::try_from(
                        BLADE_ATTACHMENT_POINT_ID.to_owned(),
                    )
                    .map_err(|error| error.to_string())?,
                }
                .into(),
            ),
        }),
        EquipmentEffectPolicy::LoadoutAssembly,
        EquipmentConflictPolicy::RejectOccupied,
    )
    .apply(ctx)
}

fn authored_placement_index(
    ctx: &ReducerContext,
    item_id: &str,
    placement_id: &str,
) -> Result<EquipmentPlacementIndex, String> {
    let definition = ctx
        .db
        .item()
        .id()
        .find(item_id.to_owned())
        .ok_or_else(|| format!("Generated carry kit item {item_id} is missing"))?;
    definition
        .equipment_placements
        .iter()
        .position(|placement| placement.id == placement_id)
        .and_then(|index| u16::try_from(index).ok())
        .map(EquipmentPlacementIndex::new)
        .ok_or_else(|| format!("{item_id} lacks authored placement {placement_id}"))
}

fn sheath_compatible_parent_placement_index(
    ctx: &ReducerContext,
    item_id: &str,
) -> Result<EquipmentPlacementIndex, String> {
    let definition = ctx
        .db
        .item()
        .id()
        .find(item_id.to_owned())
        .ok_or_else(|| format!("Generated sidearm {item_id} is missing"))?;
    select_sheath_compatible_parent_placement(&definition.equipment_placements).ok_or_else(|| {
        format!("Sheathable sidearm {item_id} lacks a single containment parent placement")
    })
}

pub(super) fn require_weapon_carry_placement(
    item_id: &str,
    placement: &crate::item::PersistedEquipmentPlacement,
) -> Result<(), String> {
    match adventuresim_core::item_catalog::weapon_carry(item_id) {
        Some(adventuresim_core::item_catalog::WeaponCarry::HandOnly) => {
            return Err(format!(
                "Hand-only weapon {} requires a generic hand destination",
                item_id
            ));
        }
        Some(adventuresim_core::item_catalog::WeaponCarry::Sheathable) => {}
        _ if !placement.parents.is_empty() => {
            return Err(format!(
                "Can't attach weapon {} without an authored sheathable carry contract",
                item_id
            ));
        }
        _ => {}
    }
    Ok(())
}
