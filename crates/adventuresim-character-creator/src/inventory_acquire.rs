//! Add catalog items, new cloth and plate armor to the inventory.
use super::*;
use adventuresim_character_creator::inventory::CatalogArticle;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog) {
    ui.small("New items are worn at once, replacing whatever fills their place.");
    ui.horizontal(|ui| {
        ui.label("Search");
        ui.text_edit_singleline(&mut studio.inventory.search);
    });
    let search = studio.inventory.search.trim().to_lowercase();
    let matches = |name: &str| search.is_empty() || name.to_lowercase().contains(&search);

    ui.horizontal(|ui| {
        if ui
            .button("New cloth")
            .on_hover_text("A cotton shirt to reshape: pattern, fabric and layer are all editable.")
            .clicked()
        {
            acquire(
                studio,
                catalog,
                vec![Article::Draped(GarmentSelection::default())],
            );
        }
        if ui.button("New plate armor").clicked() {
            acquire(
                studio,
                catalog,
                vec![Article::Plate(fabelgeist_armor::Armor::default())],
            );
        }
    });

    wardrobe(ui, studio, catalog, &search);

    let mut layers: Vec<_> = catalog
        .wearable()
        .filter(|item| matches(&item.display_name))
        .map(|item| {
            let layer = item
                .equipment
                .as_ref()
                .and_then(|equipment| equipment.placements.first())
                .and_then(|placement| placement.outermost_channel())
                .unwrap_or(EquipmentChannel::Accessory);
            (layer, item)
        })
        .collect();
    layers.sort_by_key(|(layer, item)| (layer.order(), item.display_name.clone()));
    for layer in layers
        .iter()
        .map(|(layer, _)| *layer)
        .collect::<std::collections::BTreeSet<_>>()
    {
        egui::CollapsingHeader::new(format!("Catalog · {}", channel_label(layer)))
            .id_salt(("acquire", layer))
            .default_open(!search.is_empty())
            .show(ui, |ui| {
                for (_, item) in layers.iter().filter(|(other, _)| *other == layer) {
                    catalog_item(ui, studio, catalog, item);
                }
            });
    }
}

/// Garments saved in the wardrobe, each worn as its own copy.
fn wardrobe(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog, search: &str) {
    let saved: Vec<_> = studio
        .wardrobe
        .library
        .iter()
        .filter(|(name, _)| search.is_empty() || name.as_ref().to_lowercase().contains(search))
        .map(|(name, garment)| (name.clone(), garment.clone()))
        .collect();
    if saved.is_empty() {
        return;
    }
    egui::CollapsingHeader::new("Wardrobe")
        .id_salt("acquire_wardrobe")
        .default_open(!search.is_empty())
        .show(ui, |ui| {
            for (name, garment) in saved {
                ui.horizontal(|ui| {
                    ui.label(name.as_ref()).on_hover_text(format!(
                        "{} · {} · saved drape, fitted without simulating",
                        garment.selection.fabric.label(),
                        garment.selection.layer.label()
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("add").clicked() {
                            acquire(studio, catalog, vec![Article::Settled(garment)]);
                        }
                    });
                });
            }
        });
}

fn catalog_item(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    item: &ItemDefinition,
) {
    let placements = &item.equipment.as_ref().expect("wearable item").placements;
    let article = |placement: &str| Article::Catalog(CatalogArticle::new(&item.id, placement));
    ui.horizontal(|ui| {
        ui.label(&item.display_name)
            .on_hover_text(format!("{:.2} kg · {}", item.weight_kg, item.id));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if placements.len() > 1 && ui.small_button("both").clicked() {
                acquire(
                    studio,
                    catalog,
                    placements.iter().map(|p| article(&p.id)).collect(),
                );
            }
            for placement in placements.iter().rev() {
                let label = if placements.len() == 1 {
                    "add"
                } else {
                    &placement.id
                };
                if ui.small_button(label).clicked() {
                    acquire(studio, catalog, vec![article(&placement.id)]);
                }
            }
        });
    });
}

/// Carry the new articles, wear them, and select the last.
/// Add articles to the inventory and wear each, selecting the last.
pub(in super::super) fn acquire(
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    articles: Vec<Article>,
) {
    for article in articles {
        let id = studio.recipe.inventory.add(article);
        studio.inventory.selected = Some(id);
        super::wear(studio, catalog, id);
    }
}
