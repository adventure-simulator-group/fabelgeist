// Owns food-lot splitting and carried binding transfers.
pub fn split_lot(
    ctx: &ReducerContext,
    source_inventory_id: u64,
    destination_inventory_id: u64,
    taken: u32,
    original: u32,
) -> Result<(), FoodLotMutationError> {
    if taken == 0 || original == 0 || taken > original {
        return Err(FoodLotMutationError::InvalidSplit);
    }
    let mut source = lot_for_inventory(ctx, source_inventory_id)?;
    if taken == original {
        source.inventory_item_id = Some(destination_inventory_id);
        ctx.db.food_lot().id().update(source);
        return Ok(());
    }
    let ratio = taken as f32 / original as f32;
    let mut child = source.clone();
    child.id = 0;
    child.inventory_item_id = Some(destination_inventory_id);
    retain_lot_fraction(&mut child, ratio)?;
    let (source_ingredients, child_ingredients) =
        split_ingredient_quantities(&source.ingredient_quantities, taken, original);
    child.ingredient_quantities = child_ingredients;
    retain_lot_fraction(&mut source, 1.0 - ratio)?;
    source.ingredient_quantities = source_ingredients;
    let contamination = ctx
        .db
        .food_contamination()
        .food_lot_id()
        .find(source.id)
        .ok_or(FoodLotMutationError::MissingContamination(source.id.into()))?;
    ensure_food_material_object(
        ctx,
        CarriedInventoryScope::Personal,
        destination_inventory_id.into(),
    )?;
    let child = ctx.db.food_lot().insert(child);
    split_food_contamination_provenance(ctx, source.id, child.id, ratio);
    crate::herbalism::split_food_medicine(ctx, source.id, child.id, ratio)?;
    ctx.db.food_contamination().insert(FoodContamination {
        food_lot_id: child.id,
        ..contamination
    });
    ctx.db.food_lot().id().update(source);
    Ok(())
}

pub fn move_or_split_to_party(
    ctx: &ReducerContext,
    source_inventory_id: u64,
    destination_party_id: u64,
    taken: u32,
    original: u32,
) -> Result<(), FoodLotMutationError> {
    let mut source = lot_for_inventory(ctx, source_inventory_id)?;
    if taken == original {
        source.inventory_item_id = None;
        source.party_inventory_item_id = Some(destination_party_id);
        ctx.db.food_lot().id().update(source);
        return Ok(());
    }
    let ratio = taken as f32 / original as f32;
    let mut child = source.clone();
    child.id = 0;
    child.inventory_item_id = None;
    child.party_inventory_item_id = Some(destination_party_id);
    retain_lot_fraction(&mut child, ratio)?;
    let (source_ingredients, child_ingredients) =
        split_ingredient_quantities(&source.ingredient_quantities, taken, original);
    child.ingredient_quantities = child_ingredients;
    retain_lot_fraction(&mut source, 1.0 - ratio)?;
    source.ingredient_quantities = source_ingredients;
    let hidden = ctx
        .db
        .food_contamination()
        .food_lot_id()
        .find(source.id)
        .ok_or(FoodLotMutationError::MissingContamination(source.id.into()))?;
    ensure_food_material_object(
        ctx,
        CarriedInventoryScope::Party,
        destination_party_id.into(),
    )?;
    let child = ctx.db.food_lot().insert(child);
    split_food_contamination_provenance(ctx, source.id, child.id, ratio);
    crate::herbalism::split_food_medicine(ctx, source.id, child.id, ratio)?;
    ctx.db.food_contamination().insert(FoodContamination {
        food_lot_id: child.id,
        ..hidden
    });
    ctx.db.food_lot().id().update(source);
    Ok(())
}

pub fn move_or_split_to_personal(
    ctx: &ReducerContext,
    source_party_id: u64,
    destination_inventory_id: u64,
    taken: u32,
    original: u32,
) -> Result<(), FoodLotMutationError> {
    let mut source = ctx
        .db
        .food_lot()
        .iter()
        .find(|lot| lot.party_inventory_item_id == Some(source_party_id))
        .ok_or(FoodLotMutationError::MissingLot {
            scope: CarriedInventoryScope::Party,
            row: source_party_id.into(),
        })?;
    if taken == original {
        source.party_inventory_item_id = None;
        source.inventory_item_id = Some(destination_inventory_id);
        ctx.db.food_lot().id().update(source);
        return Ok(());
    }
    let ratio = taken as f32 / original as f32;
    let mut child = source.clone();
    child.id = 0;
    child.party_inventory_item_id = None;
    child.inventory_item_id = Some(destination_inventory_id);
    retain_lot_fraction(&mut child, ratio)?;
    let (source_ingredients, child_ingredients) =
        split_ingredient_quantities(&source.ingredient_quantities, taken, original);
    child.ingredient_quantities = child_ingredients;
    retain_lot_fraction(&mut source, 1.0 - ratio)?;
    source.ingredient_quantities = source_ingredients;
    let hidden = ctx
        .db
        .food_contamination()
        .food_lot_id()
        .find(source.id)
        .ok_or(FoodLotMutationError::MissingContamination(source.id.into()))?;
    ensure_food_material_object(
        ctx,
        CarriedInventoryScope::Personal,
        destination_inventory_id.into(),
    )?;
    let child = ctx.db.food_lot().insert(child);
    split_food_contamination_provenance(ctx, source.id, child.id, ratio);
    crate::herbalism::split_food_medicine(ctx, source.id, child.id, ratio)?;
    ctx.db.food_contamination().insert(FoodContamination {
        food_lot_id: child.id,
        ..hidden
    });
    ctx.db.food_lot().id().update(source);
    Ok(())
}
