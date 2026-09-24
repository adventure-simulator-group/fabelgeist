//! Fit recipe meshes on the device and assemble them into skinned armor with
//! stable morph correspondence.

use super::*;
use adventuresim_character_creator::{
    armor_frames::Side,
    armor_recipes::{self, ParametricDesign},
    device_frames::DeviceWearer,
    device_piece::DeviceRecording,
    inventory::{FittedPiece, Loadout},
    item_design::ItemDesign,
};
use fabelgeist_armor::{ArmorMorph, HelmetDesign, LimbArmorDesign};
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
        design_hash: fabelgeist_armor::parametric_design_hash(&bytes),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: base.positions,
        normals: base.normals,
        texcoords: skin.texcoords,
        joint_indices: skin.joint_indices,
        joint_weights: skin.joint_weights,
        indices: base.indices,
        faces: base.faces,
        trim: None,
        grids: base.grids,
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
    pub trim: Option<SelectedTrim>,
    /// The cord lacing its small plates, when it is built of them.
    pub lacing: Option<SelectedLacing>,
}

/// A worn piece built, trimmed and laced, short of the piece it came from.
#[derive(Clone)]
pub(super) struct Finish {
    pub generated: GeneratedArmor,
    pub trim: Option<SelectedTrim>,
    pub lacing: Option<SelectedLacing>,
}

impl<'a> SelectedArmor<'a> {
    pub(super) fn new(piece: &FittedPiece<'a>, finish: Finish) -> Self {
        let Finish {
            generated,
            trim,
            lacing,
        } = finish;
        Self {
            name: piece_name(piece),
            generated,
            trim,
            lacing,
            piece: piece.clone(),
        }
    }
}

/// The band along a worn piece's edges, and how it is shaded.
#[derive(Clone)]
pub(super) struct SelectedTrim {
    pub metal: fabelgeist_armor::material::Metal,
    /// Per vertex, at the metal's texture density along each edge.
    pub texcoords: Vec<[f32; 2]>,
    pub name: String,
}

/// The cord lacing a worn piece's small plates, and how it is shaded.
#[derive(Clone)]
pub(super) struct SelectedLacing {
    pub generated: GeneratedArmor,
    pub cord: fabelgeist_armor::Lacing,
    pub name: String,
}

/// Whether a worn piece is plate steel, which can be built of small plates,
/// engraved and trimmed.
fn is_steel(piece: &FittedPiece<'_>) -> bool {
    piece
        .piece
        .item
        .equipment
        .as_ref()
        .and_then(|equipment| equipment.material)
        .is_some_and(adventuresim_character_creator::armor_metal::is_plate_steel)
}

/// Build a worn plate-steel piece in its construction.
fn construct(
    piece: &FittedPiece<'_>,
    name: &str,
    fitted: GeneratedArmor,
) -> Result<(GeneratedArmor, Option<SelectedLacing>)> {
    let construction = if is_steel(piece) {
        &piece.construction
    } else {
        &fabelgeist_armor::Construction::Solid
    };
    let constructed = fitted.constructed(construction)?;
    let lacing = constructed
        .lacing
        .zip(
            construction
                .tiling()
                .and_then(|tiling| tiling.lacing.clone()),
        )
        .map(|(generated, cord)| SelectedLacing {
            generated,
            cord,
            name: format!("{name}.lacing"),
        });
    Ok((constructed.plates, lacing))
}

/// Cut a worn plate-steel piece's trim along its edges.
fn trim(
    piece: &FittedPiece<'_>,
    name: &str,
    generated: GeneratedArmor,
) -> Result<(GeneratedArmor, Option<SelectedTrim>)> {
    let Some(trim) = piece.decoration.trim.as_ref().filter(|_| is_steel(piece)) else {
        return Ok((generated, None));
    };
    let trim = piece.construction.trim_on(trim);
    let (generated, texcoords) =
        adventuresim_character_creator::armor_metal::trimmed(generated, &trim)?;
    Ok((
        generated,
        Some(SelectedTrim {
            metal: trim.metal.clone(),
            texcoords,
            name: format!("{name}.trim"),
        }),
    ))
}

/// The name a worn piece's meshes go by.
pub(super) fn piece_name(piece: &FittedPiece<'_>) -> String {
    format!("{}--{}", piece.piece.item.id, piece.piece.placement.id)
}

/// Whether a worn piece is rigid plate, which garments under it lie beneath.
pub(super) fn is_rigid(piece: &FittedPiece<'_>) -> bool {
    piece.piece.placement.outermost_channel()
        == Some(adventuresim_character_creator::item_catalog_schema::EquipmentChannel::RigidArmor)
}

/// Fit a worn piece to the wearer and its morph samples.
pub(super) fn fit(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    piece: &FittedPiece<'_>,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let placement = &piece.piece.placement.id;
    fitted_catalog_item(
        model,
        generated,
        piece.piece.item,
        &piece.design,
        placement,
        morphs,
    )
    .with_context(|| format!("fitting {} ({placement})", piece.piece.item.id))
}

/// Build, lace and trim a fitted piece as its article asks.
pub(super) fn finish(piece: &FittedPiece<'_>, fitted: GeneratedArmor) -> Result<Finish> {
    let name = piece_name(piece);
    let (plates, lacing) =
        construct(piece, &name, fitted).with_context(|| format!("building {name}"))?;
    let (generated, trim) =
        trim(piece, &name, plates).with_context(|| format!("trimming {name}"))?;
    Ok(Finish {
        generated,
        trim,
        lacing,
    })
}

