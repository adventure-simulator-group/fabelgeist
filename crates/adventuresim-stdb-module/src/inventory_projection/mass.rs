//! Exact carried row mass; both read views and reducers use this producer.

use adventuresim_core::{
    inventory_measurement::{ConsumableFractionMicros, InventoryMassKg, ItemQuantity},
    material::{Microliters, Milliliters},
    physical_object::{CarriedInventoryScope, InventoryLocation, PhysicalObjectId},
};
use spacetimedb::ViewContext;

use super::{InventoryProjectionError, InventoryRowMass, custody::InventoryCarriage};
use crate::{
    InventoryItem, Item,
    food::food_lot__view,
    inventory_amount::inventory_item_amount__view,
    inventory_container::{WATER_ITEM_ID, container_liquid__view, object_for_row},
    item::item__view,
};

type Result<T> = std::result::Result<T, InventoryProjectionError>;

pub(crate) fn personal_row_mass(
    ctx: &ViewContext,
    row: &InventoryItem,
    item: &Item,
) -> Result<Option<InventoryRowMass>> {
    let quantity =
        ItemQuantity::new(row.quantity).ok_or(InventoryProjectionError::InvalidQuantity)?;
    let object = object_for_row(ctx, CarriedInventoryScope::Personal, row.id)
        .map_err(|_| InventoryProjectionError::InvalidIdentity)?;
    let fraction = remaining_fraction(ctx, row.id)?;
    let food = adventuresim_core::food::definition(&item.id).is_some();
    let measured =
        fraction.is_some() || crate::inventory_amount::is_measured_definition(item) || food;
    if crate::item::requires_stable_object(Some(item), food, measured) && object.is_none() {
        return Err(InventoryProjectionError::InvalidIdentity);
    }
    if fraction.is_some() && quantity != ItemQuantity::ONE {
        return Err(InventoryProjectionError::InvalidRemainingMaterial);
    }
    let object_id = object
        .as_ref()
        .map(|object| {
            if object.item_id != row.item_id
                || row.quantity != 1
                || object.location != InventoryLocation::personal(row.character_id, row.id)
            {
                return Err(InventoryProjectionError::InvalidIdentity);
            }
            PhysicalObjectId::try_new(object.id)
                .map_err(|_| InventoryProjectionError::InvalidIdentity)
        })
        .transpose()?;
    if let Some(object) = &object {
        match InventoryCarriage::for_personal_object(ctx, object, row.character_id)? {
            InventoryCarriage::Carried => {}
            InventoryCarriage::StoredAtFireplace => return Ok(None),
        }
    }
    let unit = construction_mass(ctx, row, item)?;
    let mut lots = ctx
        .db
        .food_lot()
        .material_revision()
        .filter(0u64..)
        .filter(|lot| lot.inventory_item_id == Some(row.id));
    let lot = lots.next();
    if lots.next().is_some() {
        return Err(InventoryProjectionError::InvalidRemainingMaterial);
    }
    let dry = if let Some(lot) = lot {
        InventoryMassKg::try_from(lot.mass_kg).map_err(|_| InventoryProjectionError::InvalidMass)?
    } else {
        // Checked kg and count/fraction enter the existing f32 load kernel here.
        let amount = fraction.map_or(quantity.get() as f32, |fraction| fraction.as_unit_f32());
        InventoryMassKg::try_from(unit.kilograms() * amount)
            .map_err(|_| InventoryProjectionError::InvalidMass)?
    };
    let liquid = object_id
        .map(|object_id| liquid_mass(ctx, object_id))
        .transpose()?
        .unwrap_or(InventoryMassKg::ZERO);
    let carried = InventoryMassKg::try_from(dry.kilograms() + liquid.kilograms())
        .map_err(|_| InventoryProjectionError::InvalidMass)?;
    Ok(Some(InventoryRowMass {
        object_id,
        unit,
        carried,
    }))
}

fn remaining_fraction(ctx: &ViewContext, row_id: u64) -> Result<Option<ConsumableFractionMicros>> {
    ctx.db
        .inventory_item_amount()
        .inventory_item_id()
        .find(row_id)
        .map(|amount| ConsumableFractionMicros::try_new(amount.remaining_fraction_micros))
        .transpose()
        .map_err(|_| InventoryProjectionError::InvalidRemainingMaterial)
}

fn construction_mass(
    ctx: &ViewContext,
    row: &InventoryItem,
    item: &Item,
) -> Result<InventoryMassKg> {
    let kilograms = if adventuresim_weapon_model::default_design(&item.id).is_some() {
        crate::weapon_instance::combat_geometry(ctx, row.id, &item.id)
            .ok_or(InventoryProjectionError::InvalidConstruction)?
            .mass_kg
    } else {
        crate::weapon_instance::fitted_holder_mass(ctx, row.id, &item.id)
            .map_err(|_| InventoryProjectionError::InvalidConstruction)?
            .unwrap_or(item.weight)
    };
    InventoryMassKg::try_from(kilograms).map_err(|_| InventoryProjectionError::InvalidMass)
}

fn liquid_mass(ctx: &ViewContext, object_id: PhysicalObjectId) -> Result<InventoryMassKg> {
    let Some(liquid) = ctx
        .db
        .container_liquid()
        .container_object_id()
        .find(object_id.get())
    else {
        return Ok(InventoryMassKg::ZERO);
    };
    let volume = Milliliters::new(liquid.water_ml);
    let kilograms = if liquid.liquid_item_id == WATER_ITEM_ID {
        Microliters::try_from_milliliters(volume)
            .map_err(|_| InventoryProjectionError::InvalidLiquidVolume)?
            .as_water_kilograms_f32()
    } else {
        let definition = ctx
            .db
            .item()
            .id()
            .find(&liquid.liquid_item_id)
            .ok_or(InventoryProjectionError::MissingDefinition)?;
        if definition.exterior_volume_ml == 0 {
            return Err(InventoryProjectionError::InvalidLiquidVolume);
        }
        let unit = InventoryMassKg::try_from(definition.weight)
            .map_err(|_| InventoryProjectionError::InvalidMass)?;
        // Authored unit kg / authored unit ml gives this substance's density.
        unit.kilograms() * (volume.get() as f32 / definition.exterior_volume_ml as f32)
    };
    InventoryMassKg::try_from(kilograms).map_err(|_| InventoryProjectionError::InvalidMass)
}
