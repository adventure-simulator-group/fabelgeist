//! Reproducible body-visible candidate export for independent mesh review.
use super::*;
use adventuresim_character_creator::{armor_frames::Wearer, armor_recipes};

pub(super) fn export(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    breastplate_design: &BreastplateDesign,
) -> Result<()> {
    std::fs::create_dir_all(output)?;
    let body = generate_character(model, recipe)?;
    let character = &model.mhr.character;
    std::fs::write(
        output.join("body.json"),
        serde_json::to_vec(&serde_json::json!({
            "positions":body.positions,"normals":body.normals,"faces":character.mesh.faces,"recipe":recipe,
            "joint_names":character.skeleton.names,"joints":body.global_joint_states,
            "joint_indices":character.skin_weights.index,"joint_weights":character.skin_weights.weight,
            "generator_version":adventuresim_armor_model::GENERATOR_VERSION
        }))?,
    )?;
    let wearer = Wearer {
        positions: &body.positions,
        normals: &body.normals,
        joints: &body.global_joint_states,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    for item in procedural_items(catalog) {
        let Some(design) = catalog.design(&item.id) else {
            existing(output, model, &body, item, breastplate_design)?;
            continue;
        };
        for placement in &item.equipment.as_ref().expect("procedural item").placements {
            let frame = wearer.frame(armor_recipes::fit_region(&design, &placement.id)?)?;
            let mesh = armor_recipes::fitted_mesh(&design, &placement.id, &wearer)
                .with_context(|| format!("review generation {}/{}", item.id, placement.id))?;
            std::fs::write(
                output.join(format!("{}--{}.json", item.id, placement.id)),
                serde_json::to_vec(&serde_json::json!({
                    "id":item.id,"placement":placement.id,"design":design,"positions":mesh.positions,"normals":mesh.normals()?,"indices":mesh.indices,
                    "frame":{"origin":frame.origin,"axes":frame.axes,"half_extents":frame.half_extents},
                    "generator_version":adventuresim_armor_model::GENERATOR_VERSION
                }))?,
            )?;
        }
    }
    Ok(())
}

fn existing(
    output: &std::path::Path,
    model: &BodyModel,
    body: &GeneratedCharacter,
    item: &ItemDefinition,
    breastplate_design: &BreastplateDesign,
) -> Result<()> {
    for placement in &item.equipment.as_ref().expect("procedural item").placements {
        let (armor, design) = match item.id.as_str() {
            "vambrace" => {
                let side = match adventuresim_character_creator::armor_frames::Side::from_placement(
                    &placement.id,
                )? {
                    adventuresim_character_creator::armor_frames::Side::Left => ForearmSide::Left,
                    adventuresim_character_creator::armor_frames::Side::Right => ForearmSide::Right,
                };
                let design = BracerDesign::default();
                (
                    fitted_bracer(model, body, &design, side, &[])?,
                    serde_json::to_value(design)?,
                )
            }
            "breastplate" | "cuirass" => (
                fitted_breastplate(model, body, breastplate_design, &[])?,
                serde_json::to_value(breastplate_design)?,
            ),
            _ => continue,
        };
        std::fs::write(
            output.join(format!("{}--{}.json", item.id, placement.id)),
            serde_json::to_vec(&serde_json::json!({
                "id":item.id,"placement":placement.id,"design":design,"positions":armor.positions,"normals":armor.normals,"indices":armor.indices,
                "generator_version":adventuresim_armor_model::GENERATOR_VERSION
            }))?,
        )?;
    }
    Ok(())
}