/// Fit and finish every worn parametric catalog item.
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
            let fitted = fit(model, generated, piece, morphs)?;
            Ok(SelectedArmor::new(piece, finish(piece, fitted)?))
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
        let mut tiled = 0;
        for item in catalog.wearable() {
            let Some(design) = catalog.design(&item.id) else {
                continue;
            };
            for placement in &item.equipment.as_ref().expect("wearable item").placements {
                eprintln!("{}--{}", item.id, placement.id);
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
                let steel = item
                    .equipment
                    .as_ref()
                    .and_then(|equipment| equipment.material)
                    .is_some_and(adventuresim_character_creator::armor_metal::is_plate_steel);
                if steel {
                    trims_along_every_edge(&armor)
                        .with_context(|| format!("trimming {}--{}", item.id, placement.id))?;
                    tiled += builds_of_small_plates(&armor)
                        .with_context(|| format!("tiling {}--{}", item.id, placement.id))?;
                }
                fitted += 1;
            }
        }
        anyhow::ensure!(fitted > 0, "the catalog has no parametric armor");
        anyhow::ensure!(tiled > 0, "no steel piece takes small plates");
        Ok(())
    }

    /// Scales and lamellar lames laced over a fitted steel piece keep one
    /// topology across its morphs, and trim like any plate. A piece no grid
    /// describes, such as a helmet, cannot take them. Returns how many
    /// constructions were built.
    fn builds_of_small_plates(armor: &GeneratedArmor) -> Result<usize> {
        use fabelgeist_armor::{Construction, ConstructionError, Tiling};
        let mut built = 0;
        for construction in [
            Construction::Scale(Tiling::scale()),
            Construction::Lamellar(Tiling::lamellar()),
        ] {
            let constructed = match armor.clone().constructed(&construction) {
                Err(ConstructionError::NoSurfaceGrid) if armor.grids.is_empty() => continue,
                Err(ConstructionError::NoPlateFits) => {
                    eprintln!("  {}: the plates are too large", construction.name());
                    continue;
                }
                result => result?,
            };
            let lacing = constructed.lacing.context("the default plates are laced")?;
            let reach = construction
                .tiling()
                .map_or(0.0, |tiling| tiling.plate.height);
            let [lo, hi] = [f32::min, f32::max].map(|pick| {
                std::array::from_fn::<f32, 3, _>(|k| {
                    armor
                        .positions
                        .iter()
                        .map(|p| p[k])
                        .fold(armor.positions[0][k], pick)
                })
            });
            for piece in [&constructed.plates, &lacing] {
                anyhow::ensure!(
                    piece
                        .positions
                        .iter()
                        .all(|p| (0..3).all(|k| p[k] > lo[k] - reach && p[k] < hi[k] + reach)),
                    "{}: plates reach a plate's height beyond the piece",
                    construction.name()
                );
                let count = piece.positions.len();
                anyhow::ensure!(count > 0 && piece.indices.iter().all(|i| (*i as usize) < count));
                anyhow::ensure!(piece.positions.iter().flatten().all(|v| v.is_finite()));
                anyhow::ensure!(piece.morphs.len() == armor.morphs.len());
                for morph in &piece.morphs {
                    anyhow::ensure!(morph.direct_positions.len() == count);
                    anyhow::ensure!(
                        morph
                            .position_deltas
                            .iter()
                            .flatten()
                            .all(|v| v.is_finite())
                    );
                }
            }
            eprintln!(
                "  {}: {} plate and {} lacing triangles",
                construction.name(),
                constructed.plates.indices.len() / 3,
                lacing.indices.len() / 3,
            );
            trims_along_every_edge(&constructed.plates)?;
            built += 1;
        }
        Ok(built)
    }

    /// A default trim cuts a band on every surface of a fitted steel piece,
    /// keeping its skin, morphs and components valid.
    fn trims_along_every_edge(armor: &GeneratedArmor) -> Result<()> {
        let trim = fabelgeist_armor::trim::Trim::default();
        let (trimmed, texcoords) =
            adventuresim_character_creator::armor_metal::trimmed(armor.clone(), &trim)?;
        let count = trimmed.positions.len();
        anyhow::ensure!(trimmed.indices.iter().all(|i| (*i as usize) < count));
        anyhow::ensure!(trimmed.faces.len() * 3 == trimmed.indices.len());
        anyhow::ensure!(texcoords.len() == count && trimmed.texcoords.len() == count);
        anyhow::ensure!(texcoords.iter().flatten().all(|v| v.is_finite()));
        for morph in &trimmed.morphs {
            anyhow::ensure!(morph.direct_positions.len() == count);
            anyhow::ensure!(morph.position_deltas.len() == count);
        }
        for weights in &trimmed.joint_weights {
            anyhow::ensure!((weights.iter().sum::<f32>() - 1.0).abs() < 1e-4);
        }
        let surfaces = trimmed.surfaces();
        for surface in &surfaces {
            anyhow::ensure!(!surface.trim.is_empty(), "a surface has no band");
            if let Some(component) = surface.component.map(|i| &trimmed.components[i]) {
                anyhow::ensure!(component.indices == (surface.plate.start..surface.trim.end));
                anyhow::ensure!(
                    trimmed.indices[component.indices.clone()]
                        .iter()
                        .all(|v| component.vertices.contains(&(*v as usize))),
                    "a component uses another's vertices"
                );
            }
        }
        let band: usize = surfaces.iter().map(|s| s.trim.len() / 3).sum();
        eprintln!(
            "  trimmed: {} -> {count} vertices, {band} of {} triangles in the band",
            armor.positions.len(),
            trimmed.indices.len() / 3,
        );
        Ok(())
    }
}
