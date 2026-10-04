//! Durable parametric weapon recipes keyed by stable physical-object identity.

use adventuresim_core::physical_object::{CarriedInventoryScope, InventoryLocation};
use adventuresim_weapon_model::{
    GENERATOR_VERSION, HOLDER_GENERATOR_VERSION, WeaponDesign, WeaponHolderDesign, default_design,
    default_holder_design, design_hash, holder_design_hash,
};
use spacetimedb::{ReducerContext, SpacetimeType, Table, ViewContext, table, view};

use crate::inventory_container::inventory_object__view;
use crate::item::item;
use crate::strategic::PartyInventoryItem;
use crate::strategic::strategic_gateway_authority__view;
use crate::{CatalogItemKind, InventoryItem, inventory_object};

mod appearance;
mod authentication_error;
use authentication_error::WeaponAuthenticationError;
mod error;
mod evaluation;
pub(crate) use appearance::{connected_appearance, connected_holder_appearance};
pub(crate) use error::{WeaponInstanceError, WeaponProjectionError};
use error::{WeaponProjectionField, WeaponRecipeKind};
use evaluation::{evaluate_holder_instance, evaluate_instance};

pub const MAX_WEAPON_RECIPE_BYTES: usize = adventuresim_weapon_model::MAX_ENCODED_RECIPE_BYTES;

fn checked_scaled_u32(
    value: f32,
    scale: f32,
    label: WeaponProjectionField,
) -> Result<u32, WeaponProjectionError> {
    let scaled = value * scale;
    if !scaled.is_finite() || scaled < 0.0 || scaled > u32::MAX as f32 {
        return Err(WeaponProjectionError::OutOfRange(label));
    }
    Ok(scaled.round() as u32)
}

#[derive(Clone, Debug, PartialEq)]
/// Immutable evaluated recipe and storage projections, owned by
/// `from_evaluation`. Consumption re-evaluates the recipe and compares every
/// projection; catalog defaults never rewrite an existing physical object.
#[table(accessor = weapon_instance)]
pub struct WeaponInstance {
    #[primary_key]
    pub physical_object_id: u64,
    pub generator_version: u16,
    pub design_hash: Vec<u8>,
    pub recipe: Vec<u8>,
    pub mass_grams: u32,
    pub length_mm: u32,
    pub grip_to_tip_mm: u32,
}

#[derive(Clone, Debug, PartialEq)]
/// Independently evaluated holder recipe with the same construction and
/// authentication contract as `WeaponInstance`; holder geometry is distinct.
#[table(accessor = weapon_holder_instance)]
pub struct WeaponHolderInstance {
    #[primary_key]
    pub physical_object_id: u64,
    pub generator_version: u16,
    pub design_hash: Vec<u8>,
    pub recipe: Vec<u8>,
    pub mass_grams: u32,
    pub length_mm: u32,
    pub grip_to_tip_mm: u32,
}

/// Trusted strategic-backend projection used to produce per-instance inventory
/// icons. Recipes remain absent for ordinary player subscriptions and HTML.
#[view(accessor = backend_weapon_instances, public)]
pub fn backend_weapon_instances(ctx: &ViewContext) -> Vec<WeaponInstance> {
    let gateway = ctx.db.strategic_gateway_authority().id().find(0);
    if gateway.is_none_or(|authority| authority.identity != ctx.sender()) {
        return Vec::new();
    }
    let mut instances = Vec::new();
    for object in ctx
        .db
        .inventory_object()
        .item_id()
        .filter(""..)
        .filter(|object| {
            matches!(
                &object.location,
                InventoryLocation::Personal(_) | InventoryLocation::Party(_)
            )
        })
    {
        if let Some(instance) = ctx
            .db
            .weapon_instance()
            .physical_object_id()
            .find(object.id)
        {
            instances.push(instance);
        }
    }
    instances
}

