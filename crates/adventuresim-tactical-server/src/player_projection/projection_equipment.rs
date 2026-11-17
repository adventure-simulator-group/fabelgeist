use adventuresim_stdb_client::{ConnectedPlayerItem, EquipmentBodyPart};
use adventuresim_tactical_core::prelude::{
    ArmorItem, ArmorSide, ArmorSlot, WeaponAppearance, WeaponHolderAppearance,
};

use super::inventory_projection::{InventoryAdmissionError, InventoryAdmissionResult};
use super::tactical_covered_parts;

pub(super) fn projected_parametric_weapon(
    item: &ConnectedPlayerItem,
    parameters: adventuresim_core::combat::WeaponContactParameters,
) -> InventoryAdmissionResult<Option<adventuresim_core::equipment::ParametricWeaponCombatGeometry>>
{
    let Some(appearance) = item.weapon_appearance.as_ref() else {
        if adventuresim_weapon_model::default_design(&item.item.id).is_some() {
            return Err(InventoryAdmissionError::InvalidRecipe);
        }
        return Ok(None);
    };
    parametric_combat_geometry(&item.item.id, appearance, parameters)
        .map(Some)
        .ok_or(InventoryAdmissionError::InvalidRecipe)
}

pub(super) fn parametric_combat_geometry(
    item_id: &str,
    appearance: &adventuresim_stdb_client::ConnectedWeaponAppearance,
    parameters: adventuresim_core::combat::WeaponContactParameters,
) -> Option<adventuresim_core::equipment::ParametricWeaponCombatGeometry> {
    use adventuresim_weapon_model::{GENERATOR_VERSION, decode, derive_properties, design_hash};
    if appearance.generator_version != GENERATOR_VERSION {
        return None;
    }
    let design = decode(&appearance.recipe).ok()?;
    if design.catalog_id != item_id || design_hash(&design).0.as_slice() != appearance.design_hash {
        return None;
    }
    let derived = derive_properties(&design).ok()?;
    adventuresim_core::equipment::ParametricWeaponCombatGeometry::new(
        derived.mass_kg,
        derived.length_m,
        derived.grip_to_tip_m,
        derived.striking_head_length_m,
        derived.moment_of_inertia_kg_m2,
        derived.balance,
        parameters.precision_for_design(&design).value(),
    )
}

pub(super) fn projected_weapon_appearance(
    item: &ConnectedPlayerItem,
) -> InventoryAdmissionResult<Option<WeaponAppearance>> {
    item.weapon_appearance
        .as_ref()
        .map(|appearance| {
            Ok(WeaponAppearance {
                generator_version: appearance.generator_version,
                design_hash: appearance
                    .design_hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| InventoryAdmissionError::InvalidRecipe)?,
                recipe: appearance.recipe.clone(),
            })
        })
        .transpose()
}

pub(super) fn projected_holder_appearance(
    item: &ConnectedPlayerItem,
) -> InventoryAdmissionResult<Option<WeaponHolderAppearance>> {
    validate_holder(item)?;
    item.weapon_holder_appearance
        .as_ref()
        .map(|appearance| {
            Ok(WeaponHolderAppearance {
                generator_version: appearance.generator_version,
                design_hash: appearance
                    .design_hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| InventoryAdmissionError::InvalidRecipe)?,
                recipe: appearance.recipe.clone(),
            })
        })
        .transpose()
}

pub(super) fn projected_armor(
    item: &ConnectedPlayerItem,
    selection: Option<&adventuresim_core::equipment_placement::EquippedPlacement>,
) -> InventoryAdmissionResult<Option<ArmorItem>> {
    let Some(part) = item.protected_body_parts.first() else {
        return Ok(None);
    };
    let definition = adventuresim_core::item_catalog::definition(&item.item.id)
        .ok_or(InventoryAdmissionError::InvalidEquipment)?;
    let authored = definition
        .equipment
        .as_ref()
        .ok_or(InventoryAdmissionError::InvalidEquipment)?;
    let placement = authored
        .placements
        .iter()
        .find(|placement| {
            Some(placement.id.as_str())
                == selection
                    .and_then(|selection| selection.authored_id())
                    .map(|id| id.as_str())
        })
        .ok_or(InventoryAdmissionError::InvalidEquipment)?;
    let slot = match part {
        EquipmentBodyPart::LeftArm => ArmorSlot::Arms(Some(ArmorSide::Left)),
        EquipmentBodyPart::RightArm => ArmorSlot::Arms(Some(ArmorSide::Right)),
        EquipmentBodyPart::LeftLeg => ArmorSlot::Legs(Some(ArmorSide::Left)),
        EquipmentBodyPart::RightLeg => ArmorSlot::Legs(Some(ArmorSide::Right)),
        EquipmentBodyPart::Head => ArmorSlot::Head,
        EquipmentBodyPart::Chest => ArmorSlot::Chest,
        EquipmentBodyPart::Stomach => ArmorSlot::Stomach,
    };
    let mut coverage_geometry = [None; 7];
    for protected in &placement.protection {
        let body_part = adventuresim_core::equipment::equipment_body_part(*protected);
        let geometry =
            adventuresim_core::combat::AuthoredArmorCoverage::from_placement(placement, body_part);
        let index = adventuresim_core::autoresolve::body_part_index(body_part);
        coverage_geometry[index] = Some(geometry);
    }
    Ok(Some(ArmorItem {
        material: authored
            .material
            .ok_or(InventoryAdmissionError::InvalidEquipment)?,
        range_of_motion: item.item.range_of_motion,
        coverage: item.item.coverage,
        slot,
        resistance: item.item.resistance,
        padding: item.item.padding,
        flexibility: item.item.flexibility,
        covered_parts: tactical_covered_parts(&item.protected_body_parts),
        coverage_geometry,
        layer_order: placement
            .outermost_channel()
            .map(|channel| channel.order())
            .unwrap_or_default(),
    }))
}

fn validate_holder(item: &ConnectedPlayerItem) -> InventoryAdmissionResult<()> {
    use adventuresim_weapon_model::{
        HOLDER_GENERATOR_VERSION, decode_holder, derive_holder_properties, holder_design_hash,
    };
    let Some(appearance) = item.weapon_holder_appearance.as_ref() else {
        return Ok(());
    };
    if appearance.generator_version != HOLDER_GENERATOR_VERSION {
        return Err(InventoryAdmissionError::InvalidRecipe);
    }
    let design =
        decode_holder(&appearance.recipe).map_err(|_| InventoryAdmissionError::InvalidRecipe)?;
    if design.catalog_id != item.item.id
        || holder_design_hash(&design).0.as_slice() != appearance.design_hash
    {
        return Err(InventoryAdmissionError::InvalidRecipe);
    }
    derive_holder_properties(&design).map_err(|_| InventoryAdmissionError::InvalidRecipe)?;
    Ok(())
}
