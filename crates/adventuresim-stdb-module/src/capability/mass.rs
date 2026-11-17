//! Strategic calculations use the same checked load as the live projection.

use adventuresim_core::inventory_measurement::InventoryMassKg;
use spacetimedb::ReducerContext;

use crate::{
    inventory_item,
    inventory_projection::{InventoryProjectionError, personal_row_mass},
    item::item,
};

pub(super) fn inventory_weight(ctx: &ReducerContext, character_id: u64) -> f32 {
    let view = ctx.as_read_only();
    let mass: Result<InventoryMassKg, InventoryProjectionError> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .try_fold(InventoryMassKg::ZERO, |total, row| {
            let definition = ctx
                .db
                .item()
                .id()
                .find(&row.item_id)
                .ok_or(InventoryProjectionError::MissingDefinition)?;
            let Some(mass) = personal_row_mass(&view, &row, &definition)? else {
                return Ok(total);
            };
            InventoryMassKg::try_from(total.kilograms() + mass.carried.kilograms())
                .map_err(|_| InventoryProjectionError::InvalidMass)
        });
    // The strategic numerical kernel already treats an unavailable load as
    // infinite burden. Native admission receives the explicit projection error.
    mass.map_or_else(
        |error| {
            log::error!("Cannot evaluate character {character_id} inventory: {error}");
            f32::INFINITY
        },
        |mass| mass.kilograms(),
    )
}
