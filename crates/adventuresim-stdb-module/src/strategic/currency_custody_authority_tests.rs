//! Guarded integration coverage for party currency backing and debit atomicity.
use super::*;

#[spacetimedb::reducer]
pub fn authority_test_party_currency_custody(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> crate::StrategicOperationResult<()> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let party_id = "authority:currency-custody";
    let character_id = 739005;
    crate::character::create_named_character_with_id(
        ctx,
        character_id,
        "Party Currency Fixture".into(),
    )?;
    let currency = "rhenish_gulden";
    let first = ctx.db.party_inventory_item().insert(PartyInventoryItem {
        id: 0,
        party_id: party_id.into(),
        item_id: currency.into(),
        quantity: 1,
    });
    let second = ctx.db.party_inventory_item().insert(PartyInventoryItem {
        id: 0,
        party_id: party_id.into(),
        item_id: currency.into(),
        quantity: 1,
    });
    ctx.db.party_item_condition().insert(PartyItemCondition {
        party_inventory_item_id: second.id,
        tier_1: 0.0,
        tier_2: 0.0,
        tier_3: 0.0,
        tier_4: 0.0,
        tier_5: 0.0,
    });
    let personal_before = crate::item::personal_currency_total(ctx, character_id);
    if consume_party_currency(ctx, party_id, 2).is_ok()
        || transfer_party_currency_to_personal(ctx, party_id, character_id, 2).is_ok()
        || party_currency_total(ctx, party_id) != 2
        || crate::item::personal_currency_total(ctx, character_id) != personal_before
    {
        return Err("Corrupt party currency was accepted or partially debited".into());
    }
    ctx.db
        .party_item_condition()
        .party_inventory_item_id()
        .delete(second.id);
    consume_party_currency(ctx, party_id, 1)?;
    if party_currency_total(ctx, party_id) != 1 {
        return Err("Party currency consumption did not debit one coin".into());
    }
    transfer_party_currency_to_personal(ctx, party_id, character_id, 1)?;
    if party_currency_total(ctx, party_id) != 0
        || crate::item::personal_currency_total(ctx, character_id) != personal_before + 1
    {
        return Err("Party currency transfer did not conserve one coin".into());
    }
    if ctx.db.party_inventory_item().id().find(first.id).is_some()
        || ctx.db.party_inventory_item().id().find(second.id).is_some()
    {
        return Err("Consumed party currency rows remain".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_personal_currency_backing(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> crate::StrategicOperationResult<()> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let character_id = 739004;
    let settlement_id = "authority-currency-credit-fixture";
    crate::character::create_named_character_with_id(ctx, character_id, "Currency Fixture".into())?;
    let currency = crate::item::currency_id_for_settlement(ctx, settlement_id)?;
    crate::item::credit_personal_currency(ctx, character_id, settlement_id, 2)?;
    let selected = ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, &currency))
        .next()
        .ok_or("Currency fixture missing stack")?;
    let before: Vec<_> = ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, &currency))
        .map(|row| (row.id, row.quantity))
        .collect();
    ctx.db
        .item_condition()
        .insert(crate::repair::ItemCondition {
            inventory_item_id: selected.id,
            tier_1: 0.0,
            tier_2: 0.0,
            tier_3: 0.0,
            tier_4: 0.0,
            tier_5: 0.0,
        });
    if crate::item::change_inventory_item(ctx, character_id, &currency, 1).is_ok()
        || crate::item::change_inventory_item(ctx, character_id, &currency, -1).is_ok()
        || crate::item::credit_personal_currency(ctx, character_id, settlement_id, 1).is_ok()
    {
        return Err("Corrupt personal currency was accepted".into());
    }
    let after: Vec<_> = ctx
        .db
        .inventory_item()
        .character_and_item_id()
        .filter((character_id, &currency))
        .map(|row| (row.id, row.quantity))
        .collect();
    if before != after {
        return Err("Rejected personal currency operations changed the wallet".into());
    }
    ctx.db
        .item_condition()
        .inventory_item_id()
        .delete(selected.id);
    crate::item::change_inventory_item(ctx, character_id, &currency, 1)?;
    crate::item::change_inventory_item(ctx, character_id, &currency, -1)?;
    crate::item::credit_personal_currency(ctx, character_id, settlement_id, 1)?;
    let ending = ctx
        .db
        .inventory_item()
        .id()
        .find(selected.id)
        .ok_or("Currency stack disappeared")?;
    if ending.quantity != selected.quantity + 1 {
        return Err("Valid personal currency operations changed the wrong amount".into());
    }
    Ok(())
}
