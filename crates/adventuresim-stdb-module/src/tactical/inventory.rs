//! The registered host's complete durable personal inventory projection.

use super::*;
use crate::inventory_projection::InventoryProjectionError;

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedPlayerItem {
    pub inventory_item_id: u64,
    pub quantity: u32,
    pub physical_object_id: Option<u64>,
    pub unit_mass_kg: f32,
    pub carried_mass_kg: f32,
    pub item: Item,
    pub selected_placement_id: Option<String>,
    pub occupancies: Vec<ConnectedEquipmentOccupancy>,
    pub protected_body_parts: Vec<adventuresim_core::item_catalog::EquipmentBodyPart>,
    pub condition: Option<ItemCondition>,
    pub weapon_appearance: Option<crate::weapon_instance::ConnectedWeaponAppearance>,
    pub weapon_holder_appearance: Option<crate::weapon_instance::ConnectedWeaponAppearance>,
}

#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedEquipmentOccupancy {
    pub id: String,
    pub anchor_kind: crate::character::EquipmentAnchorKind,
    pub location: Option<adventuresim_core::item_catalog::EquipmentLocation>,
    pub parent_inventory_item_id: Option<u64>,
    pub attachment_point_id: Option<String>,
    pub channel: adventuresim_core::item_catalog::EquipmentChannel,
    pub order: u16,
    pub requirement_index: u16,
    pub capacity_index: u16,
}

#[derive(SpacetimeType, Clone, Debug)]
pub enum ConnectedInventory {
    Available(Vec<ConnectedPlayerItem>),
    Unavailable(InventoryProjectionError),
}

impl ConnectedPlayerItem {
    fn from_inventory(
        ctx: &ViewContext,
        inventory_item: crate::InventoryItem,
    ) -> Result<Option<Self>, InventoryProjectionError> {
        let mut item = ctx
            .db
            .item()
            .id()
            .find(&inventory_item.item_id)
            .ok_or(InventoryProjectionError::MissingDefinition)?;
        let Some(mass) =
            crate::inventory_projection::personal_row_mass(ctx, &inventory_item, &item)?
        else {
            return Ok(None);
        };
        let weapon_appearance = crate::weapon_instance::connected_appearance(
            ctx,
            inventory_item.id,
            &inventory_item.item_id,
        );
        let weapon_holder_appearance = crate::weapon_instance::connected_holder_appearance(
            ctx,
            inventory_item.id,
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
        let mut occupancies: Vec<_> = ctx
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
        occupancies.sort_by(|first, second| first.id.cmp(&second.id));
        let protected_body_parts = placement
            .map(|placement| placement.protection.clone())
            .unwrap_or_default();
        Ok(Some(Self {
            inventory_item_id: inventory_item.id,
            quantity: inventory_item.quantity,
            physical_object_id: mass.object_id.map(|id| id.get()),
            unit_mass_kg: mass.unit.kilograms(),
            carried_mass_kg: mass.carried.kilograms(),
            item,
            selected_placement_id: worn.map(|row| row.placement_id),
            condition,
            occupancies,
            protected_body_parts,
            weapon_appearance,
            weapon_holder_appearance,
        }))
    }
}

pub(super) fn connected_inventory(ctx: &ViewContext, character_id: u64) -> ConnectedInventory {
    match connected_player_items(ctx, character_id) {
        Ok(items) => ConnectedInventory::Available(items),
        Err(error) => ConnectedInventory::Unavailable(error),
    }
}

fn connected_player_items(
    ctx: &ViewContext,
    character_id: u64,
) -> Result<Vec<ConnectedPlayerItem>, InventoryProjectionError> {
    let mut items: Vec<_> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .map(|row| ConnectedPlayerItem::from_inventory(ctx, row))
        .filter_map(Result::transpose)
        .collect::<Result<_, _>>()?;
    items.sort_by_key(|item| item.inventory_item_id);
    Ok(items)
}
