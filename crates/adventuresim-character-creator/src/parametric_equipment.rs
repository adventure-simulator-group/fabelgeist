//! Fit recipe meshes on the device and assemble them into skinned armor with
//! stable morph correspondence.

use super::*;
use adventuresim_armor_model::{ArmorMorph, HelmetDesign, LimbArmorDesign};
use adventuresim_character_creator::{
    armor_frames::Side,
    armor_recipes::{self, ParametricDesign},
    device_frames::DeviceWearer,
    device_piece::DeviceRecording,
    inventory::{FittedPiece, Loadout},
    item_design::ItemDesign,
};
use fabelgeist_compute::KernelBatch;

pub(super) fn fitted_design(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    if let ParametricDesign::Underlayer(d) = design {
        return crate::underlayer_equipment::fitted(model, generated, d, placement, morphs);
    }
    let piece = crate::device_equipment::fitted(model, generated, morphs, |wearer, batch| {
        record_design(wearer, batch, design, placement)
    })?;
    assembled(model, generated, design, piece, morphs)
}

/// Record a recipe fitted to one realization of the wearer, short of
/// thickening. Underlayers are cut from the body instead.
fn record_design(
    wearer: &DeviceWearer,
    batch: &mut KernelBatch,
    design: &ParametricDesign,
    placement: &str,
) -> Result<DeviceRecording> {
    match design {
        ParametricDesign::Helmet(HelmetDesign::CloseHelmet(d)) => {
            wearer.record_fitted_close_helmet(batch, d)
        }
        ParametricDesign::Helmet(HelmetDesign::MailCoif(d)) => wearer.record_fitted_coif(batch, d),
        ParametricDesign::Helmet(helmet) => wearer.record_helmet(batch, helmet),
        ParametricDesign::Limb(limb) => {
            let region = armor_recipes::fit_region(design, placement)?;
            if matches!(
                limb,
                LimbArmorDesign::MittenGauntlet(_)
                    | LimbArmorDesign::Sabaton(_)
                    | LimbArmorDesign::LeatherBoot(_)
            ) {
                wearer.record_fitted_extremity(batch, limb, region)
            } else {
                wearer.record_fitted_limb(batch, limb, region)
            }
        }
        ParametricDesign::Garment(garment) => {
            wearer.record_fitted_garment(batch, garment, placement)
        }
        ParametricDesign::Underlayer(_) => {
            anyhow::bail!("underlayers are cut from the body, not recorded as parts")
        }
    }
}

/// A close helmet moves with the head alone, whatever skin lies nearest.
fn rigid_helmet(
    model: &BodyModel,
    design: &ParametricDesign,
    armor: &mut GeneratedArmor,
) -> Result<()> {
    if matches!(
        design,
        ParametricDesign::Helmet(HelmetDesign::CloseHelmet(_))
    ) {
        let head = model
            .mhr
            .character
            .skeleton
            .names
            .iter()
            .position(|name| name == "c_head")
            .context("rigid helmet requires c_head joint")? as u32;
        armor.joint_indices.fill([head; 8]);
        armor
            .joint_weights
            .fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }
    Ok(())
}

/// The armor of a piece fitted on the device.
fn assembled(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    piece: crate::device_equipment::DevicePiece,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let crate::device_equipment::DevicePiece {
        base,
        skin,
        endpoints,
    } = piece;
    let mut targets = Vec::with_capacity(endpoints.len());
    for (sample, endpoint) in morphs.iter().zip(endpoints) {
        anyhow::ensure!(
            endpoint.indices == base.indices && endpoint.positions.len() == base.positions.len(),
            "armor fit changed morph topology"
        );
        anyhow::ensure!(
            endpoint
                .components
                .iter()
                .map(|part| (&part.role, &part.vertices, &part.indices))
                .eq(base
                    .components
                    .iter()
                    .map(|part| (&part.role, &part.vertices, &part.indices))),
            "armor fit changed component correspondence"
        );
        targets.push(ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&base.positions, &endpoint.positions),
            normal_deltas: deltas(&base.normals, &endpoint.normals),
            direct_positions: endpoint.positions,
        });
    }
    let bytes = serde_json::to_vec(design)?;
    let mut armor = GeneratedArmor {
        design_hash: adventuresim_armor_model::parametric_design_hash(&bytes),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: base.positions,
        normals: base.normals,
        texcoords: skin.texcoords,
        joint_indices: skin.joint_indices,
        joint_weights: skin.joint_weights,
        indices: base.indices,
        morphs: targets,
        components: base.components,
    };
    rigid_helmet(model, design, &mut armor)?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

