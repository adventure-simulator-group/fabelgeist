//! Reproducible body-visible candidate export for independent mesh review.
use super::*;
use adventuresim_character_creator::{armor_recipes, inventory::Article, item_design::ItemDesign};

use adventuresim_core::equipment_assets::CatalogPlacement;
struct ReviewPiece<'a> {
    selection: CatalogPlacement<'a>,
    design: ItemDesign,
}

#[derive(serde::Serialize)]
struct ReviewComponent<'a> {
    role: fabelgeist_armor::ArmorComponentRole,
    vertices: &'a std::ops::Range<usize>,
    indices: &'a std::ops::Range<usize>,
    hinge: Option<fabelgeist_armor::ArmorHinge>,
    mount: Option<&'a fabelgeist_armor::PlateMount>,
    material: Option<adventuresim_character_creator::surface_projection::LinearSurfaceMaterial>,
}

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

impl<'a> TryFrom<&'a fabelgeist_armor::ArmorComponent> for ReviewComponent<'a> {
    type Error = adventuresim_character_creator::surface_projection::SurfaceProjectionError;

    fn try_from(component: &'a fabelgeist_armor::ArmorComponent) -> std::result::Result<Self, Self::Error> {
        let material = component.appearance.material()
            .map(adventuresim_character_creator::surface_projection::LinearSurfaceMaterial::try_from)
            .transpose()?;
        Ok(Self {
            role: component.role,
            vertices: &component.vertices,
            indices: &component.indices,
            hinge: component.hinge,
            mount: component.mount.as_ref(),
            material,
        })
    }
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
            "joint_indices":character.skin_weights.index.native_words(),"joint_weights":character.skin_weights.weight,
            "generator_version":fabelgeist_armor::GENERATOR_VERSION
        }))?,
    )?;
    let pieces = match selection {
        ReviewSelection::Body => return Ok(()),
        ReviewSelection::Catalog => catalog_pieces(catalog),
        ReviewSelection::Recipe => worn_pieces(recipe, catalog)?,
    };
    for piece in pieces {
        let selection = piece.selection;
        let item = selection.item();
        let placement = selection.definition();
        eprintln!("Review mesh: {} ({})", item.id, placement.id);
        let record = review_record(model, &body, selection, &piece.design)
            .with_context(|| format!("review generation {}/{}", item.id, placement.id))?;
        std::fs::write(
            output.join(format!("{}--{}.json", item.id, placement.id)),
            serde_json::to_vec(&record)?,
        )?;
    }
    Ok(())
}

/// Every placement of every parametric catalog item, in its placement's design.
fn catalog_pieces(catalog: &EquipmentCatalog) -> Vec<ReviewPiece<'_>> {
    catalog
        .wearable()
        .flat_map(|item| {
            item.placements().filter_map(|selection| {
                let design = catalog.placed_design(selection)?;
                Some(ReviewPiece { selection, design })
            })
        })
        .collect()
}

/// The parametric catalog articles the recipe wears, in their own designs.
fn worn_pieces<'a>(
    recipe: &CharacterRecipe,
    catalog: &'a EquipmentCatalog,
) -> Result<Vec<ReviewPiece<'a>>> {
    let mut pieces = Vec::new();
    for worn in recipe.inventory.worn() {
        let Article::Catalog(article) = &worn.article else {
            continue;
        };
        let selection = article
            .resolve(catalog)
            .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
        let design = article
            .design_for(&catalog.designs, selection)
            .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
        if let Some(design) = design {
            pieces.push(ReviewPiece { selection, design });
        }
    }
    Ok(pieces)
}

/// One item's review record: its design, the fitted mesh and, for a recipe,
/// the part frame it was fitted in.
fn review_record(
    model: &BodyModel,
    body: &GeneratedCharacter,
    placement: CatalogPlacement<'_>,
    design: &ItemDesign,
) -> Result<serde_json::Value> {
    let armor = parametric_equipment::fitted_item(model, body, design, placement, &[], &[])?;
    let components = armor.components.iter().map(ReviewComponent::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut record = serde_json::json!({
        "id":placement.item().id,"placement":placement.definition().id,"positions":armor.positions,"normals":armor.normals,
        "indices":armor.indices,"components":components,
        "generator_version":fabelgeist_armor::GENERATOR_VERSION
    });
    record["design"] = match design {
        ItemDesign::Recipe(recipe) => {
            let region = armor_recipes::fit_region(recipe, placement.into())?;
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
