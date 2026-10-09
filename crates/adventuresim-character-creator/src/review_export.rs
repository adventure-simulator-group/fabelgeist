//! Reproducible body-visible candidate export for independent mesh review.
use super::*;
use adventuresim_character_creator::{
    armor_recipes,
    inventory::{Article, CatalogArticle},
    item_design::ItemDesign,
};

/// What the review exports besides the body.
#[derive(Clone, Copy, clap::ValueEnum)]
pub(super) enum ReviewSelection {
    /// Every placement of every parametric catalog item.
    Catalog,
    /// Only the catalog articles the recipe wears.
    Recipe,
    /// Only the body.
    Body,
}

pub(super) fn export(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    selection: ReviewSelection,
) -> Result<()> {
    std::fs::create_dir_all(output)?;
    let body = crate::profiling::measure("body_generation", || generate_character(model, recipe))?;
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
    let pieces = match selection {
        ReviewSelection::Body => return Ok(()),
        ReviewSelection::Catalog => catalog_pieces(catalog),
        ReviewSelection::Recipe => worn_pieces(recipe, catalog)?,
    };
    for (article, design) in pieces {
        let (item, placement) = (&article.item_id, &article.placement_id);
        eprintln!("Review mesh: {item} ({placement})");
        let item = catalog
            .item(item)
            .with_context(|| format!("unknown review item {item}"))?;
        let record = review_record(model, &body, item, placement, &design)
            .with_context(|| format!("review generation {}/{placement}", item.id))?;
        std::fs::write(
            output.join(format!("{}--{placement}.json", item.id)),
            serde_json::to_vec(&record)?,
        )?;
    }
    Ok(())
}

/// Every placement of every parametric catalog item, in its placement's design.
fn catalog_pieces(catalog: &EquipmentCatalog) -> Vec<(CatalogArticle, ItemDesign)> {
    catalog
        .wearable()
        .flat_map(|item| {
            let placements = &item.equipment.as_ref().expect("wearable item").placements;
            placements.iter().filter_map(|placement| {
                let design = catalog.placed_design(&item.id, &placement.id)?;
                Some((CatalogArticle::new(&item.id, &placement.id), design))
            })
        })
        .collect()
}

/// The parametric catalog articles the recipe wears, in their own designs.
fn worn_pieces(
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
) -> Result<Vec<(CatalogArticle, ItemDesign)>> {
    let mut pieces = Vec::new();
    for worn in recipe.inventory.worn() {
        let Article::Catalog(article) = &worn.article else {
            continue;
        };
        let design = article
            .design(catalog)
            .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
        if let Some(design) = design {
            pieces.push((article.clone(), design));
        }
    }
    Ok(pieces)
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
    let armor = parametric_equipment::fitted_item(model, body, design, placement, &[], &[])?;
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
