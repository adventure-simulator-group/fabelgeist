use super::character_morphs::{
    CharacterMorphs, MorphDelta, armor_targets, rigged_armor, rigged_clothing,
};
use super::*;

pub(super) fn export_character(
    path: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
) -> Result<()> {
    let generated = generate_character(model, recipe)?;
    let morphs = CharacterMorphs::generate(model, recipe, &generated)?;
    let body_targets = morphs
        .body
        .iter()
        .map(MorphDelta::rigged)
        .collect::<Vec<_>>();
    let character = &model.mhr.character;
    let specifications = selected_garments(recipe, catalog).map_err(anyhow::Error::msg)?;
    let clothed = generate_clothing_shells(
        &specifications,
        &generated.positions,
        &generated.normals,
        &character.mesh.faces,
        &character.skin_weights.index,
        &character.skin_weights.weight,
        &character.skeleton.names,
        &generated.global_joint_states,
    )
    .map_err(anyhow::Error::msg)?;
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
    for (i, piece) in armor.iter().enumerate() {
        let mut parts = rigged_armor(
            &piece.name,
            &piece.generated,
            &armor_faces[i],
            &armor_targets[i],
        );
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
        crate::character_morphs::component_materials(&piece.generated, &mut parts);
        shells.extend(parts);
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