/// Trusted strategic-backend projection for independently persisted scabbard
/// and haft-loop recipes. Ordinary players still receive holder recipes only
/// through the sender-scoped tactical projection.
#[view(accessor = backend_weapon_holder_instances, public)]
pub fn backend_weapon_holder_instances(ctx: &ViewContext) -> Vec<WeaponHolderInstance> {
    let gateway = ctx.db.strategic_gateway_authority().id().find(0);
    if gateway.is_none_or(|authority| authority.identity != ctx.sender()) {
        return Vec::new();
    }
    let mut instances = Vec::new();
    for object in ctx
        .db
        .inventory_object()
        .item_id()
        .filter(""..)
        .filter(|object| {
            matches!(
                &object.location,
                InventoryLocation::Personal(_) | InventoryLocation::Party(_)
            )
        })
    {
        if let Some(instance) = ctx
            .db
            .weapon_holder_instance()
            .physical_object_id()
            .find(object.id)
        {
            instances.push(instance);
        }
    }
    instances
}

/// Sender-scoped tactical transport. Strategic HTML likewise never embeds the
/// full smithing recipe; its trusted icon endpoint consumes the backend view.
#[derive(SpacetimeType, Clone, Debug)]
pub struct ConnectedWeaponAppearance {
    pub generator_version: u16,
    pub design_hash: Vec<u8>,
    pub recipe: Vec<u8>,
    pub mass_kg: f32,
    pub length_m: f32,
    pub grip_to_tip_m: f32,
}

pub(crate) fn initialize_personal_weapon(
    ctx: &ReducerContext,
    inventory: &InventoryItem,
) -> Result<(), WeaponInstanceError> {
    let Some(definition) = ctx.db.item().id().find(inventory.item_id.clone()) else {
        return Err(WeaponInstanceError::MissingDefinition(
            (&inventory.item_id).into(),
        ));
    };
    if definition.kind != CatalogItemKind::Weapon || !definition.melee {
        return Ok(());
    }
    let Some(design) = default_design(&inventory.item_id) else {
        return Ok(());
    };
    if inventory.quantity != 1 {
        return Err(WeaponInstanceError::NonIndividual {
            scope: CarriedInventoryScope::Personal,
            row: inventory.id.into(),
        });
    }
    let object = crate::inventory_container::object_for_row(
        ctx,
        CarriedInventoryScope::Personal,
        (inventory.id).into(),
    )?
    .ok_or(WeaponInstanceError::MissingStableObject {
        scope: CarriedInventoryScope::Personal,
        row: inventory.id.into(),
    })?;
    replace_design(ctx, object.id, &design)?;
    Ok(())
}

pub(crate) fn initialize_party_weapon(
    ctx: &ReducerContext,
    inventory: &PartyInventoryItem,
) -> Result<(), WeaponInstanceError> {
    let Some(definition) = ctx.db.item().id().find(inventory.item_id.clone()) else {
        return Err(WeaponInstanceError::MissingDefinition(
            (&inventory.item_id).into(),
        ));
    };
    if definition.kind != CatalogItemKind::Weapon || !definition.melee {
        return Ok(());
    }
    let Some(design) = default_design(&inventory.item_id) else {
        return Ok(());
    };
    if inventory.quantity != 1 {
        return Err(WeaponInstanceError::NonIndividual {
            scope: CarriedInventoryScope::Party,
            row: inventory.id.into(),
        });
    }
    let object = crate::inventory_container::object_for_row(
        ctx,
        CarriedInventoryScope::Party,
        (inventory.id).into(),
    )?
    .ok_or(WeaponInstanceError::MissingStableObject {
        scope: CarriedInventoryScope::Party,
        row: inventory.id.into(),
    })?;
    replace_design(ctx, object.id, &design)?;
    Ok(())
}

pub(crate) fn delete_for_object(ctx: &ReducerContext, physical_object_id: u64) {
    ctx.db
        .weapon_instance()
        .physical_object_id()
        .delete(physical_object_id);
    ctx.db
        .weapon_holder_instance()
        .physical_object_id()
        .delete(physical_object_id);
}

