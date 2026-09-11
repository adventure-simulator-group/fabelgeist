//! Reproducible body-visible candidate export for independent mesh review.
use super::*;

pub(super) fn export(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
) -> Result<()> {
    std::fs::create_dir_all(output)?;
    let body = generate_character(model, recipe)?;
    let character = &model.mhr.character;
    std::fs::write(
        output.join("body.json"),
        serde_json::to_vec(&serde_json::json!({
            "positions":body.positions,"normals":body.normals,"faces":character.mesh.faces,"recipe":recipe,
            "texcoords":character.mesh.texcoords,"texcoord_faces":character.mesh.texcoord_faces,
            "joint_names":character.skeleton.names,"joints":body.global_joint_states,
            "joint_indices":character.skin_weights.index,"joint_weights":character.skin_weights.weight,
            "generator_version":adventuresim_armor_model::GENERATOR_VERSION
        }))?,
    )?;
    let mut review_recipe = recipe.clone();
    review_recipe.clothing = procedural_items(catalog)
        .filter(|item| adventuresim_character_creator::armor_recipes::is_parametric(&item.id))
        .flat_map(|item| {
            item.equipment
                .as_ref()
                .expect("procedural item")
                .placements
                .iter()
                .map(|p| ClothingSelection {
                    item_id: item.id.clone(),
                    placement_id: p.id.clone(),
                })
        })
        .collect();
    let pieces = parametric_equipment::selected(
        model,
        &body,
        &review_recipe,
        catalog,
        bracer_design,
        breastplate_design,
        &[],
    )?;
    for (selection, piece) in review_recipe.clothing.iter().zip(pieces) {
        let armor = piece.generated;
        let design = if let Some(design) = catalog.design(&selection.item_id) {
            serde_json::to_value(design)?
        } else if selection.item_id == "vambrace" {
            serde_json::to_value(bracer_design)?
        } else {
            serde_json::to_value(breastplate_design)?
        };
        std::fs::write(
            output.join(format!("{}.json", piece.name)),
            serde_json::to_vec(&serde_json::json!({
                "id": selection.item_id, "placement": selection.placement_id,
                "design": design, "fasteners": catalog.2.get(&selection.item_id),
                "positions": armor.positions, "normals": armor.normals, "indices": armor.indices,
                "components": armor.components,
                "generator_version": adventuresim_armor_model::GENERATOR_VERSION,
            }))?,
        )?;
    }
    Ok(())
}
