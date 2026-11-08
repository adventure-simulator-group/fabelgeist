pub(crate) fn party_currency_total(ctx: &ReducerContext, party_id: &str) -> u64 {
    ctx.db
        .party_inventory_item()
        .party_id()
        .filter(party_id)
        .filter(|stack| crate::item::is_currency(ctx, &stack.item_id))
        .map(|stack| u64::from(stack.quantity))
        .sum()
}

pub(crate) fn consume_party_currency(
    ctx: &ReducerContext,
    party_id: &str,
    amount: u64,
) -> Result<(), String> {
    let mut stacks: Vec<_> = ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(party_id)
        .filter(|stack| crate::item::is_currency(ctx, &stack.item_id))
        .collect();
    if stacks
        .iter()
        .map(|stack| u64::from(stack.quantity))
        .sum::<u64>()
        < amount
    {
        return Err("Not enough party coin to cover this payment".into());
    }
    stacks.sort_by(|a, b| (&a.item_id, a.id).cmp(&(&b.item_id, b.id)));
    validate_party_currency_debits(ctx, &stacks, amount)?;
    let mut remaining = amount;
    for mut stack in stacks {
        if remaining == 0 {
            break;
        }
        let taken = remaining.min(u64::from(stack.quantity)) as u32;
        if taken == 0 {
            continue;
        }
        stack.quantity -= taken;
        remaining -= u64::from(taken);
        if stack.quantity == 0 {
            ctx.db.party_inventory_item().id().delete(stack.id);
        } else {
            ctx.db.party_inventory_item().id().update(stack);
        }
        if remaining == 0 {
            break;
        }
    }
    Ok(())
}

pub(crate) fn credit_party_currency(
    ctx: &ReducerContext,
    party_id: &str,
    settlement_id: &str,
    amount: u32,
) -> Result<(), String> {
    let currency_id = crate::item::currency_id_for_settlement(ctx, settlement_id)?;
    add_to_party_inventory(ctx, party_id, &currency_id, amount)
}

fn transfer_personal_currency_to_party(
    ctx: &ReducerContext,
    character_id: u64,
    party_id: &str,
    amount: u64,
) -> Result<(), String> {
    let mut stacks: Vec<_> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter(|stack| crate::item::is_currency(ctx, &stack.item_id))
        .collect();
    if stacks.iter().map(|s| u64::from(s.quantity)).sum::<u64>() < amount {
        return Err("Not enough personal coin".into());
    }
    stacks.sort_by(|a, b| (&a.item_id, a.id).cmp(&(&b.item_id, b.id)));
    validate_personal_currency_debits(ctx, &stacks, amount)?;
    let mut remaining = amount;
    for mut stack in stacks {
        if remaining == 0 {
            break;
        }
        let taken = remaining.min(u64::from(stack.quantity)) as u32;
        if taken == 0 {
            continue;
        }
        add_to_party_inventory(ctx, party_id, &stack.item_id, taken)?;
        stack.quantity -= taken;
        remaining -= u64::from(taken);
        if stack.quantity == 0 {
            ctx.db.inventory_item().id().delete(stack.id);
        } else {
            ctx.db.inventory_item().id().update(stack);
        }
        if remaining == 0 {
            break;
        }
    }
    Ok(())
}

fn transfer_party_currency_to_personal(
    ctx: &ReducerContext,
    party_id: &str,
    character_id: u64,
    amount: u64,
) -> crate::StrategicOperationResult<()> {
    let mut stacks: Vec<_> = ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(party_id)
        .filter(|stack| crate::item::is_currency(ctx, &stack.item_id))
        .collect();
    if stacks.iter().map(|s| u64::from(s.quantity)).sum::<u64>() < amount {
        return Err("The party has insufficient coin".into());
    }
    stacks.sort_by(|a, b| (&a.item_id, a.id).cmp(&(&b.item_id, b.id)));
    validate_party_currency_debits(ctx, &stacks, amount)?;
    let mut remaining = amount;
    for mut stack in stacks {
        if remaining == 0 {
            break;
        }
        let taken = remaining.min(u64::from(stack.quantity)) as u32;
        if taken == 0 {
            continue;
        }
        crate::add_inventory_item(ctx, character_id, &stack.item_id, taken)?;
        stack.quantity -= taken;
        remaining -= u64::from(taken);
        if stack.quantity == 0 {
            ctx.db.party_inventory_item().id().delete(stack.id);
        } else {
            ctx.db.party_inventory_item().id().update(stack);
        }
        if remaining == 0 {
            break;
        }
    }
    Ok(())
}

// Preflight every stack that will be debited before mutating any currency.
fn validate_party_currency_debits(
    ctx: &ReducerContext,
    stacks: &[PartyInventoryItem],
    mut remaining: u64,
) -> Result<(), String> {
    for stack in stacks {
        if remaining == 0 {
            break;
        }
        let taken = remaining.min(u64::from(stack.quantity));
        if taken == 0 {
            continue;
        }
        crate::inventory_container::require_fungible_inventory_row(
            ctx,
            CarriedInventoryRowId::Party(PartyInventoryRowId::from_raw(stack.id)),
            &stack.item_id,
        )?;
        remaining -= taken;
    }
    Ok(())
}

fn validate_personal_currency_debits(
    ctx: &ReducerContext,
    stacks: &[InventoryItem],
    mut remaining: u64,
) -> Result<(), String> {
    for stack in stacks {
        if remaining == 0 {
            break;
        }
        let taken = remaining.min(u64::from(stack.quantity));
        if taken == 0 {
            continue;
        }
        crate::inventory_container::require_fungible_inventory_row(
            ctx,
            CarriedInventoryRowId::Personal(PersonalInventoryRowId::from_raw(stack.id)),
            &stack.item_id,
        )?;
        remaining -= taken;
    }
    Ok(())
}