pub(crate) fn fit_personal_holder(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    holder_inventory_row_id: adventuresim_core::identity::InventoryItemId,
    weapon_inventory_row_id: adventuresim_core::identity::InventoryItemId,
) -> Result<(), WeaponInstanceError> {
    let holder_object = crate::inventory_container::require_object(
        ctx,
        character_id,
        CarriedInventoryScope::Personal,
        holder_inventory_row_id,
    )?;
    let weapon_object = crate::inventory_container::object_for_row(
        ctx,
        CarriedInventoryScope::Personal,
        weapon_inventory_row_id,
    )?
    .ok_or(WeaponInstanceError::MissingStableObject {
        scope: CarriedInventoryScope::Personal,
        row: weapon_inventory_row_id,
    })?;
    let weapon = ctx
        .db
        .weapon_instance()
        .physical_object_id()
        .find(weapon_object.id)
        .ok_or(WeaponInstanceError::MissingParametricRecipe(
            weapon_inventory_row_id,
        ))?;
    let weapon_key =
        adventuresim_core::item_catalog::ItemDefinitionId::from(&weapon_object.item_id);
    let evaluated = evaluate_instance(&weapon, &weapon_key)?;
    let expected_holder: adventuresim_core::item_catalog::ItemDefinitionId =
        match adventuresim_weapon_model::recommended_holder(&weapon_object.item_id) {
            Some(adventuresim_weapon_model::WeaponHolderKind::BladeSheath) => "scabbard".into(),
            Some(adventuresim_weapon_model::WeaponHolderKind::HaftLoop) => "weapon_loop".into(),
            None => return Err(WeaponInstanceError::UnsupportedBodyHolder(weapon_key)),
        };
    if holder_object.item_id != expected_holder.as_str() {
        return Err(WeaponInstanceError::WrongHolder {
            weapon: weapon_key,
            expected: expected_holder,
            supplied: holder_object.item_id.into(),
        });
    }
    let weapon_design = evaluated.design();
    let holder_design = default_holder_design(weapon_design)
        .ok_or(WeaponInstanceError::MissingHolderTemplate(weapon_key))?;
    replace_holder_design(ctx, holder_object.id, &holder_design)?;
    Ok(())
}

/// Atomic holder-design replacement seam for a future leatherworking/smithing
/// reducer. Holder parameters and the fitted weapon snapshot are independently
/// versioned; changing the weapon does not silently rewrite an existing holder.
pub(crate) fn replace_holder_design(
    ctx: &ReducerContext,
    physical_object_id: u64,
    design: &WeaponHolderDesign,
) -> Result<(), WeaponInstanceError> {
    let object = ctx
        .db
        .inventory_object()
        .id()
        .find(physical_object_id)
        .ok_or(WeaponInstanceError::MissingPhysicalObject(
            WeaponRecipeKind::Holder,
        ))?;
    if object.item_id != design.catalog_id {
        return Err(WeaponInstanceError::ChassisMismatch {
            role: WeaponRecipeKind::Holder,
            expected: object.item_id.into(),
            supplied: (&design.catalog_id).into(),
        });
    }
    let fit = WeaponHolderInstance::from_design(physical_object_id, design)?;
    if ctx
        .db
        .weapon_holder_instance()
        .physical_object_id()
        .find(physical_object_id)
        .is_some()
    {
        ctx.db
            .weapon_holder_instance()
            .physical_object_id()
            .update(fit);
    } else {
        ctx.db.weapon_holder_instance().insert(fit);
    }
    Ok(())
}

