use super::*;

pub(super) fn seed_damaged_equipment(
    ctx: &ReducerContext,
    character_id: u64,
) -> crate::StrategicOperationResult<()> {
    // Exercise field, settlement, and beyond-smith repair states across both
    // specialist screens. Equipped pieces make the durability column useful,
    // while the two spares also exercise the combined sell/repair row actions.
    let mut damaged_equipment = Vec::new();
    let armor_damage = [
        [0.18, 0.0, 0.0, 0.0, 0.0],
        [0.0, 0.10, 0.10, 0.0, 0.0],
        [0.10, 0.0, 0.08, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.12, 0.08],
        [0.0, 0.0, 0.15, 0.05, 0.0],
        [0.05, 0.10, 0.08, 0.07, 0.0],
        [0.0, 0.14, 0.0, 0.0, 0.0],
    ];
    damaged_equipment.extend(
        equipped_wearable_ids(ctx, character_id)
            .into_iter()
            // Tactical carry provisioning also equips non-durable belts and
            // sheaths. The strategic damage fixture may only damage items
            // that actually own an authoritative condition row.
            .filter(|inventory_item_id| {
                ctx.db
                    .item_condition()
                    .inventory_item_id()
                    .find(*inventory_item_id)
                    .is_some()
            })
            .enumerate()
            .map(|(index, id)| (Some(id), armor_damage[index % armor_damage.len()])),
    );
    for (inventory_item_id, bins) in damaged_equipment {
        if let Some(inventory_item_id) = inventory_item_id {
            set_demo_item_damage(ctx, inventory_item_id, bins)?;
        }
    }

    for (item_id, bins) in [
        ("arming_sword", [0.10, 0.08, 0.12, 0.05, 0.0]),
        ("brigandine", [0.06, 0.10, 0.08, 0.08, 0.04]),
    ] {
        let inventory_item_id = ctx
            .db
            .inventory_item()
            .character_id()
            .filter(character_id)
            .find(|inventory| inventory.item_id == item_id)
            .map(|inventory| inventory.id);
        let inventory_item_id = match inventory_item_id {
            Some(id) => id,
            None => add_inventory_item(ctx, character_id, item_id, 1)?
                .ok_or_else(|| format!("Failed to add {item_id} to damaged demo inventory"))?,
        };
        set_demo_item_damage(ctx, inventory_item_id, bins)?;
    }

    Ok(())
}

pub(super) fn seed_herbalism_supplies(
    ctx: &ReducerContext,
    character_id: u64,
) -> crate::StrategicOperationResult<()> {
    for (item_id, quantity) in [
        ("willow_bark", 2),
        ("comfrey", 2),
        ("poppy", 2),
        ("tincture_spirit", 4),
        ("sage", 2),
        ("glass_bottle", 2),
        ("glass_jar", 1),
        ("mortar_and_pestle", 1),
        ("knife", 1),
    ] {
        if ctx
            .db
            .inventory_item()
            .character_and_item_id()
            .filter((character_id, item_id))
            .next()
            .is_none()
        {
            add_inventory_item(ctx, character_id, item_id, quantity)?
                .ok_or_else(|| format!("Failed to add {item_id} to Herbalism and Foraging Demo"))?;
        }
    }
    Ok(())
}
