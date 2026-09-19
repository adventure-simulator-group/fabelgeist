//! Base-pose review GLBs share runtime attributes and the ordinary rig exporter.
use super::*;

pub(super) fn write(
    path: &std::path::Path,
    piece: &parametric_equipment::SelectedArmor,
    model: &BodyModel,
    body: &GeneratedCharacter,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
) -> Result<()> {
    let character = &model.mhr.character;
    let faces = piece.generated.indices.as_chunks::<3>().0;
    let mut shells = character_morphs::rigged_armor(&piece.name, &piece.generated, faces, &[]);
    let item = catalog
        .0
        .iter()
        .find(|item| item.id == piece.item_id)
        .context("review item missing from catalog")?;
    let material = item
        .equipment
        .as_ref()
        .and_then(|equipment| equipment.material)
        .context("review equipment material missing")?;
    let (color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
    for shell in &mut shells {
        shell.base_color = color;
        shell.metallic = metallic;
        shell.roughness = roughness;
        if let Some(textures) = adventuresim_character_creator::underlayer_material::textures(
            catalog.design(&piece.item_id, &piece.placement_id).as_ref(),
        ) {
            shell.textures = Some(textures);
        }
    }
    character_morphs::component_materials(&piece.generated, &mut shells);
    export_rigged_glb(
        GlbOutput::SharedTextures(path),
        &piece.name,
        recipe.version,
        model.lod,
        &RiggedMesh {
            joint_proportions: &body.joint_proportions,
            morph_targets: &[],
            positions: &body.positions,
            normals: &body.normals,
            faces: &character.mesh.faces,
            export_body: false,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            joint_parents: &character.skeleton.parents,
            global_joint_states: &body.global_joint_states,
        },
        &shells,
        &[],
    )
}
