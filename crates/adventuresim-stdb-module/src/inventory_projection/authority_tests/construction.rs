//! Corrupt construction projections after warming the actual carried-load path.

use super::*;
use crate::{
    inventory_projection::{InventoryProjectionError, personal_row_mass},
    weapon_instance::{
        WeaponHolderInstance, WeaponInstance, weapon_holder_instance, weapon_instance,
    },
};

const CORRUPTIONS: [ConstructionCorruption; 7] = [
    ConstructionCorruption::Generator,
    ConstructionCorruption::Hash,
    ConstructionCorruption::Mass,
    ConstructionCorruption::Length,
    ConstructionCorruption::Grip,
    ConstructionCorruption::NonCanonicalRecipe,
    ConstructionCorruption::TruncatedRecipe,
];

#[derive(Clone, Copy)]
enum ConstructionCorruption {
    Generator,
    Hash,
    Mass,
    Length,
    Grip,
    NonCanonicalRecipe,
    TruncatedRecipe,
}

macro_rules! corrupt_construction {
    ($row:ident, $corruption:expr) => {
        match $corruption {
            ConstructionCorruption::Generator => $row.generator_version += 1,
            ConstructionCorruption::Hash => {
                *$row
                    .design_hash
                    .first_mut()
                    .ok_or("Empty construction hash")? ^= 1
            }
            ConstructionCorruption::Mass => $row.mass_grams += 1,
            ConstructionCorruption::Length => $row.length_mm += 1,
            ConstructionCorruption::Grip => $row.grip_to_tip_mm += 1,
            ConstructionCorruption::NonCanonicalRecipe => $row.recipe.insert(0, b' '),
            ConstructionCorruption::TruncatedRecipe => {
                $row.recipe.pop().ok_or("Empty construction recipe")?;
            }
        }
    };
}

#[reducer]
pub fn authority_test_inventory_construction_validation(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::character::require_living_character(ctx, character_id)?;
    let mut saw_weapon = false;
    let mut saw_holder = false;
    for row in ctx.db.inventory_item().character_id().filter(character_id) {
        let Some(object) = object_for_row(
            ctx,
            adventuresim_core::physical_object::CarriedInventoryScope::Personal,
            row.id,
        )?
        else {
            continue;
        };
        let definition = ctx
            .db
            .item()
            .id()
            .find(&row.item_id)
            .ok_or("Missing construction catalog")?;
        if let Some(instance) = ctx
            .db
            .weapon_instance()
            .physical_object_id()
            .find(object.id)
        {
            check_weapon(ctx, &row, &definition, instance)?;
            saw_weapon = true;
        }
        if let Some(instance) = ctx
            .db
            .weapon_holder_instance()
            .physical_object_id()
            .find(object.id)
        {
            check_holder(ctx, &row, &definition, instance)?;
            saw_holder = true;
        }
    }
    if !saw_weapon || !saw_holder {
        return Err("Construction validation requires a weapon and a fitted holder".into());
    }
    Ok(())
}

fn check_weapon(
    ctx: &ReducerContext,
    row: &InventoryItem,
    definition: &Item,
    valid: WeaponInstance,
) -> Result<(), String> {
    let original = personal_row_mass(&ctx.as_read_only(), row, definition)
        .map_err(|error| error.to_string())?
        .ok_or("Construction is not carried")?;
    for corruption in CORRUPTIONS {
        let mut changed = valid.clone();
        corrupt_construction!(changed, corruption);
        ctx.db
            .weapon_instance()
            .physical_object_id()
            .update(changed);
        if !matches!(
            personal_row_mass(&ctx.as_read_only(), row, definition),
            Err(InventoryProjectionError::InvalidConstruction)
        ) {
            return Err("Corrupt weapon projection passed warmed carried-load validation".into());
        }
        ctx.db
            .weapon_instance()
            .physical_object_id()
            .update(valid.clone());
        let restored = personal_row_mass(&ctx.as_read_only(), row, definition)
            .map_err(|error| error.to_string())?
            .ok_or("Restored construction is not carried")?;
        if restored.carried != original.carried || restored.unit != original.unit {
            return Err("Restored weapon construction changed carried load".into());
        }
    }
    Ok(())
}

fn check_holder(
    ctx: &ReducerContext,
    row: &InventoryItem,
    definition: &Item,
    valid: WeaponHolderInstance,
) -> Result<(), String> {
    let original = personal_row_mass(&ctx.as_read_only(), row, definition)
        .map_err(|error| error.to_string())?
        .ok_or("Construction is not carried")?;
    for corruption in CORRUPTIONS {
        let mut changed = valid.clone();
        corrupt_construction!(changed, corruption);
        ctx.db
            .weapon_holder_instance()
            .physical_object_id()
            .update(changed);
        if !matches!(
            personal_row_mass(&ctx.as_read_only(), row, definition),
            Err(InventoryProjectionError::InvalidConstruction)
        ) {
            return Err("Corrupt holder projection passed warmed carried-load validation".into());
        }
        ctx.db
            .weapon_holder_instance()
            .physical_object_id()
            .update(valid.clone());
        let restored = personal_row_mass(&ctx.as_read_only(), row, definition)
            .map_err(|error| error.to_string())?
            .ok_or("Restored construction is not carried")?;
        if restored.carried != original.carried || restored.unit != original.unit {
            return Err("Restored holder construction changed carried load".into());
        }
    }
    Ok(())
}
