//! Guarded real-database inventory projection acceptance for isolated profiles.

use adventuresim_core::{food::FoodPreparation, inventory_measurement::ConsumableFractionMicros};
use adventuresim_world_schema::calendar::StrategicMinute;
use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer};

use crate::{
    CatalogItemKind, FoodLot, InventoryItem, Item,
    food::food_lot,
    inventory_amount::{InventoryItemAmount, inventory_item_amount},
    inventory_container::{
        ContainerLiquid, InventoryContainment, container_liquid, insert_personal_object,
        inventory_containment, inventory_object, object_for_row,
    },
    item::{inventory_item, item},
    tactical::{ConnectedInventory, ConnectedPlayer, connected_players},
};

const COUNTED: &str = "authority-projection-counted";
const LOT: &str = "authority-projection-lot";
const STOCK: &str = "authority-projection-stock";
const CONTAINER: &str = "authority-projection-container";
const EMPTY_CONTAINER: &str = "authority-empty-container";

#[derive(SpacetimeType, Clone, Copy)]
pub enum InventoryProjectionStage {
    Seed,
    Consume,
    Spirit,
    Corrupt,
    Repair,
    CorruptIdentity,
    RepairIdentity,
    CorruptContainment,
    RepairContainment,
    Remove,
}

#[reducer]
pub fn authority_test_empty_inventory_containers(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::character::require_living_character(ctx, character_id)?;
    crate::tactical::local_time::require_unleased_time(ctx, character_id)?;
    if ctx
        .db
        .item()
        .id()
        .find(EMPTY_CONTAINER.to_owned())
        .is_none()
    {
        ctx.db.item().insert(Item {
            id: EMPTY_CONTAINER.into(),
            weight: 1.0,
            kind: CatalogItemKind::Container,
            exterior_volume_ml: 100,
            container_capacity_ml: 1000,
            ..Item::default()
        });
    }
    for index in 0..2 {
        let row = ctx.db.inventory_item().insert(InventoryItem {
            id: 0,
            character_id,
            item_id: EMPTY_CONTAINER.into(),
            quantity: 1,
        });
        let object = insert_personal_object(ctx, &row)?;
        if index == 0 {
            ctx.db.container_liquid().insert(ContainerLiquid {
                container_object_id: object.id,
                liquid_item_id: crate::inventory_container::WATER_ITEM_ID.into(),
                water_ml: 1,
            });
        }
    }
    Ok(())
}

#[reducer]
pub fn authority_test_empty_inventory_mass(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let definition = ctx
        .db
        .item()
        .id()
        .find(EMPTY_CONTAINER.to_owned())
        .ok_or("Missing empty-container fixture definition")?;
    let view = ctx.as_read_only();
    let rows: Vec<_> = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter(|row| row.item_id == EMPTY_CONTAINER)
        .collect();
    if rows.len() != 2 {
        return Err("Empty containers lost their individual rows".into());
    }
    for row in rows {
        let mass = super::personal_row_mass(&view, &row, &definition)
            .map_err(|error| error.to_string())?
            .ok_or("Empty container is not carried")?;
        if row.quantity != 1 || mass.object_id.is_none() || mass.carried.kilograms() != 1.0 {
            return Err("Empty container lost its identity or tare mass".into());
        }
    }
    Ok(())
}

#[reducer]
pub fn authority_test_inventory_projection(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
    stage: InventoryProjectionStage,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    // This is the real registered sender's view, without forged view identity.
    let _before = projected(ctx, character_id)?;
    match stage {
        InventoryProjectionStage::Seed => seed(ctx, character_id)?,
        InventoryProjectionStage::Consume => consume(ctx, character_id)?,
        InventoryProjectionStage::Spirit => {
            let container = row(ctx, character_id, CONTAINER)?;
            let object = object(ctx, &container)?;
            ctx.db
                .container_liquid()
                .container_object_id()
                .update(ContainerLiquid {
                    container_object_id: object,
                    liquid_item_id: "tincture_spirit".into(),
                    water_ml: 150,
                });
        }
        InventoryProjectionStage::Corrupt | InventoryProjectionStage::Repair => {
            let stock = row(ctx, character_id, STOCK)?;
            let fraction = match stage {
                InventoryProjectionStage::Corrupt => ConsumableFractionMicros::MICROS_PER_WHOLE + 1,
                _ => ConsumableFractionMicros::whole_divided_by(4).get(),
            };
            ctx.db
                .inventory_item_amount()
                .inventory_item_id()
                .update(InventoryItemAmount {
                    inventory_item_id: stock.id,
                    remaining_fraction_micros: fraction,
                });
        }
        InventoryProjectionStage::Remove => {
            ctx.db
                .inventory_item()
                .id()
                .delete(row(ctx, character_id, COUNTED)?.id);
        }
        InventoryProjectionStage::CorruptIdentity => set_lot_owner(ctx, character_id, 0)?,
        InventoryProjectionStage::RepairIdentity => set_lot_owner(ctx, character_id, character_id)?,
        InventoryProjectionStage::CorruptContainment => set_stock_parent(ctx, character_id, STOCK)?,
        InventoryProjectionStage::RepairContainment => {
            set_stock_parent(ctx, character_id, CONTAINER)?
        }
    }
    verify(ctx, character_id, stage)
}

