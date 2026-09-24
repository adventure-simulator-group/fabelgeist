//! Reproducible body-visible candidate export for independent mesh review.
use super::*;
use adventuresim_character_creator::{armor_recipes, item_design::ItemDesign};

pub(super) fn export(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
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
            "generator_version":fabelgeist_armor::GENERATOR_VERSION
        }))?,
    )?;
    for item in catalog.wearable() {
        let Some(design) = catalog.design(&item.id) else {
            continue;
        };
        for placement in &item.equipment.as_ref().expect("wearable item").placements {
            let record = review_record(model, &body, item, &placement.id, &design)
                .with_context(|| format!("review generation {}/{}", item.id, placement.id))?;
            std::fs::write(
                output.join(format!("{}--{}.json", item.id, placement.id)),
                serde_json::to_vec(&record)?,
            )?;
        }
    }
    Ok(())
}

/// One item's review record: its design, the fitted mesh and, for a recipe,
/// the part frame it was fitted in.
fn review_record(
    model: &BodyModel,
    body: &GeneratedCharacter,
    item: &ItemDefinition,
    placement: &str,
    design: &ItemDesign,
) -> Result<serde_json::Value> {
    let armor = parametric_equipment::fitted_item(model, body, design, placement, &[])?;
    let mut record = serde_json::json!({
        "id":item.id,"placement":placement,"positions":armor.positions,"normals":armor.normals,
        "indices":armor.indices,"components":armor.components,
        "generator_version":fabelgeist_armor::GENERATOR_VERSION
    });
    record["design"] = match design {
        ItemDesign::Recipe(recipe) => {
            let region = armor_recipes::fit_region(recipe, placement)?;
            let frame = device_equipment::frame(model, body, region)?;
            record["frame"] = serde_json::json!({
                "origin":frame.origin,"axes":frame.axes,"half_extents":frame.half_extents
            });
            serde_json::to_value(recipe)?
        }
        ItemDesign::Vambrace(vambrace) => serde_json::to_value(vambrace)?,
        ItemDesign::Breastplate(breastplate) => serde_json::to_value(breastplate)?,
    };
    Ok(record)
}
