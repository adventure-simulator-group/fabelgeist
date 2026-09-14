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
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    fitted: Option<&[adventuresim_character_creator::garment::DrapedGarment]>,
) -> Result<()> {
    let generated = generate_character(model, recipe)?;
    let fitted = draped::prepare(model, recipe, &generated, fitted)?;
    draped::validate(model, recipe, &generated, &fitted)?;
    let morphs = CharacterMorphs::generate(model, recipe, &generated)?;
    let body_targets = morphs
        .body
        .iter()
        .map(MorphDelta::rigged)
        .collect::<Vec<_>>();
    let character = &model.mhr.character;
    let clothed = catalog_clothing(model, recipe, &generated, catalog)?;
    let clothing_morphs = morphs.clothing(&clothed.shells)?;
    let clothing_targets = clothing_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let armor = parametric_equipment::selected(
        model,
        &generated,
        recipe,
        catalog,
        bracer_design,
        breastplate_design,
        &morphs.samples,
    )?;
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
    shells.extend(catalog_shells(
        &armor,
        &armor_faces,
        &armor_targets,
        catalog,
    )?);
    let draped_morphs = morphs.draped(&generated, &character.mesh.faces, &fitted)?;
    let draped_targets: Vec<_> = draped_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect();
    // Chainmail appearance comes from the current recipe, not the drape.
    let mail_surfaces = fitted
        .iter()
        .enumerate()
        .map(|(index, garment)| {
            let selection = recipe
                .garments
                .get(index)
                .context("draped garment has no recipe selection")?;
            (garment.fabric == FabricPreset::Chainmail)
                .then(|| {
                    adventuresim_character_creator::garment_material::MailSurface::new(
                        &selection.mail,
                        &garment.texcoords,
                    )
                })
                .transpose()
        })
        .collect::<Result<Vec<_>>>()?;
    for ((garment, targets), mail) in fitted.iter().zip(&draped_targets).zip(&mail_surfaces) {
        shells.push(garment.rigged(targets, mail.as_ref()));
    }
    let plates = recipe
        .armor
        .as_ref()
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
    )
}

fn catalog_shells<'a>(
    armor: &'a [parametric_equipment::SelectedArmor],
    faces: &'a [Vec<[u32; 3]>],
    targets: &'a [Vec<RiggedMorphTarget<'a>>],
    catalog: &EquipmentCatalog,
) -> Result<Vec<RiggedShell<'a>>> {
    let mut shells = Vec::new();
    for (i, piece) in armor.iter().enumerate() {
        let mut parts = rigged_armor(&piece.name, &piece.generated, &faces[i], &targets[i]);
        let (color, metallic, roughness) =
            adventuresim_character_creator::equipment_pbr(catalog.material(&piece.item_id)?);
        for shell in &mut parts {
            shell.base_color = color;
            shell.metallic = metallic;
            shell.roughness = roughness;
            shell.textures = adventuresim_character_creator::underlayer_material::textures(
                catalog.design(&piece.item_id).as_ref(),
            );
        }
        shells.extend(parts);
    }
    Ok(shells)
}

fn catalog_clothing(
    model: &BodyModel,
    recipe: &CharacterRecipe,
    generated: &GeneratedCharacter,
    catalog: &EquipmentCatalog,
) -> Result<adventuresim_character_creator::clothing::ClothedMesh> {
    let character = &model.mhr.character;
    let specifications = selected_garments(recipe, catalog).map_err(anyhow::Error::msg)?;
    generate_clothing_shells(
        &specifications,
        &generated.positions,
        &generated.normals,
        &character.mesh.faces,
        &character.skin_weights.index,
        &character.skin_weights.weight,
        &character.skeleton.names,
        &generated.global_joint_states,
    )
    .map_err(anyhow::Error::msg)
}
