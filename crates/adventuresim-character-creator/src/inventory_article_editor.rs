//! Shape, fabric and drape controls for the selected inventory article.
use super::*;
use adventuresim_character_creator::inventory::CatalogArticle;

pub(super) fn show(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &mut EquipmentCatalog,
    drape_job: &mut DrapeJob,
) {
    let Studio {
        recipe,
        status,
        dirty,
        inventory: view,
        decorations,
        ..
    } = studio;
    let Some(item) = view.selected.and_then(|id| recipe.inventory.get_mut(id)) else {
        view.selected = None;
        ui.weak("Select an item to edit it.");
        return;
    };
    ui.heading(item.article.name(catalog));
    let rebuild = match &mut item.article {
        Article::Catalog(article) => {
            catalog_article(ui, item.id, article, catalog, &decorations.library, status)
        }
        Article::Draped(selection) => draped(ui, item.id, selection, drape_job),
        Article::Settled(garment) => settled(ui, item.id, garment),
    };
    if !item.worn {
        ui.small("Carried items are not shown on the body.");
    }
    *dirty |= rebuild && item.worn;
}

/// Returns whether the article's appearance on the body changed.
fn catalog_article(
    ui: &mut egui::Ui,
    id: InventoryItemId,
    article: &mut CatalogArticle,
    catalog: &mut EquipmentCatalog,
    library: &adventuresim_character_creator::decoration::DecorationLibrary,
    status: &mut String,
) -> bool {
    let mut steel = false;
    if let Some((item, placement)) = catalog.placement(&article.item_id, &article.placement_id) {
        let material = item
            .equipment
            .as_ref()
            .and_then(|equipment| equipment.material);
        steel = material.is_some_and(adventuresim_character_creator::armor_metal::is_plate_steel);
        let material = material.map_or_else(String::new, |material| format!(" · {material:?}"));
        ui.small(format!(
            "{} · {} placement · {:.2} kg{material}",
            item.id, placement.id, item.weight_kg
        ));
    }
    let mut changed = catalog_shape(ui, article, catalog, status);
    if steel {
        ui.horizontal(|ui| {
            ui.label("Decoration");
            changed |= decoration_controls::choose(
                ui,
                ("article_decoration", id),
                library,
                &mut article.decoration,
            )
            .is_some();
        })
        .response
        .on_hover_text("Choose an engraving and trim saved from the armory.");
        changed |= decoration_controls::edit(ui, &mut article.decoration);
        changed |= construction_controls::edit(ui, &mut article.construction);
    }
    changed
}

/// Returns whether the article's generated shape changed.
fn catalog_shape(
    ui: &mut egui::Ui,
    article: &mut CatalogArticle,
    catalog: &mut EquipmentCatalog,
    status: &mut String,
) -> bool {
    let mut design = match article.design(catalog) {
        Ok(Some(design)) => design,
        Ok(None) => {
            ui.weak("Generated from its catalog surface coverage; it has no shape controls.");
            return false;
        }
        Err(conflict) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                conflict.describe(|id| id.to_string()),
            );
            return false;
        }
    };
    let own = article.design.is_some();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(if own { "Own shape" } else { "Catalog shape" });
        if ui
            .add_enabled(own, egui::Button::new("Use catalog shape"))
            .clicked()
        {
            article.design = None;
            changed = true;
        }
        if ui
            .add_enabled(own, egui::Button::new("Make catalog default"))
            .on_hover_text("Newly acquired items and equipment asset generation use it.")
            .clicked()
        {
            *status = match catalog
                .designs
                .set_default(&article.item_id, design.clone())
            {
                Ok(()) => format!(
                    "{} is now the catalog default; save it from the Output tab",
                    article.item_id
                ),
                Err(error) => format!("Could not change the catalog default: {error:#}"),
            };
        }
    });
    if !changed && armor_controls::design(ui, &mut design) {
        article.design = Some(design);
        changed = true;
    }
    changed
}

/// Returns whether the outfit must be fitted again. A settled garment keeps
/// its drape, so only its name, layer and appearance are edited here.
fn settled(
    ui: &mut egui::Ui,
    id: InventoryItemId,
    garment: &mut adventuresim_character_creator::garment::SettledGarment,
) -> bool {
    let selection = &mut garment.selection;
    ui.small(format!(
        "{} · {} vertices · fitted from its saved drape without simulating",
        selection.fabric.label(),
        garment.drape.vertex_count()
    ));
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut selection.name);
    });
    let layer = selection.layer;
    garment_controls::layer(ui, egui::Id::new(("article", id)), selection);
    if selection.fabric == FabricPreset::Chainmail {
        fabric_controls::mail_weave(ui, &mut selection.mail);
    }
    ui.weak("Reshape it in the wardrobe and save it again to change its cut or fabric.");
    // The layer sets which garments drape over which.
    selection.layer != layer
}

/// Returns whether the garment must be draped again; appearance edits apply live.
fn draped(
    ui: &mut egui::Ui,
    id: InventoryItemId,
    selection: &mut GarmentSelection,
    drape_job: &mut DrapeJob,
) -> bool {
    let before = selection.clone();
    let id = egui::Id::new(("article", id));
    garment_controls::show(ui, id, selection);
    ui.separator();
    fabric_controls::show(ui, id, selection);
    garment_controls::layer(ui, id, selection);
    ui.separator();
    drape_controls::show(ui, selection);
    if ui.button("Drape again").clicked() {
        drape_job.restart_from_placement();
        return true;
    }
    // The layer sets which garments drape over which.
    !before.same_simulation(selection) || before.layer != selection.layer
}
