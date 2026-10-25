//! Guarded operation, digest, conservation, and terminal-attempt checks.

use super::*;
use crate::condition::character_condition;

fn preparation_actor(ctx: &ReducerContext, id: u64) -> Result<u64, String> {
    crate::character::create_named_character_with_id(ctx, id, "Preparation Fixture".into())?;
    let mut actor = ctx.db.character().id().find(id).ok_or("Actor missing")?;
    actor.current_settlement_id = Some("riverdale".into());
    ctx.db.character().id().update(actor);
    crate::item::add_inventory_item_checked(ctx, id, "utility_knife", 1)?;
    crate::item::add_inventory_item_checked(ctx, id, "apple", 1)?.ok_or("Ingredient missing".into())
}

fn prepare_and_replay(
    ctx: &ReducerContext,
    actor_id: u64,
    row_id: u64,
    action: IngredientPreparationAction,
    interrupted: bool,
) -> Result<(), String> {
    let actor = ctx
        .db
        .character()
        .id()
        .find(actor_id)
        .ok_or("Actor missing")?;
    let lot = personal_lot(ctx, row_id).ok_or("Lot missing")?;
    let object =
        crate::inventory_container::object_for_row(ctx, CarriedInventoryScope::Personal, row_id)?
            .ok_or("Object missing")?;
    let place = preparation_place(ctx, &actor)?.to_string();
    let custody = crate::object_custody::canonical_custody_binding(
        crate::object_custody::require_actor_carried_object(ctx, &actor, &object)?
            .object
            .custody(),
    );
    let request = preparation_request_id(
        actor_id,
        "personal",
        row_id,
        lot.id,
        object.id,
        lot.material_revision,
        action,
        0,
        &place,
        &custody,
    );
    let submit = || {
        prepare_ingredient_lot(
            ctx,
            actor_id,
            "personal".into(),
            row_id,
            lot.id,
            object.id,
            request.clone(),
            lot.material_revision,
            0,
            action,
        )
    };
    submit()?;
    let after = personal_lot(ctx, row_id).ok_or("Lot missing")?;
    let receipt = ctx
        .db
        .ingredient_preparation_receipt()
        .request_id()
        .find(&request)
        .ok_or("Receipt missing")?;
    if receipt.interrupted != interrupted
        || after.mass_kg != lot.mass_kg
        || after.nutrition_kcal != lot.nutrition_kcal
        || after.total_value != lot.total_value
        || after.material_revision != lot.material_revision + u64::from(!interrupted)
        || after.preparation
            != if interrupted {
                lot.preparation
            } else {
                match action {
                    IngredientPreparationAction::Cut => FoodPreparation::Cut,
                    IngredientPreparationAction::Grind => FoodPreparation::Ground,
                }
            }
    {
        return Err("Preparation operation or conservation disagrees".into());
    }
    let minute = current_minute(ctx, actor_id);
    submit()?;
    if current_minute(ctx, actor_id) != minute
        || personal_lot(ctx, row_id)
            .ok_or("Lot missing")?
            .material_revision
            != after.material_revision
    {
        return Err("Preparation replay repeated work".into());
    }
    let conflicting = prepare_ingredient_lot(
        ctx,
        actor_id,
        "personal".into(),
        row_id,
        lot.id,
        object.id,
        request.clone(),
        lot.material_revision,
        1,
        action,
    );
    if conflicting.is_ok() {
        return Err("Preparation accepted conflicting generation".into());
    }
    let key = preparation_attempt_state_key(
        actor_id,
        "personal",
        row_id,
        lot.id,
        object.id,
        lot.material_revision,
        action,
    );
    let state = ctx
        .db
        .ingredient_preparation_attempt_state()
        .key()
        .find(key)
        .ok_or("Cursor missing")?;
    if state.completed == interrupted || state.next_generation != u64::from(interrupted) {
        return Err("Terminal preparation cursor disagrees".into());
    }
    if interrupted
        && preparation_request_id(
            actor_id,
            "personal",
            row_id,
            lot.id,
            object.id,
            lot.material_revision,
            action,
            state.next_generation,
            &place,
            &custody,
        ) == request
    {
        return Err("Clipped preparation did not issue a distinct generation".into());
    }
    Ok(())
}

#[reducer]
pub fn authority_test_preparation_operations(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    for (action, digest) in [
        (
            IngredientPreparationAction::Cut,
            "ff085c9cd2db80b7148e478c427e9a4d8a86b65d024663e5c8aa4c096f5ad478",
        ),
        (
            IngredientPreparationAction::Grind,
            "74bc89b9f01632edcf152a80ec6db97d2f2cb8f8bab32cc13a795ed6038485d9",
        ),
    ] {
        if preparation_request_id(
            1,
            "personal",
            2,
            3,
            4,
            5,
            action,
            0,
            "settlement:test",
            "character:1",
        ) != digest
        {
            return Err("Versioned preparation digest changed".into());
        }
    }
    let actor = 732051;
    let row = preparation_actor(ctx, actor)?;
    prepare_and_replay(ctx, actor, row, IngredientPreparationAction::Cut, false)?;
    prepare_and_replay(ctx, actor, row, IngredientPreparationAction::Grind, false)?;
    let interrupted = 732052;
    let row = preparation_actor(ctx, interrupted)?;
    let mut condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(interrupted)
        .ok_or("Condition missing")?;
    condition.current_blood_ml = condition.maximum_blood_ml * 0.09;
    ctx.db
        .character_condition()
        .character_id()
        .update(condition);
    prepare_and_replay(
        ctx,
        interrupted,
        row,
        IngredientPreparationAction::Grind,
        true,
    )
}