fn projected(ctx: &ReducerContext, character: u64) -> Result<ConnectedPlayer, String> {
    connected_players(&ctx.as_read_only())
        .into_iter()
        .find(|profile| profile.character.id == character)
        .ok_or("Fixture actor is not projected to this registered host".into())
}

fn row(ctx: &ReducerContext, character: u64, item_id: &str) -> Result<InventoryItem, String> {
    ctx.db
        .inventory_item()
        .character_id()
        .filter(character)
        .find(|row| row.item_id == item_id)
        .ok_or("Missing fixture inventory row".into())
}

fn object(ctx: &ReducerContext, row: &InventoryItem) -> Result<u64, String> {
    object_for_row(
        ctx,
        adventuresim_core::physical_object::CarriedInventoryScope::Personal,
        row.id,
    )?
    .map(|object| object.id)
    .ok_or("Missing fixture physical object".into())
}

fn seed(ctx: &ReducerContext, character: u64) -> Result<(), String> {
    for (id, weight, quantity) in [
        (COUNTED, 2.0, 3),
        (LOT, 9.0, 1),
        (STOCK, 8.0, 1),
        (CONTAINER, 1.0, 1),
    ] {
        if ctx.db.item().id().find(id.to_owned()).is_none() {
            let kind = match id {
                CONTAINER => CatalogItemKind::Container,
                LOT => CatalogItemKind::Food,
                _ => CatalogItemKind::Simple,
            };
            ctx.db.item().insert(Item {
                id: id.into(),
                weight,
                kind,
                exterior_volume_ml: 1,
                container_capacity_ml: if id == CONTAINER { 2000 } else { 0 },
                ..Item::default()
            });
        }
        if ctx
            .db
            .inventory_item()
            .character_id()
            .filter(character)
            .any(|row| row.item_id == id)
        {
            return Err("Projection fixture has already been seeded".into());
        }
        ctx.db.inventory_item().insert(InventoryItem {
            id: 0,
            character_id: character,
            item_id: id.into(),
            quantity,
        });
    }
    let lot = row(ctx, character, LOT)?;
    insert_personal_object(ctx, &lot)?;
    ctx.db.food_lot().insert(FoodLot {
        id: 0,
        inventory_item_id: Some(lot.id),
        party_inventory_item_id: None,
        material_revision: 1,
        display_name: "Measured fixture batch".into(),
        preparation: FoodPreparation::Raw,
        ingredient_item_ids: vec![],
        ingredient_quantities: vec![],
        salty_kg: 0.0,
        spicy_kg: 0.0,
        sweet_kg: 0.0,
        sour_kg: 0.0,
        savory_kg: 0.0,
        quality: 0,
        mass_kg: 1.5,
        nutrition_kcal: 1.0,
        total_value: 1.0,
        created_at_minute: StrategicMinute::new(0),
    });
    let stock = row(ctx, character, STOCK)?;
    ctx.db.inventory_item_amount().insert(InventoryItemAmount {
        inventory_item_id: stock.id,
        remaining_fraction_micros: ConsumableFractionMicros::whole_divided_by(2).get(),
    });
    let stock_object = insert_personal_object(ctx, &stock)?;
    let container = insert_personal_object(ctx, &row(ctx, character, CONTAINER)?)?;
    ctx.db.inventory_containment().insert(InventoryContainment {
        child_object_id: stock_object.id,
        parent_object_id: container.id,
    });
    ctx.db.container_liquid().insert(ContainerLiquid {
        container_object_id: container.id,
        liquid_item_id: crate::inventory_container::WATER_ITEM_ID.into(),
        water_ml: 1500,
    });
    Ok(())
}

