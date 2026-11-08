//! Guarded integration coverage for tavern currency failures.
use super::*;
use crate::repair::{ItemCondition, item_condition};

#[spacetimedb::reducer]
pub fn authority_test_tavern_currency_failure(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> crate::StrategicOperationResult<()> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let character_id = 739003;
    crate::character::create_named_character_with_id(ctx, character_id, "Tavern Fixture".into())?;
    let drink = ctx
        .db
        .item()
        .id()
        .find(TAVERN_DRINK_ITEM_ID.to_owned())
        .ok_or("Tavern fixture has no drink")?;
    let price = drink
        .base_value
        .ok_or("Tavern fixture drink has no price")?;
    let target = ethanol_ml(properties(&drink));
    if price == 0 || target == 0 {
        return Err("Tavern fixture requires a priced drink".into());
    }
    crate::add_inventory_item(ctx, character_id, "rhenish_gulden", price)?;
    let mut stacks: Vec<_> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter(|stack| crate::item::is_currency(ctx, &stack.item_id))
        .collect();
    stacks.sort_by(|a, b| (&a.item_id, a.id).cmp(&(&b.item_id, b.id)));
    let row = stacks
        .into_iter()
        .find(|stack| stack.quantity > 0)
        .ok_or("Tavern fixture has no currency")?;
    let wallet_before = crate::item::personal_currency_total(ctx, character_id);
    ctx.db.item_condition().insert(ItemCondition {
        inventory_item_id: row.id,
        tier_1: 0.0,
        tier_2: 0.0,
        tier_3: 0.0,
        tier_4: 0.0,
        tier_5: 0.0,
    });
    if tavern_purchase(ctx, character_id, target).is_ok()
        || crate::item::personal_currency_total(ctx, character_id) != wallet_before
    {
        return Err("Tavern accepted corrupt currency or changed the wallet".into());
    }
    ctx.db.item_condition().inventory_item_id().delete(row.id);
    if tavern_purchase(ctx, character_id, target)? != target
        || crate::item::personal_currency_total(ctx, character_id)
            != wallet_before - u64::from(price)
    {
        return Err("Tavern purchase failed to consume its exact price".into());
    }
    Ok(())
}
