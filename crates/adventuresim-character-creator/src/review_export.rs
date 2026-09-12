//! Reproducible body-visible candidate export for independent mesh review.
use super::*;

#[derive(Clone, Copy, clap::ValueEnum)]
pub(super) enum ReviewSelection {
    Catalog,
    Recipe,
    Body,
}

pub(super) fn export(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    selection: ReviewSelection,
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
    if matches!(selection, ReviewSelection::Body) {
        return Ok(());
    }
    let mut review_recipe = recipe.clone();
    if matches!(selection, ReviewSelection::Catalog) {
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
    }
    let pieces = parametric_equipment::selected(
        model,
        &body,
        &review_recipe,
        catalog,
        bracer_design,
        breastplate_design,
        &[],
    )?;
    for piece in pieces {
        let mut document = piece_document(&piece, catalog, bracer_design, breastplate_design)?;
        document["joint_names"] = serde_json::to_value(&character.skeleton.names)?;
        document["joints"] = serde_json::to_value(&body.global_joint_states)?;
        std::fs::write(
            output.join(format!("{}.json", piece.name)),
            serde_json::to_vec(&document)?,
        )?;
    }
    Ok(())
}

/// Generated pieces are authoritative after selection filters ordinary clothes.
fn piece_document(
    piece: &parametric_equipment::SelectedArmor,
    catalog: &EquipmentCatalog,
    bracer: &BracerDesign,
    breastplate: &BreastplateDesign,
) -> Result<serde_json::Value> {
    let design = if let Some(design) = catalog.design(&piece.item_id, &piece.placement_id) {
        serde_json::to_value(design)?
    } else if piece.item_id == "vambrace" {
        serde_json::to_value(bracer)?
    } else {
        serde_json::to_value(breastplate)?
    };
    let armor = &piece.generated;
    Ok(serde_json::json!({
        "id": piece.item_id, "placement": piece.placement_id,
        "design": design, "fasteners": catalog.2.get(&piece.item_id),
        "positions": armor.positions, "normals": armor.normals, "indices": armor.indices,
        "components": armor.components,
        "joint_indices": armor.joint_indices, "joint_weights": armor.joint_weights,
        "generator_version": adventuresim_armor_model::GENERATOR_VERSION,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::{LimbArmorDesign, Milliradians};
    use adventuresim_character_creator::armor_design_input::{ArmorDesigns, ArmorPlacement};
    use adventuresim_character_creator::armor_recipes::{self, ParametricDesign};
    use std::collections::BTreeMap;

    #[test]
    fn filtered_review_piece_keeps_its_placement_design_and_identity() {
        let default = armor_recipes::recipe("pauldron").unwrap();
        let mut left = default.clone();
        let ParametricDesign::Limb(LimbArmorDesign::Pauldron(d)) = &mut left else {
            panic!()
        };
        d.outline.front_return = Milliradians(1800);
        let catalog = EquipmentCatalog(
            vec![],
            ArmorDesigns {
                defaults: BTreeMap::from([("pauldron".into(), default.clone())]),
                placements: BTreeMap::from([(
                    "pauldron".into(),
                    BTreeMap::from([(ArmorPlacement::Left, left.clone())]),
                )]),
            },
            Default::default(),
        );
        // This already-filtered result may follow any number of ordinary
        // clothing selections; its metadata cannot depend on their positions.
        let mut piece = parametric_equipment::SelectedArmor {
            item_id: "pauldron".into(),
            placement_id: "left".into(),
            name: "pauldron--left".into(),
            generated: GeneratedArmor {
                plate_edges: vec![],
                components: vec![],
                design_hash: [0; 32],
                surface_domain: "test".into(),
                positions: vec![[0.1, 0.2, 0.3]],
                normals: vec![],
                texcoords: vec![],
                joint_indices: vec![],
                joint_weights: vec![],
                indices: vec![],
                morphs: vec![],
            },
        };
        let document = piece_document(
            &piece,
            &catalog,
            &BracerDesign::default(),
            &BreastplateDesign::default(),
        )
        .unwrap();
        assert_eq!(document["id"], "pauldron");
        assert_eq!(document["placement"], "left");
        assert_eq!(document["design"], serde_json::to_value(left).unwrap());
        assert_eq!(
            document["positions"],
            serde_json::to_value(&piece.generated.positions).unwrap()
        );
        piece.placement_id = "right".into();
        let document = piece_document(
            &piece,
            &catalog,
            &BracerDesign::default(),
            &BreastplateDesign::default(),
        )
        .unwrap();
        assert_eq!(document["placement"], "right");
        assert_eq!(document["design"], serde_json::to_value(default).unwrap());
    }
}
