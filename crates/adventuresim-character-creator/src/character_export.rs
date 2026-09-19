use super::character_morphs::{
    CharacterMorphs, MorphDelta, armor_targets, rigged_armor, rigged_clothing,
};
use super::*;
mod draped;
mod plates;

pub(super) fn export_character(
    path: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    fitted: Option<&[adventuresim_character_creator::garment::DrapedGarment]>,
) -> Result<Vec<String>> {
    let loadout = outfit::loadout(recipe, catalog)?;
    let generated = generate_character(model, recipe)?;
    let (fitted, mut warnings) = draped::prepare(model, &loadout, &generated, fitted);
    warnings.extend(draped::validate(model, &loadout, &generated, &fitted));
    let morphs = CharacterMorphs::generate(model, recipe, &generated)?;
    let body_targets = morphs
        .body
        .iter()
        .map(MorphDelta::rigged)
        .collect::<Vec<_>>();
    let character = &model.mhr.character;
    let clothed = outfit::clothing(model, &loadout, &generated)?;
    let clothing_morphs = morphs.clothing(&clothed.shells)?;
    let clothing_targets = clothing_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let armor = parametric_equipment::selected(model, &generated, &loadout, &morphs.samples)?;
    let armor_faces = armor
        .iter()
        .map(|piece| piece.generated.indices.as_chunks::<3>().0.to_vec())
        .collect::<Vec<_>>();
    let armor_targets = armor
        .iter()
        .map(|piece| armor_targets(&piece.generated))
        .collect::<Vec<_>>();
    let mut shells = clothed
        .shells
        .iter()
        .zip(&clothing_targets)
        .map(|(shell, targets)| rigged_clothing(shell, targets))
        .collect::<Vec<_>>();
    let metals = catalog_metals(&armor, catalog)?;
    shells.extend(catalog_shells(
        &armor,
        &armor_faces,
        &armor_targets,
        &metals,
        catalog,
    )?);
    let draped_morphs = morphs.draped(&generated, &character.mesh.faces, &fitted)?;
    let draped_targets: Vec<_> = draped_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect();
    let mail_surfaces = mail_surfaces(&loadout, &fitted)?;
    for ((garment, targets), mail) in fitted.iter().zip(&draped_targets).zip(&mail_surfaces) {
        shells.push(garment.rigged(targets, mail.as_ref()));
    }
    let plates = loadout
        .plate
        .map(|armor| {
            plates::PlateExport::new(armor, model, &generated.global_joint_states, &body_targets)
        })
        .transpose()?;
    let plate_targets = plates.as_ref().map(|p| p.targets()).unwrap_or_default();
    if let Some(plates) = &plates {
        shells.extend(plates.shells(&plate_targets));
    }
    export_rigged_glb(
        GlbOutput::Standalone(path),
        &recipe.name,
        recipe.version,
        model.lod,
        &RiggedMesh {
            joint_proportions: &generated.joint_proportions,
            morph_targets: &body_targets,
            positions: &generated.positions,
            normals: &generated.normals,
            faces: &clothed.visible_body_faces,
            export_body: true,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            joint_parents: &character.skeleton.parents,
            global_joint_states: &generated.global_joint_states,
        },
        &shells,
        &[],
    )?;
    Ok(warnings)
}

type CatalogMetals = Vec<(
    fabelgeist_armor::material::Metal,
    adventuresim_character_creator::export::ShellTextures,
)>;

/// Maps for each distinct catalog steel worn, engraving included, baked once.
fn catalog_metals(
    armor: &[parametric_equipment::SelectedArmor<'_>],
    catalog: &EquipmentCatalog,
) -> Result<CatalogMetals> {
    let mut metals: CatalogMetals = Vec::new();
    for piece in armor {
        let material = catalog.material(&piece.piece.piece.item.id)?;
        let Some(metal) = adventuresim_character_creator::armor_metal::metal(
            material,
            piece.piece.engraving.as_ref(),
        ) else {
            continue;
        };
        if metals.iter().any(|(known, _)| *known == metal) {
            continue;
        }
        let textures = adventuresim_character_creator::export::ShellTextures::armor(&metal)?;
        metals.push((metal, textures));
    }
    Ok(metals)
}

fn catalog_shells<'a>(
    armor: &'a [parametric_equipment::SelectedArmor<'_>],
    faces: &'a [Vec<[u32; 3]>],
    targets: &'a [Vec<RiggedMorphTarget<'a>>],
    metals: &'a CatalogMetals,
    catalog: &EquipmentCatalog,
) -> Result<Vec<RiggedShell<'a>>> {
    let mut shells = Vec::new();
    for (i, piece) in armor.iter().enumerate() {
        let mut parts = rigged_armor(&piece.name, &piece.generated, &faces[i], &targets[i]);
        let material = catalog.material(&piece.piece.piece.item.id)?;
        let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
        let metal = adventuresim_character_creator::armor_metal::metal(
            material,
            piece.piece.engraving.as_ref(),
        )
        .and_then(|metal| metals.iter().find(|(known, _)| *known == metal))
        .map(|(_, textures)| (piece.generated.texcoords.as_slice(), textures));
        for shell in &mut parts {
            shell.base_color = color;
            shell.metallic = metallic;
            shell.roughness = roughness;
            shell.textures = adventuresim_character_creator::underlayer_material::textures(
                piece.piece.design.recipe(),
            );
            shell.surface = metal;
        }
        shells.extend(parts);
    }
    Ok(shells)
}

/// Chainmail appearance comes from the current recipe, not the drape.
fn mail_surfaces(
    loadout: &adventuresim_character_creator::inventory::Loadout<'_>,
    fitted: &[adventuresim_character_creator::garment::DrapedGarment],
) -> Result<Vec<Option<adventuresim_character_creator::garment_material::MailSurface>>> {
    fitted
        .iter()
        .enumerate()
        .map(|(index, garment)| {
            let selection = loadout
                .draped
                .get(index)
                .context("draped garment has no worn inventory article")?
                .selection;
            (garment.fabric == FabricPreset::Chainmail)
                .then(|| {
                    adventuresim_character_creator::garment_material::MailSurface::new(
                        &selection.mail,
                        &garment.texcoords,
                    )
                })
                .transpose()
        })
        .collect()
}
