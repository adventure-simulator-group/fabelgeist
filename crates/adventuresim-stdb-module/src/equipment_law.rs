//! Centralized coarse settlement arms-and-armor compliance.

use adventuresim_core::organization::{Privilege, settlement_policy};
use spacetimedb::ReducerContext;

use crate::{
    CatalogItemKind,
    character::character,
    item::{inventory_item, item},
};

#[derive(Debug, thiserror::Error)]
pub enum EquipmentLawError {
    #[error("Character not found")]
    MissingCharacter,
    #[error("Inventory item not found")]
    MissingInventoryItem,
    #[error("Item definition not found")]
    MissingItemDefinition,
    #[error("{0} restricts armor; present a recognized organization with the right to wear armor")]
    ArmorProhibited(String),
    #[error("{0} restricts arms; present a recognized organization with the right to bear arms")]
    ArmsProhibited(String),
}

pub fn require_item_legal(
    ctx: &ReducerContext,
    character_id: u64,
    inventory_item_id: u64,
) -> Result<(), EquipmentLawError> {
    let Some(character) = ctx.db.character().id().find(character_id) else {
        return Err(EquipmentLawError::MissingCharacter);
    };
    let Some(settlement_id) = character.current_settlement_id else {
        return Ok(());
    };
    let Some(policy) = settlement_policy(&settlement_id) else {
        return Ok(());
    };
    let inventory = ctx
        .db
        .inventory_item()
        .character_and_id()
        .filter((character_id, inventory_item_id))
        .next()
        .ok_or(EquipmentLawError::MissingInventoryItem)?;
    let definition = ctx
        .db
        .item()
        .id()
        .find(&inventory.item_id)
        .ok_or(EquipmentLawError::MissingItemDefinition)?;
    let privilege = match definition.kind {
        CatalogItemKind::Weapon | CatalogItemKind::Shield if policy.restrict_arms => {
            Some(Privilege::BearArms)
        }
        CatalogItemKind::Armor if policy.restrict_armor => Some(Privilege::WearArmor),
        _ => None,
    };
    if privilege.is_some_and(|privilege| {
        !crate::organization::presented_privilege(ctx, character_id, &settlement_id, privilege)
    }) {
        return Err(match definition.kind {
            CatalogItemKind::Armor => EquipmentLawError::ArmorProhibited(settlement_id),
            _ => EquipmentLawError::ArmsProhibited(settlement_id),
        });
    }
    Ok(())
}

/// Auto-unequip prohibited items. Inventory ownership is unchanged.
pub fn enforce_equipment_compliance(
    ctx: &ReducerContext,
    character_id: u64,
) -> crate::StrategicOperationResult<Vec<u64>> {
    let equipped = crate::character::equipped_wearable_ids(ctx, character_id);
    let mut removed = Vec::new();
    for inventory_item_id in equipped {
        match require_item_legal(ctx, character_id, inventory_item_id) {
            Ok(()) => {}
            Err(EquipmentLawError::ArmorProhibited(_) | EquipmentLawError::ArmsProhibited(_)) => {
                removed.push(inventory_item_id);
            }
            Err(error) => return Err(error.into()),
        }
    }
    for &inventory_item_id in &removed {
        crate::character::unequip_wearable(ctx, inventory_item_id);
    }
    if !removed.is_empty() {
        crate::capability::refresh_character_capability(ctx, character_id)?;
        crate::condition::refresh_character_strategic_condition(ctx, character_id)?;
    }
    Ok(removed)
}
