// Owns food-lot admission and initial material state.
fn current_minute(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> StrategicMinute {
    ctx.db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map_or(StrategicMinute::ZERO, |t| t.minutes)
}

fn ensure_food_material_object(
    ctx: &ReducerContext,
    scope: CarriedInventoryScope,
    row_id: adventuresim_core::identity::InventoryItemId,
) -> Result<crate::InventoryObject, FoodLotCreationError> {
    let (item_id, location, quantity) = match scope {
        CarriedInventoryScope::Personal => {
            let row = ctx
                .db
                .inventory_item()
                .id()
                .find(row_id.get())
                .ok_or(FoodLotCreationError::MissingRow { scope, row: row_id })?;
            (
                row.item_id,
                InventoryLocation::personal(row.character_id, row.id),
                row.quantity,
            )
        }
        CarriedInventoryScope::Party => {
            let row = ctx
                .db
                .party_inventory_item()
                .id()
                .find(row_id.get())
                .ok_or(FoodLotCreationError::MissingRow { scope, row: row_id })?;
            (
                row.item_id,
                InventoryLocation::party(row.party_id, row.id),
                row.quantity,
            )
        }
    };
    if quantity != 1 {
        return Err(FoodLotCreationError::NonIndividual { scope, row: row_id });
    }
    if let Some(object) = crate::inventory_container::object_for_row(ctx, scope, row_id)? {
        if object.item_id != item_id || object.location != location {
            return Err(FoodLotCreationError::MismatchedObject { scope, row: row_id });
        }
        return Ok(object);
    }
    Ok(ctx.db.inventory_object().insert(crate::InventoryObject {
        id: 0,
        item_id,
        location,
    }))
}

pub(crate) fn create_personal_food_lot(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    inventory_item_id: adventuresim_core::identity::InventoryItemId,
    item_id: &adventuresim_core::item_catalog::ItemDefinitionId,
    quantity: adventuresim_core::inventory_measurement::ItemQuantity,
) -> Result<FoodLot, FoodLotCreationError> {
    let definition = food::definition(item_id)
        .ok_or_else(|| crate::item::MissingInventoryFoodDefinition::for_item(item_id.clone()))?;
    let minute = current_minute(ctx, (character_id).into());
    ensure_food_material_object(ctx, CarriedInventoryScope::Personal, inventory_item_id)?;
    let lot = ctx.db.food_lot().insert(FoodLot {
        id: 0,
        inventory_item_id: Some(inventory_item_id.get()),
        party_inventory_item_id: None,
        material_revision: 1,
        display_name: definition.name.into(),
        preparation: if definition.class == food::FoodClass::Ration {
            FoodPreparation::Preserved
        } else {
            FoodPreparation::Raw
        },
        ingredient_item_ids: vec![item_id.to_string()],
        ingredient_quantities: vec![quantity.get() as f32],
        salty_kg: definition.flavors_per_unit.salty * quantity.get() as f32,
        spicy_kg: definition.flavors_per_unit.spicy * quantity.get() as f32,
        sweet_kg: definition.flavors_per_unit.sweet * quantity.get() as f32,
        sour_kg: definition.flavors_per_unit.sour * quantity.get() as f32,
        savory_kg: definition.flavors_per_unit.savory * quantity.get() as f32,
        quality: definition.default_quality.clamp(1, 5),
        mass_kg: definition.mass_kg_per_unit * quantity.get() as f32,
        nutrition_kcal: definition.kcal_per_unit * quantity.get() as f32,
        total_value: definition.value_per_unit * quantity.get() as f32,
        created_at_minute: minute,
    });
    ctx.db.food_contamination().insert(FoodContamination {
        food_lot_id: lot.id,
        concentration_anchor: food::deterministic_initial_contamination(
            fabelgeist_determinism::StreamId::new("food.lot-consumption")
                .rng(ctx.random(), &[lot.id, u64::from(character_id)])
                .next_u64(),
        ),
        growth_per_hour: definition.growth_per_hour,
        anchor_minute: minute,
    });
    Ok(lot)
}

pub fn create_party_food_lot(
    ctx: &ReducerContext,
    inventory_item_id: u64,
    item_id: &adventuresim_core::item_catalog::ItemDefinitionId,
    quantity: u32,
    minute: StrategicMinute,
) -> Option<FoodLot> {
    let definition = food::definition(item_id)?;
    ensure_food_material_object(ctx, CarriedInventoryScope::Party, inventory_item_id.into())
        .ok()?;
    let lot = ctx.db.food_lot().insert(FoodLot {
        id: 0,
        inventory_item_id: None,
        party_inventory_item_id: Some(inventory_item_id),
        material_revision: 1,
        display_name: definition.name.into(),
        preparation: if definition.class == food::FoodClass::Ration {
            FoodPreparation::Preserved
        } else {
            FoodPreparation::Raw
        },
        ingredient_item_ids: vec![item_id.to_string()],
        ingredient_quantities: vec![quantity as f32],
        salty_kg: definition.flavors_per_unit.salty * quantity as f32,
        spicy_kg: definition.flavors_per_unit.spicy * quantity as f32,
        sweet_kg: definition.flavors_per_unit.sweet * quantity as f32,
        sour_kg: definition.flavors_per_unit.sour * quantity as f32,
        savory_kg: definition.flavors_per_unit.savory * quantity as f32,
        quality: definition.default_quality.clamp(1, 5),
        mass_kg: definition.mass_kg_per_unit * quantity as f32,
        nutrition_kcal: definition.kcal_per_unit * quantity as f32,
        total_value: definition.value_per_unit * quantity as f32,
        created_at_minute: minute,
    });
    ctx.db.food_contamination().insert(FoodContamination {
        food_lot_id: lot.id,
        concentration_anchor: food::deterministic_initial_contamination(
            fabelgeist_determinism::StreamId::new("food.lot-update")
                .rng(ctx.random(), &[lot.id])
                .next_u64(),
        ),
        growth_per_hour: definition.growth_per_hour,
        anchor_minute: minute,
    });
    Some(lot)
}
