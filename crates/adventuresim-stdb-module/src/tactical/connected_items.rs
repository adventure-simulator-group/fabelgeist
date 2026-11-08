//! Project carried inventory rows into tactical player items.

use super::*;

pub(super) fn connected_player_items(
    ctx: &ViewContext,
    character_id: u64,
) -> impl Iterator<Item = ConnectedPlayerItem> + use<'_> {
    ctx.db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter_map(move |inventory_item| {
            let mut item = ctx.db.item().id().find(&inventory_item.item_id)?;
            let inventory_row_id = PersonalInventoryRowId::from_raw(inventory_item.id);
            let weapon_appearance = crate::weapon_instance::connected_appearance(
                ctx,
                inventory_row_id,
                &inventory_item.item_id,
            );
            let weapon_holder_appearance = crate::weapon_instance::connected_holder_appearance(
                ctx,
                inventory_row_id,
                &inventory_item.item_id,
            );
            let condition = ctx
                .db
                .item_condition()
                .inventory_item_id()
                .find(inventory_item.id);
            if let Some(condition) = &condition {
                let damage = condition.bins();
                item.precision = adventuresim_core::durability::effective_weapon_stat(
                    item.precision,
                    damage,
                    item.edge_sensitivity,
                );
                item.block = adventuresim_core::durability::effective_weapon_stat(
                    item.block,
                    damage,
                    item.handling_sensitivity,
                );
                item.range_of_motion = adventuresim_core::durability::effective_handling(
                    item.range_of_motion,
                    damage,
                    item.handling_sensitivity,
                );
            }
            let worn = ctx
                .db
                .character_equipped_item()
                .inventory_item_id()
                .find(inventory_item.id);
            let placement = worn.as_ref().and_then(|worn| {
                item.equipment_placements
                    .iter()
                    .find(|placement| placement.id == worn.placement_id)
            });
            let occupancies = ctx
                .db
                .equipment_occupancy()
                .inventory_item_id()
                .filter(inventory_item.id)
                .map(|row| ConnectedEquipmentOccupancy {
                    id: row.id,
                    anchor_kind: row.anchor_kind,
                    location: row.location,
                    parent_inventory_item_id: row.parent_inventory_item_id,
                    attachment_point_id: row.attachment_point_id,
                    channel: row.channel,
                    order: row.order,
                    requirement_index: row.requirement_index,
                    capacity_index: row.capacity_index,
                })
                .collect();
            let protected_body_parts = placement
                .map(|placement| placement.protection.clone())
                .unwrap_or_default();
            Some(ConnectedPlayerItem {
                inventory_item_id: inventory_item.id,
                quantity: inventory_item.quantity,
                item,
                selected_placement_id: worn.map(|row| row.placement_id),
                condition,
                occupancies,
                protected_body_parts,
                weapon_appearance,
                weapon_holder_appearance,
            })
        })
}
