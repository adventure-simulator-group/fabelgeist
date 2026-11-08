// Resolve an ingredient selection against its carried inventory.
struct SelectedCookingIngredient {
    item_id: String,
    available: ConsumableFractionMicros,
    lot: FoodLot,
}

fn resolve_cooking_ingredient(
    ctx: &ReducerContext,
    actor: &crate::Character,
    row_id: CarriedInventoryRowId,
) -> Result<SelectedCookingIngredient, String> {
    match row_id {
        CarriedInventoryRowId::Personal(id) => {
            let row = ctx
                .db
                .inventory_item()
                .id()
                .find(id.get())
                .ok_or("Ingredient inventory row not found")?;
            if row.character_id != actor.id || crate::character::wearable_is_equipped(ctx, id.get())
            {
                return Err("Ingredient is equipped or not in this inventory".into());
            }
            Ok(SelectedCookingIngredient {
                item_id: row.item_id,
                available: crate::inventory_amount::personal_fraction(ctx, id.get())
                    .ok_or("Ingredient amount state is missing")?,
                lot: personal_lot(ctx, id).ok_or("Food lot metadata not found")?,
            })
        }
        CarriedInventoryRowId::Party(id) => {
            let party_id = actor
                .party_id
                .as_deref()
                .ok_or("Character has no party inventory")?;
            let row = ctx
                .db
                .party_inventory_item()
                .id()
                .find(id.get())
                .ok_or("Ingredient inventory row not found")?;
            if row.party_id != party_id {
                return Err("Ingredient is not in this party inventory".into());
            }
            Ok(SelectedCookingIngredient {
                item_id: row.item_id,
                available: crate::inventory_amount::party_fraction(ctx, id.get())
                    .ok_or("Ingredient amount state is missing")?,
                lot: party_lot(ctx, id).ok_or("Food lot metadata not found")?,
            })
        }
    }
}
