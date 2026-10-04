//! Authoritative recipe-driven smithing transactions.

mod consumption_error;
use adventuresim_core::{
    identity::CharacterId, inventory_measurement::MeasuredItemAmountMicros,
    item_catalog::ItemDefinitionId,
};
use adventuresim_weapon_model::decode;
use consumption_error::SmithingConsumptionError;
use spacetimedb::{ReducerContext, reducer};

use crate::character::character;
use crate::inventory_amount::inventory_item_amount;
use crate::item::{add_inventory_item_checked, inventory_item};

fn available(
    ctx: &ReducerContext,
    character_id: CharacterId,
    item_id: &ItemDefinitionId,
) -> MeasuredItemAmountMicros {
    ctx.db
        .inventory_item()
        .character_id()
        .filter(u64::from(character_id))
        .filter(|row| row.item_id == item_id.as_str())
        .filter_map(|row| {
            ctx.db
                .inventory_item_amount()
                .inventory_item_id()
                .find(row.id)
                .map(|amount| MeasuredItemAmountMicros::new(amount.remaining_fraction_micros))
        })
        .fold(
            MeasuredItemAmountMicros::ZERO,
            MeasuredItemAmountMicros::saturating_add,
        )
}

fn consume(
    ctx: &ReducerContext,
    character_id: CharacterId,
    item_id: &ItemDefinitionId,
    requested_micros: MeasuredItemAmountMicros,
) -> Result<(), SmithingConsumptionError> {
    let mut rows = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(u64::from(character_id))
        .filter(|row| row.item_id == item_id.as_str())
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.id);
    let mut remaining_micros = requested_micros;
    for row in rows {
        let requested_fraction = remaining_micros.row_request();
        let consumed =
            crate::inventory_amount::consume_personal(ctx, row.id.into(), requested_fraction)?;
        remaining_micros = remaining_micros
            .checked_sub_fraction(consumed)
            .expect("consumption cannot exceed the bounded request");
        if remaining_micros == MeasuredItemAmountMicros::ZERO {
            return Ok(());
        }
    }
    Err(SmithingConsumptionError::Insufficient {
        item: item_id.clone(),
        missing: remaining_micros,
    })
}

#[reducer]
pub fn forge_weapon(
    ctx: &ReducerContext,
    character_id: u64,
    settlement_id: String,
    recipe: Vec<u8>,
) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, (character_id).into())
        .map_err(|error: crate::strategic::StrategicCharacterAuthorityError| error.to_string())?;
    if recipe.len() > crate::weapon_instance::MAX_WEAPON_RECIPE_BYTES {
        return Err("Weapon recipe exceeds the smithing limit".into());
    }
    let design = decode(&recipe).map_err(|error| format!("Invalid weapon recipe: {error}"))?;
    if !adventuresim_weapon_model::MELEE_CATALOG_IDS.contains(&design.catalog_id.as_str()) {
        return Err("Weapon recipe uses an unsupported chassis".into());
    }
    let quote = adventuresim_core::smithing::quote_weapon(&design).map_err(
        |error: adventuresim_core::smithing::ForgeQuoteError| -> String { error.to_string() },
    )?;
    let character = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or("Character not found")?;
    if character.current_settlement_id.as_deref() != Some(&settlement_id) {
        return Err("Character must be at this forge".into());
    }
    use adventuresim_world_schema::SettlementService as Service;
    let economy_has_forge =
        crate::strategic::require_settlement_service(ctx, &settlement_id, Service::Weaponsmith)
            .is_ok()
            || crate::strategic::require_settlement_service(
                ctx,
                &settlement_id,
                Service::GeneralBlacksmith,
            )
            .is_ok();
    let organization_has_forge =
        adventuresim_core::organization::organization_service_chapter(&settlement_id, "weapons")
            .is_some();
    if !economy_has_forge && !organization_has_forge {
        return Err("This settlement has no weaponsmith forge".into());
    }
    for (item_id, required) in &quote.requirements {
        if available(ctx, character_id.into(), item_id) < *required {
            return Err(format!(
                "Insufficient {item_id}: {} milliunits required",
                required.get()
            ));
        }
    }
    for (item_id, required) in &quote.requirements {
        consume(ctx, character_id.into(), item_id, *required)
            .map_err(|error: SmithingConsumptionError| -> String { error.to_string() })?;
    }
    if !crate::time::advance_character_wait_time(ctx, (character_id).into(), quote.minutes)? {
        return Err("The smithing session could not be completed".into());
    }
    let inventory_id = add_inventory_item_checked(
        ctx,
        character_id.into(),
        &(&design.catalog_id).into(),
        1.into(),
    )
    .map_err(|error: crate::item::InventoryGrantError| -> String { error.to_string() })?
    .ok_or("Could not create forged weapon")?;
    let object = crate::inventory_container::object_for_row(
        ctx,
        adventuresim_core::physical_object::CarriedInventoryScope::Personal,
        inventory_id,
    )
    .map_err(
        |error: crate::inventory_container::InventoryObjectError| -> String { error.to_string() },
    )?
    .ok_or("Forged weapon has no physical identity")?;
    crate::weapon_instance::replace_design(ctx, object.id, &design).map_err(
        |error: crate::weapon_instance::WeaponInstanceError| -> String { error.to_string() },
    )
}
