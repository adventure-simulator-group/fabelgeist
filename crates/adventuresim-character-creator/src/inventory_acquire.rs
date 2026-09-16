//! Add catalog items, generated cloth and plate armor to the inventory.
use super::*;
use adventuresim_character_creator::inventory::CatalogArticle;

/// Generated articles that are not catalog items.
fn generated() -> [(&'static str, Article); 8] {
    let draped = |preset| {
        Article::Draped(GarmentSelection {
            preset,
            ..GarmentSelection::default()
        })
    };
    [
        ("Cloth shirt", draped(GarmentPreset::Shirt)),
        ("Fitted shirt", draped(GarmentPreset::FittedShirt)),
        ("Trousers", draped(GarmentPreset::Trousers)),
        ("Skirt", draped(GarmentPreset::Skirt)),
        ("Dress", draped(GarmentPreset::Dress)),
        (
            "Chainmail shirt",
            Article::Draped(GarmentSelection::chainmail()),
        ),
        (
            "Chainmail coif",
            Article::Draped(GarmentSelection::chainmail_coif()),
        ),
        (
            "Plate breastplate",
            Article::Plate(fabelgeist_armor::Armor::default()),
        ),
    ]
}

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog) {
    ui.small("New items are worn at once, replacing whatever fills their place.");
    ui.horizontal(|ui| {
        ui.label("Search");
        ui.text_edit_singleline(&mut studio.inventory.search);
    });
    let search = studio.inventory.search.trim().to_lowercase();
    let matches = |name: &str| search.is_empty() || name.to_lowercase().contains(&search);

    ui.label("Draped cloth and plate");
    ui.horizontal_wrapped(|ui| {
        for (label, article) in generated() {
            if matches(label) && ui.button(label).clicked() {
                acquire(studio, catalog, vec![article]);
            }
        }
    });

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
fn acquire(studio: &mut Studio, catalog: &EquipmentCatalog, articles: Vec<Article>) {
    for article in articles {
        let id = studio.recipe.inventory.add(article);
        studio.inventory.selected = Some(id);
        super::wear(studio, catalog, id);
    }
}