/// Each morph endpoint's offset from the base.
pub(super) fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}

/// Fit any parametric item's design to the wearer.
pub(super) fn fitted_item(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ItemDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let _slot = adventuresim_character_creator::fitting_slot();
    match design {
        ItemDesign::Recipe(design) => fitted_design(model, generated, design, placement, morphs),
        ItemDesign::Vambrace(design) => {
            let side = match Side::from_placement(placement)? {
                Side::Left => ForearmSide::Left,
                Side::Right => ForearmSide::Right,
            };
            fitted_bracer(model, generated, design, side, morphs)
        }
        ItemDesign::Breastplate(design) => fitted_breastplate(model, generated, design, morphs),
    }
}

/// Fit a catalog item; plate steel gets the metal's texture density.
pub(super) fn fitted_catalog_item(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    item: &ItemDefinition,
    design: &ItemDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let mut armor = fitted_item(model, generated, design, placement, morphs)?;
    let material = item
        .equipment
        .as_ref()
        .and_then(|equipment| equipment.material);
    if material.is_some_and(adventuresim_character_creator::armor_metal::is_plate_steel) {
        adventuresim_character_creator::armor_metal::scale_to_metal_density(&mut armor);
    }
    Ok(armor)
}

pub(super) struct SelectedArmor<'a> {
    pub piece: FittedPiece<'a>,
    pub name: String,
    pub generated: GeneratedArmor,
}

/// Fit every worn parametric catalog item.
pub(super) fn selected<'a>(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    loadout: &Loadout<'a>,
    morphs: &[ForearmMorphSample],
) -> Result<Vec<SelectedArmor<'a>>> {
    loadout
        .fitted
        .iter()
        .map(|piece| {
            let placement = &piece.piece.placement.id;
            let item_id = &piece.piece.item.id;
            Ok(SelectedArmor {
                name: format!("{item_id}--{placement}"),
                generated: fitted_catalog_item(
                    model,
                    generated,
                    piece.piece.item,
                    &piece.design,
                    placement,
                    morphs,
                )
                .with_context(|| format!("fitting {item_id} ({placement})"))?,
                piece: piece.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_character_creator::item_design::CatalogDesigns;

    /// Every catalog armor placement fits the measured MHR body and its
    /// morph samples on the device, with one topology for all of them.
    #[test]
    #[ignore = "requires MHR_ASSETS and a compute-capable GPU"]
    fn every_catalog_armor_fits_the_measured_body_and_its_morphs() -> Result<()> {
        let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
        let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
        let catalog = ItemCatalog::load(
            std::path::Path::new("../../content/items"),
            CatalogDesigns::authored(),
        )?;
        let recipe = CharacterRecipe::default();
        let body = generate_character(&model, &recipe)?;
        let morphs = character_morphs::CharacterMorphs::generate(&model, &recipe, &body)?.samples;
        let mut fitted = 0;
        for item in catalog.wearable() {
            let Some(design) = catalog.design(&item.id) else {
                continue;
            };
            for placement in &item.equipment.as_ref().expect("wearable item").placements {
                let armor =
                    fitted_catalog_item(&model, &body, item, &design, &placement.id, &morphs)
                        .with_context(|| format!("fitting {}--{}", item.id, placement.id))?;
                let count = armor.positions.len();
                anyhow::ensure!(count > 0 && armor.indices.len().is_multiple_of(3));
                anyhow::ensure!(armor.indices.iter().all(|i| (*i as usize) < count));
                anyhow::ensure!(armor.normals.len() == count && armor.texcoords.len() == count);
                anyhow::ensure!(
                    armor
                        .positions
                        .iter()
                        .chain(&armor.normals)
                        .flatten()
                        .all(|v| v.is_finite()),
                    "{}: non-finite geometry",
                    item.id
                );
                anyhow::ensure!(armor.morphs.len() == morphs.len());
                for morph in &armor.morphs {
                    anyhow::ensure!(morph.direct_positions.len() == count);
                }
                fitted += 1;
            }
        }
        anyhow::ensure!(fitted > 0, "the catalog has no parametric armor");
        Ok(())
    }
}