/// Atomic design replacement seam for the future smithing reducer. Authority,
/// material consumption, skill checks, price, and elapsed time belong to that
/// reducer; this function owns recipe validation and derived-property refresh.
pub(crate) fn replace_design(
    ctx: &ReducerContext,
    physical_object_id: u64,
    design: &WeaponDesign,
) -> Result<(), WeaponInstanceError> {
    let object = ctx
        .db
        .inventory_object()
        .id()
        .find(physical_object_id)
        .ok_or(WeaponInstanceError::MissingPhysicalObject(
            WeaponRecipeKind::Weapon,
        ))?;
    let definition = ctx
        .db
        .item()
        .id()
        .find(object.item_id.clone())
        .ok_or(WeaponInstanceError::MissingCatalogDefinition)?;
    if definition.kind != CatalogItemKind::Weapon || !definition.melee {
        return Err(WeaponInstanceError::NotMelee(object.item_id.into()));
    }
    if design.catalog_id != object.item_id {
        return Err(WeaponInstanceError::ChassisMismatch {
            role: WeaponRecipeKind::Weapon,
            expected: object.item_id.into(),
            supplied: (&design.catalog_id).into(),
        });
    }
    let instance = WeaponInstance::from_design(physical_object_id, design)?;
    if ctx
        .db
        .weapon_instance()
        .physical_object_id()
        .find(physical_object_id)
        .is_some()
    {
        ctx.db
            .weapon_instance()
            .physical_object_id()
            .update(instance.clone());
    } else {
        ctx.db.weapon_instance().insert(instance.clone());
    }
    Ok(())
}

pub(crate) fn combat_geometry(
    ctx: &ReducerContext,
    inventory_row_id: u64,
    item_id: &str,
) -> Option<adventuresim_core::equipment::ParametricWeaponCombatGeometry> {
    let object = crate::inventory_container::object_for_row(
        ctx,
        CarriedInventoryScope::Personal,
        (inventory_row_id).into(),
    )
    .ok()?
    .filter(|object| object.item_id == item_id)?;
    let instance = ctx
        .db
        .weapon_instance()
        .physical_object_id()
        .find(object.id)?;
    let evaluated = evaluate_instance(&instance, &item_id.into()).ok()?;
    let design = evaluated.design();
    let derived = evaluated.derived();
    adventuresim_core::equipment::ParametricWeaponCombatGeometry::new(
        derived.mass_kg,
        derived.length_m,
        derived.grip_to_tip_m,
        derived.striking_head_length_m,
        derived.moment_of_inertia_kg_m2,
        derived.balance,
        adventuresim_core::combat::EMBEDDED_COMBAT_RESOLUTION_PARAMETERS
            .contact
            .precision_for_design(design)
            .value(),
    )
}

