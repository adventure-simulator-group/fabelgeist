// Owns food-lot binding, lookup, revision, and quantity removal.
pub fn delete_personal_food_lot(ctx: &ReducerContext, inventory_item_id: u64) {
    for lot in ctx
        .db
        .food_lot()
        .iter()
        .filter(|lot| lot.inventory_item_id == Some(inventory_item_id))
        .collect::<Vec<_>>()
    {
        crate::herbalism::delete_food_medicine(ctx, lot.id);
        ctx.db
            .food_contamination_provenance()
            .food_lot_id()
            .delete(lot.id);
        ctx.db.food_contamination().food_lot_id().delete(lot.id);
        ctx.db.food_lot().id().delete(lot.id);
    }
}

pub fn delete_party_food_lot(ctx: &ReducerContext, inventory_item_id: u64) {
    for lot in ctx
        .db
        .food_lot()
        .iter()
        .filter(|lot| lot.party_inventory_item_id == Some(inventory_item_id))
        .collect::<Vec<_>>()
    {
        crate::herbalism::delete_food_medicine(ctx, lot.id);
        ctx.db
            .food_contamination_provenance()
            .food_lot_id()
            .delete(lot.id);
        ctx.db.food_contamination().food_lot_id().delete(lot.id);
        ctx.db.food_lot().id().delete(lot.id);
    }
}

pub fn remove_party_lot_quantity(
    ctx: &ReducerContext,
    inventory_item_id: u64,
    removed: u32,
    original: u32,
) -> Result<(), FoodLotMutationError> {
    if removed == original {
        delete_party_food_lot(ctx, inventory_item_id);
        return Ok(());
    }
    let mut lot = ctx
        .db
        .food_lot()
        .iter()
        .find(|lot| lot.party_inventory_item_id == Some(inventory_item_id))
        .ok_or(FoodLotMutationError::MissingLot {
            scope: CarriedInventoryScope::Party,
            row: inventory_item_id.into(),
        })?;
    let keep = 1.0 - removed as f32 / original as f32;
    retain_lot_fraction(&mut lot, keep)?;
    ctx.db.food_lot().id().update(lot);
    Ok(())
}

fn split_ingredient_quantities(
    quantities: &[f32],
    taken: u32,
    original: u32,
) -> (Vec<f32>, Vec<f32>) {
    let ratio = taken as f32 / original as f32;
    let child = quantities
        .iter()
        .map(|quantity| food::retained_component(*quantity, ratio))
        .collect::<Vec<_>>();
    let source = quantities
        .iter()
        .zip(&child)
        .map(|(quantity, child_quantity)| (quantity - child_quantity).max(0.0))
        .collect();
    (source, child)
}

fn retain_lot_fraction(lot: &mut FoodLot, retained: f32) -> Result<(), FoodLotMutationError> {
    lot.material_revision = lot
        .material_revision
        .checked_add(1)
        .ok_or(FoodLotMutationError::RevisionExhausted(lot.id.into()))?;
    lot.mass_kg = food::retained_component(lot.mass_kg, retained);
    lot.nutrition_kcal = food::retained_component(lot.nutrition_kcal, retained);
    lot.total_value = food::retained_component(lot.total_value, retained);
    lot.salty_kg = food::retained_component(lot.salty_kg, retained);
    lot.spicy_kg = food::retained_component(lot.spicy_kg, retained);
    lot.sweet_kg = food::retained_component(lot.sweet_kg, retained);
    lot.sour_kg = food::retained_component(lot.sour_kg, retained);
    lot.savory_kg = food::retained_component(lot.savory_kg, retained);
    for quantity in &mut lot.ingredient_quantities {
        *quantity = food::retained_component(*quantity, retained);
    }
    Ok(())
}

pub fn personal_lot(ctx: &ReducerContext, inventory_item_id: u64) -> Option<FoodLot> {
    ctx.db
        .food_lot()
        .iter()
        .find(|lot| lot.inventory_item_id == Some(inventory_item_id))
}

pub fn party_lot(ctx: &ReducerContext, inventory_item_id: u64) -> Option<FoodLot> {
    ctx.db
        .food_lot()
        .iter()
        .find(|lot| lot.party_inventory_item_id == Some(inventory_item_id))
}

fn lot_for_inventory(
    ctx: &ReducerContext,
    inventory_item_id: u64,
) -> Result<FoodLot, FoodLotMutationError> {
    personal_lot(ctx, inventory_item_id).ok_or(FoodLotMutationError::MissingLot {
        scope: CarriedInventoryScope::Personal,
        row: inventory_item_id.into(),
    })
}

fn contamination(
    ctx: &ReducerContext,
    lot: &FoodLot,
    minute: StrategicMinute,
) -> Result<(FoodContamination, f32), FoodLotMutationError> {
    let row = ctx
        .db
        .food_contamination()
        .food_lot_id()
        .find(lot.id)
        .ok_or(FoodLotMutationError::MissingContamination(lot.id.into()))?;
    let current = food::contamination_at(
        row.concentration_anchor,
        row.growth_per_hour,
        minute.elapsed_since(row.anchor_minute),
    );
    Ok((row, current))
}

pub fn remove_lot_quantity(
    ctx: &ReducerContext,
    inventory_item_id: u64,
    removed: u32,
    original: u32,
) -> Result<(), FoodLotMutationError> {
    if removed == 0 || original == 0 || removed > original {
        return Err(FoodLotMutationError::InvalidQuantityChange);
    }
    if removed == original {
        delete_personal_food_lot(ctx, inventory_item_id);
        return Ok(());
    }
    let mut lot = lot_for_inventory(ctx, inventory_item_id)?;
    let keep = 1.0 - removed as f32 / original as f32;
    retain_lot_fraction(&mut lot, keep)?;
    ctx.db.food_lot().id().update(lot);
    Ok(())
}
