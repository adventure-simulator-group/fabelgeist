// Resolves a food lot to its carried physical-object backing.
fn ensure_food_material_object(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
) -> Result<crate::InventoryObject, String> {
    let (item_id, location, quantity) = match row_id {
        CarriedInventoryRowId::Personal(row_id) => {
            let row = ctx
                .db
                .inventory_item()
                .id()
                .find(row_id.get())
                .ok_or("Food inventory row is missing")?;
            (
                row.item_id,
                InventoryLocation::personal(row.character_id, row_id),
                row.quantity,
            )
        }
        CarriedInventoryRowId::Party(row_id) => {
            let row = ctx
                .db
                .party_inventory_item()
                .id()
                .find(row_id.get())
                .ok_or("Party food inventory row is missing")?;
            (
                row.item_id,
                InventoryLocation::party(row.party_id, row_id),
                row.quantity,
            )
        }
    };
    if quantity != 1 {
        return Err("Every food lot requires a quantity-one stable inventory object".into());
    }
    if let Some(object) = crate::inventory_container::object_for_row(ctx, row_id)? {
        if object.item_id != item_id || object.location != location {
            return Err("Food inventory row has a mismatched stable object identity".into());
        }
        return Ok(object);
    }
    Ok(ctx.db.inventory_object().insert(crate::InventoryObject {
        id: 0,
        item_id,
        location,
    }))
}

fn insert_cooked_meal_backing(
    ctx: &ReducerContext,
    destination: &OperationalCustody,
    container_object_id: Option<u64>,
) -> Result<CarriedInventoryRowId, String> {
    let backing = match destination {
        OperationalCustody::Character(character) => {
            let row = ctx.db.inventory_item().insert(crate::InventoryItem {
                id: 0,
                character_id: character.get(),
                item_id: "cooked_meal".into(),
                quantity: 1,
            });
            crate::inventory_amount::initialize_personal(ctx, row.id);
            CarriedInventoryRowId::Personal(PersonalInventoryRowId::from_raw(row.id))
        }
        OperationalCustody::Party(party) => {
            let row = ctx.db.party_inventory_item().insert(PartyInventoryItem {
                id: 0,
                party_id: party.as_str().into(),
                item_id: "cooked_meal".into(),
                quantity: 1,
            });
            crate::inventory_amount::initialize_party(ctx, row.id);
            CarriedInventoryRowId::Party(PartyInventoryRowId::from_raw(row.id))
        }
        _ => return Err("Invalid retrieval inventory".into()),
    };
    if let Some(parent_object_id) = container_object_id {
        let meal = ctx.db.inventory_object().insert(crate::InventoryObject {
            id: 0,
            item_id: "cooked_meal".into(),
            location: crate::inventory_container::carried_location_for_row(destination, backing)?,
        });
        ctx.db
            .inventory_containment()
            .insert(crate::InventoryContainment {
                child_object_id: meal.id,
                parent_object_id,
            });
    }
    ensure_food_material_object(ctx, backing)?;
    Ok(backing)
}