/// Unfitted holders use their catalog construction; fitted instances use their
/// authenticated material shells and must never silently fall back on corruption.
pub(crate) fn fitted_holder_mass(
    ctx: &ReducerContext,
    inventory_row_id: adventuresim_core::identity::InventoryItemId,
    item_id: &adventuresim_core::item_catalog::ItemDefinitionId,
) -> Result<Option<f32>, WeaponInstanceError> {
    let Some(object) = crate::inventory_container::object_for_row(
        ctx,
        CarriedInventoryScope::Personal,
        inventory_row_id,
    )?
    else {
        return Ok(None);
    };
    let Some(instance) = ctx
        .db
        .weapon_holder_instance()
        .physical_object_id()
        .find(object.id)
    else {
        return Ok(None);
    };
    if object.item_id != item_id.as_str() {
        return Err(WeaponInstanceError::HolderIdentityMismatch {
            expected: item_id.clone(),
            actual: object.item_id.into(),
        });
    }
    let evaluated = evaluate_holder_instance(&instance, item_id)?;
    Ok(Some(evaluated.derived().mass_kg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_recipe_retains_its_codec_cause_through_inventory_issuance() {
        use std::error::Error;
        let mut design = default_design("longsword").unwrap();
        design.recipe.components.clear();
        let projection = WeaponInstance::from_design(42, &design).unwrap_err();
        assert!(matches!(&projection, WeaponProjectionError::Evaluation(
            adventuresim_weapon_model::CodecError::InvalidDesign(errors)
        ) if !errors.is_empty()));
        let grant = crate::item::InventoryGrantError::from(WeaponInstanceError::from(projection));
        let instance = grant
            .source()
            .unwrap()
            .downcast_ref::<WeaponInstanceError>()
            .unwrap();
        let projection = instance
            .source()
            .unwrap()
            .downcast_ref::<WeaponProjectionError>()
            .unwrap();
        assert!(
            matches!(projection.source().unwrap().downcast_ref::<adventuresim_weapon_model::CodecError>(),
            Some(adventuresim_weapon_model::CodecError::InvalidDesign(errors)) if !errors.is_empty())
        );
    }

    #[test]
    fn persisted_scaling_preserves_rounding_and_reports_the_failed_field() {
        assert_eq!(
            checked_scaled_u32(1.2345, 1_000.0, WeaponProjectionField::Mass).unwrap(),
            1235
        );
        assert_eq!(
            checked_scaled_u32(-0.0, 1_000.0, WeaponProjectionField::Mass).unwrap(),
            0
        );
        assert_eq!(
            checked_scaled_u32(u32::MAX as f32, 1.0, WeaponProjectionField::Length).unwrap(),
            u32::MAX
        );
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, f32::MAX] {
            let error = checked_scaled_u32(value, 1_000.0, WeaponProjectionField::HolderGripToTip)
                .unwrap_err();
            assert!(matches!(
                error,
                WeaponProjectionError::OutOfRange(WeaponProjectionField::HolderGripToTip)
            ));
        }
    }

    #[test]
    fn persisted_projection_round_trips_and_rejects_tampering() {
        let design = default_design("longsword").expect("longsword recipe");
        let mut instance = WeaponInstance::from_design(42, &design).expect("instance");
        assert!(evaluate_instance(&instance, &"longsword".into()).is_ok());
        instance.recipe[0] ^= 0x55;
        assert!(evaluate_instance(&instance, &"longsword".into()).is_err());
    }

    #[test]
    fn same_catalog_weapon_rows_can_retain_distinct_recipes() {
        let first = default_design("longsword").expect("longsword recipe");
        let mut second = first.clone();
        let blade = second
            .recipe
            .components
            .iter_mut()
            .find_map(|component| match &mut component.shape {
                adventuresim_weapon_model::recipe::Shape::LoftedBlade(blade) => Some(blade),
                _ => None,
            })
            .expect("longsword should have a section blade");
        blade.length =
            adventuresim_weapon_model::recipe::Metres::new(blade.length.get() + 0.025).unwrap();
        let first = WeaponInstance::from_design(10, &first).unwrap();
        let second = WeaponInstance::from_design(11, &second).unwrap();
        assert_ne!(first.physical_object_id, second.physical_object_id);
        assert_ne!(first.design_hash, second.design_hash);
        assert_ne!(first.recipe, second.recipe);
        assert_ne!(first.mass_grams, second.mass_grams);
        assert_ne!(first.length_mm, second.length_mm);
        assert_ne!(first.grip_to_tip_mm, second.grip_to_tip_mm);
    }

    #[test]
    fn same_holder_template_can_retain_distinct_per_object_recipes() {
        let weapon = default_design("longsword").unwrap();
        let first = default_holder_design(&weapon).unwrap();
        let mut second = first.clone();
        second.clearance.0 += 2;
        second.chape_length.0 += 6;
        let first = WeaponHolderInstance::from_design(20, &first).unwrap();
        let second = WeaponHolderInstance::from_design(21, &second).unwrap();
        assert_ne!(first.physical_object_id, second.physical_object_id);
        assert_ne!(first.design_hash, second.design_hash);
        assert_ne!(first.recipe, second.recipe);
        assert_ne!(first.mass_grams, second.mass_grams);
    }
}
