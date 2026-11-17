//! Guarded damage and readiness fixtures for real repair-custody acceptance.

use super::*;
use crate::strategic::settlement;
use adventuresim_world_schema::SettlementService;

#[reducer]
pub fn authority_test_repair_setup(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
    inventory_item_id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::inventory_effects::require_non_live_actor(ctx, character_id)?;
    let character = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or("Missing repair fixture actor")?;
    let settlement_id = character
        .current_settlement_id
        .ok_or("Repair fixture actor must be in a settlement")?;
    let mut settlement = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id)
        .ok_or("Missing repair fixture settlement")?;
    if !settlement.economy.has_service(SettlementService::Tailor) {
        settlement.economy.services.push(SettlementService::Tailor);
        ctx.db.settlement().id().update(settlement);
    }
    let inventory = ctx
        .db
        .inventory_item()
        .id()
        .find(inventory_item_id)
        .filter(|row| row.character_id == character_id && row.quantity == 1)
        .ok_or("Repair fixture requires one owned equipment instance")?;
    initialize_item_condition(ctx, &inventory);
    let mut condition = ctx
        .db
        .item_condition()
        .inventory_item_id()
        .find(inventory_item_id)
        .ok_or("Repair fixture requires repairable equipment")?;
    condition.set_bins(DamageBins([0.25, 0.0, 0.0, 0.0, 0.0]));
    ctx.db
        .item_condition()
        .inventory_item_id()
        .update(condition);
    crate::capability::refresh_character_capability(ctx, character_id)?;
    crate::condition::refresh_character_strategic_condition(ctx, character_id)?;
    Ok(())
}

#[reducer]
pub fn authority_test_repair_ready(
    ctx: &ReducerContext,
    bootstrap_token: String,
    order_id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut order = ctx
        .db
        .repair_order()
        .id()
        .find(order_id)
        .ok_or("Missing repair fixture order")?;
    crate::inventory_effects::require_non_live_actor(ctx, order.owner_character_id)?;
    order.ready_at_minutes = ctx
        .db
        .character_time()
        .character_id()
        .find(order.owner_character_id)
        .ok_or("Missing repair fixture clock")?
        .minutes;
    ctx.db.repair_order().id().update(order);
    Ok(())
}