fn consume(ctx: &ReducerContext, character: u64) -> Result<(), String> {
    let mut counted = row(ctx, character, COUNTED)?;
    counted.quantity = 2;
    ctx.db.inventory_item().id().update(counted);
    let lot = row(ctx, character, LOT)?;
    let mut lot = ctx
        .db
        .food_lot()
        .iter()
        .find(|candidate| candidate.inventory_item_id == Some(lot.id))
        .ok_or("Missing fixture food lot")?;
    lot.mass_kg = 0.5;
    ctx.db.food_lot().id().update(lot);
    let stock = row(ctx, character, STOCK)?;
    ctx.db
        .inventory_item_amount()
        .inventory_item_id()
        .update(InventoryItemAmount {
            inventory_item_id: stock.id,
            remaining_fraction_micros: ConsumableFractionMicros::whole_divided_by(4).get(),
        });
    let container = row(ctx, character, CONTAINER)?;
    ctx.db
        .container_liquid()
        .container_object_id()
        .update(ContainerLiquid {
            container_object_id: object(ctx, &container)?,
            liquid_item_id: crate::inventory_container::WATER_ITEM_ID.into(),
            water_ml: 500,
        });
    Ok(())
}

fn verify(
    ctx: &ReducerContext,
    character: u64,
    stage: InventoryProjectionStage,
) -> Result<(), String> {
    let profile = projected(ctx, character)?;
    let expected_error = match stage {
        InventoryProjectionStage::Corrupt => {
            Some(super::InventoryProjectionError::InvalidRemainingMaterial)
        }
        InventoryProjectionStage::CorruptIdentity => {
            Some(super::InventoryProjectionError::InvalidIdentity)
        }
        InventoryProjectionStage::CorruptContainment => {
            Some(super::InventoryProjectionError::InvalidContainment)
        }
        _ => None,
    };
    if let Some(expected) = expected_error {
        return match profile.inventory {
            ConnectedInventory::Unavailable(actual) if actual == expected => Ok(()),
            _ => Err(
                "Corrupt inventory did not return the expected explicit projection error".into(),
            ),
        };
    }
    let ConnectedInventory::Available(items) = profile.inventory else {
        return Err("Fixture inventory is unavailable".into());
    };
    let expected = match stage {
        InventoryProjectionStage::Seed => {
            [(COUNTED, 6.0), (LOT, 1.5), (STOCK, 4.0), (CONTAINER, 2.5)]
        }
        InventoryProjectionStage::Consume => {
            [(COUNTED, 4.0), (LOT, 0.5), (STOCK, 2.0), (CONTAINER, 1.5)]
        }
        _ => [
            (COUNTED, 4.0),
            (LOT, 0.5),
            (STOCK, 2.0),
            (CONTAINER, 1.0 + 0.12),
        ],
    };
    for (id, mass) in expected {
        if id == COUNTED && matches!(stage, InventoryProjectionStage::Remove) {
            if items.iter().any(|item| item.item.id == id) {
                return Err("Removed inventory row remained in projection".into());
            }
            continue;
        }
        let item = items
            .iter()
            .find(|item| item.item.id == id)
            .ok_or("Fixture row missing from projection")?;
        if item.carried_mass_kg != mass {
            return Err(format!(
                "{id}: expected {mass}kg, received {}kg",
                item.carried_mass_kg
            ));
        }
        if matches!(id, STOCK | CONTAINER)
            && item.physical_object_id != Some(object(ctx, &row(ctx, character, id)?)?)
        {
            return Err("Projection lost stable physical identity".into());
        }
    }
    Ok(())
}

fn set_lot_owner(ctx: &ReducerContext, character: u64, owner: u64) -> Result<(), String> {
    let lot = row(ctx, character, LOT)?;
    let mut object = ctx
        .db
        .inventory_object()
        .id()
        .find(object(ctx, &lot)?)
        .ok_or("Missing fixture food object")?;
    object.location =
        adventuresim_core::physical_object::InventoryLocation::personal(owner, lot.id);
    ctx.db.inventory_object().id().update(object);
    Ok(())
}

fn set_stock_parent(ctx: &ReducerContext, character: u64, parent: &str) -> Result<(), String> {
    let child = object(ctx, &row(ctx, character, STOCK)?)?;
    let parent = object(ctx, &row(ctx, character, parent)?)?;
    ctx.db
        .inventory_containment()
        .child_object_id()
        .update(InventoryContainment {
            child_object_id: child,
            parent_object_id: parent,
        });
    Ok(())
}
