// Select carried tools for ingredient preparation.
fn cutting_weapon_binding(
    scope: CarriedInventoryScope,
    row_id: u64,
    item_id: &str,
    precision: f32,
    edge_sensitivity: f32,
    damage: DamageBins,
) -> String {
    use sha2::Digest as _;

    let mut hash = sha2::Sha256::new();
    for value in [
        b"adventuresim.ingredient-preparation.cutting-weapon".as_slice(),
        1u16.to_le_bytes().as_slice(),
        scope.as_str().as_bytes(),
        row_id.to_le_bytes().as_slice(),
        item_id.as_bytes(),
        precision.to_bits().to_le_bytes().as_slice(),
        edge_sensitivity.to_bits().to_le_bytes().as_slice(),
    ] {
        hash.update((value.len() as u64).to_le_bytes());
        hash.update(value);
    }
    for damage_bits in damage.0.map(f32::to_bits) {
        let value = damage_bits.to_le_bytes();
        hash.update((value.len() as u64).to_le_bytes());
        hash.update(value);
    }
    format!("cutting-weapon:v1:{:x}", hash.finalize())
}

fn carried_item_rows(
    ctx: &ReducerContext,
    character_id: u64,
) -> Vec<(CarriedInventoryScope, u64, String)> {
    let Some(actor) = ctx.db.character().id().find(character_id) else {
        return Vec::new();
    };
    let mut rows = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter(|row| {
            crate::inventory_container::object_for_row(
                ctx,
                CarriedInventoryRowId::Personal(PersonalInventoryRowId::from_raw(row.id)),
            )
            .ok()
            .flatten()
            .is_some_and(|object| {
                crate::object_custody::require_actor_carried_object(ctx, &actor, &object).is_ok()
                    && !crate::inventory_container::ancestry_reaches_fireplace(ctx, object.id)
            })
        })
        .map(|row| (CarriedInventoryScope::Personal, row.id, row.item_id))
        .collect::<Vec<_>>();
    if let Some(party_id) = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .and_then(|row| row.party_id)
    {
        rows.extend(
            ctx.db
                .party_inventory_item()
                .party_id()
                .filter(&party_id)
                .filter(|row| {
                    crate::inventory_container::object_for_row(
                        ctx,
                        CarriedInventoryRowId::Party(PartyInventoryRowId::from_raw(row.id)),
                    )
                    .ok()
                    .flatten()
                    .is_some_and(|object| {
                        crate::object_custody::require_actor_carried_object(ctx, &actor, &object)
                            .is_ok()
                            && !crate::inventory_container::ancestry_reaches_fireplace(
                                ctx, object.id,
                            )
                    })
                })
                .map(|row| (CarriedInventoryScope::Party, row.id, row.item_id)),
        );
    }
    rows
}

fn qualifying_cutting_weapon_binding(ctx: &ReducerContext, character_id: u64) -> Option<String> {
    carried_item_rows(ctx, character_id)
        .into_iter()
        .filter_map(|(scope, row_id, item_id)| {
            let item = ctx.db.item().id().find(item_id)?;
            if item.precision < 0.5 {
                return None;
            }
            let damage = match scope {
                CarriedInventoryScope::Personal => ctx
                    .db
                    .item_condition()
                    .inventory_item_id()
                    .find(row_id)
                    .map(|c| c.bins()),
                CarriedInventoryScope::Party => ctx
                    .db
                    .party_item_condition()
                    .party_inventory_item_id()
                    .find(row_id)
                    .map(|c| {
                        DamageBins([c.tier_1, c.tier_2, c.tier_3, c.tier_4, c.tier_5]).normalized()
                    }),
            }
            .unwrap_or_default();
            (effective_weapon_stat(item.precision, damage, item.edge_sensitivity) >= 0.5).then(
                || {
                    cutting_weapon_binding(
                        scope,
                        row_id,
                        &item.id,
                        item.precision,
                        item.edge_sensitivity,
                        damage,
                    )
                },
            )
        })
        .min()
}

fn grinding_tool_binding(ctx: &ReducerContext, character_id: u64) -> String {
    carried_item_rows(ctx, character_id)
        .into_iter()
        .filter(|(_, _, item_id)| item_id == "mortar_and_pestle")
        .map(|(scope, row_id, item_id)| format!("{}|{row_id}|{item_id}", scope.as_str()))
        .min()
        .unwrap_or_else(|| "hands".into())
}
