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
    let bracers = selected_vambrace_sides(recipe)
        .into_iter()
        .map(|side| fitted_bracer(model, &generated, bracer_design, side, &morphs.samples))
        .collect::<Result<Vec<_>>>()?;
    let bracer_faces = bracers
        .iter()
        .map(|bracer| bracer.indices.as_chunks::<3>().0.to_vec())
        .collect::<Vec<_>>();
    let breastplate = breastplate_selected(recipe)
        .then(|| fitted_breastplate(model, &generated, breastplate_design, &morphs.samples))
        .transpose()?;
    let breastplate_faces = breastplate
        .as_ref()
        .map(|armor| armor.indices.as_chunks::<3>().0.to_vec());
    let clothing_morphs = morphs.clothing(&clothed.shells)?;
    let clothing_targets = clothing_morphs
        .iter()
        .map(|targets| targets.iter().map(MorphDelta::rigged).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let bracer_targets = bracers.iter().map(armor_targets).collect::<Vec<_>>();
    let breastplate_targets = breastplate.as_ref().map(armor_targets);
    let mut shells = clothed
        .shells
        .iter()
        .zip(&clothing_targets)
        .map(|(shell, targets)| rigged_clothing(shell, targets))
        .collect::<Vec<_>>();
    for (index, (bracer, faces)) in bracers.iter().zip(&bracer_faces).enumerate() {
        shells.push(rigged_armor(
            if index == 0 {
                "Parametric vambrace"
            } else {
                "Parametric vambrace pair"
            },
            bracer,
            faces,
            &bracer_targets[index],
        ));
    }
    if let (Some(breastplate), Some(faces), Some(targets)) =
        (&breastplate, &breastplate_faces, &breastplate_targets)
    {
        shells.push(rigged_armor(
            "Parametric front breastplate",
            breastplate,
            faces,
            targets,
        ));
    }
    export_rigged_glb(
        path,
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
