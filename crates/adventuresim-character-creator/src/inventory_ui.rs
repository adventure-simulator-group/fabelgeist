//! The inventory tab: what the character wears, what it carries, and what it can acquire.
use super::*;
use adventuresim_character_creator::{
    inventory::{Article, InventoryItem, InventoryItemId, channel_label, location_label},
    item_catalog_schema::EquipmentChannel,
};

#[path = "inventory_acquire.rs"]
mod acquire;
#[path = "inventory_article_editor.rs"]
mod article_editor;

const WORN_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 200, 150);

/// Per-session inventory panel state; not part of the recipe.
#[derive(Default)]
pub(super) struct InventoryView {
    pub selected: Option<InventoryItemId>,
    pub search: String,
}

pub(super) fn show(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &mut EquipmentCatalog,
    drape_job: &mut DrapeJob,
) {
    summary(ui, studio, catalog, drape_job);
    ui.separator();

    egui::CollapsingHeader::new("Worn")
        .default_open(true)
        .show(ui, |ui| worn(ui, studio, catalog));
    egui::CollapsingHeader::new("Carried")
        .default_open(true)
        .show(ui, |ui| carried(ui, studio, catalog));
    egui::CollapsingHeader::new("Acquire")
        .default_open(studio.recipe.inventory.items().is_empty())
        .show(ui, |ui| acquire::show(ui, studio, catalog));
    ui.separator();

    article_editor::show(ui, studio, catalog, drape_job);
}

fn summary(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    drape_job: &mut DrapeJob,
) {
    let inventory = &studio.recipe.inventory;
    let worn = inventory.worn().count();
    let carried = inventory.items().len() - worn;
    let weight = |items: &mut dyn Iterator<Item = &InventoryItem>| {
        items
            .filter_map(|item| item.article.weight_kg(catalog))
            .sum::<f32>()
    };
    ui.label(format!(
        "{worn} worn · {carried} carried · {:.1} kg worn of {:.1} kg",
        weight(&mut inventory.worn()),
        weight(&mut inventory.items().iter()),
    ));
    ui.small("Weights count catalog items; generated cloth and plate have no authored weight.");
    if let Err(error) = inventory.fit(catalog) {
        ui.colored_label(egui::Color32::LIGHT_RED, inventory.explain(catalog, &error));
    }
    if wears_draped(&studio.recipe) && ui.button("Drape again").clicked() {
        drape_job.restart_from_placement();
        studio.dirty = true;
    }
}

/// Whether any worn article is draped cloth, which export must wait for.
pub(super) fn wears_draped(recipe: &CharacterRecipe) -> bool {
    recipe
        .inventory
        .worn()
        .any(|item| matches!(item.article, Article::Draped(_)))
}

fn worn(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog) {
    let mut layers: Vec<(EquipmentChannel, InventoryItemId)> = studio
        .recipe
        .inventory
        .worn()
        .map(|item| (layer(item, catalog), item.id))
        .collect();
    if layers.is_empty() {
        ui.weak("Nothing worn.");
        return;
    }
    layers.sort_by_key(|(layer, _)| layer.order());
    let mut current = None;
    for (layer, id) in layers {
        if current != Some(layer) {
            ui.small(channel_label(layer));
            current = Some(layer);
        }
        row(ui, studio, catalog, id);
    }
}

fn carried(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog) {
    let carried: Vec<_> = studio
        .recipe
        .inventory
        .items()
        .iter()
        .filter(|item| !item.worn)
        .map(|item| item.id)
        .collect();
    if carried.is_empty() {
        ui.weak("Nothing carried.");
    }
    for id in carried {
        row(ui, studio, catalog, id);
    }
}

fn layer(item: &InventoryItem, catalog: &EquipmentCatalog) -> EquipmentChannel {
    item.article
        .occupancy(catalog)
        .map_or(EquipmentChannel::Accessory, |occupancy| occupancy.layer())
}

/// One article: wear toggle, name, where it sits, and ordering and discard buttons.
fn row(ui: &mut egui::Ui, studio: &mut Studio, catalog: &EquipmentCatalog, id: InventoryItemId) {
    let Some(item) = studio.recipe.inventory.get(id) else {
        return;
    };
    let name = item.article.name(catalog);
    let worn = item.worn;
    let locations = item.article.occupancy(catalog).map_or_else(
        |_| String::new(),
        |occupancy| {
            let mut labels: Vec<_> = occupancy.locations().map(location_label).collect();
            labels.dedup();
            labels.join(", ")
        },
    );
    ui.push_id(id, |ui| {
        ui.horizontal(|ui| {
            let mut wear = worn;
            if ui
                .checkbox(&mut wear, "")
                .on_hover_text(if worn { "Take off" } else { "Wear" })
                .changed()
            {
                if wear {
                    self::wear(studio, catalog, id);
                } else {
                    take_off(studio, catalog, id);
                }
            }
            let selected = studio.inventory.selected == Some(id);
            let text = if worn {
                egui::RichText::new(&name).color(WORN_COLOR)
            } else {
                egui::RichText::new(&name)
            };
            if ui
                .selectable_label(selected, text)
                .on_hover_text(&locations)
                .clicked()
            {
                studio.inventory.selected = (!selected).then_some(id);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").on_hover_text("Discard").clicked() {
                    discard(studio, catalog, id);
                }
                if ui.small_button("⏷").on_hover_text("Move later").clicked() {
                    shift(studio, id, true);
                }
                if ui.small_button("⏶").on_hover_text("Move earlier").clicked() {
                    shift(studio, id, false);
                }
            });
        });
    });
}

/// Wear an article, reporting whatever it replaced.
pub(super) fn wear(studio: &mut Studio, catalog: &EquipmentCatalog, id: InventoryItemId) {
    let inventory = &mut studio.recipe.inventory;
    let name = |inventory: &adventuresim_character_creator::inventory::Inventory, id| {
        inventory
            .get(id)
            .map_or_else(String::new, |item: &InventoryItem| {
                item.article.name(catalog)
            })
    };
    studio.status = match inventory.wear(id, catalog) {
        Ok(displaced) => {
            studio.dirty = true;
            if displaced.is_empty() {
                format!("Wearing {}", name(inventory, id))
            } else {
                format!(
                    "Wearing {}; took off {}",
                    name(inventory, id),
                    displaced
                        .iter()
                        .map(|id| name(inventory, *id))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        Err(conflict) => inventory.explain(catalog, &(id, conflict)),
    };
}

fn take_off(studio: &mut Studio, catalog: &EquipmentCatalog, id: InventoryItemId) {
    let removed = studio.recipe.inventory.take_off(id, catalog);
    studio.dirty |= !removed.is_empty();
    studio.status = format!(
        "Took off {}",
        names(&studio.recipe.inventory, catalog, &removed)
    );
}

fn discard(studio: &mut Studio, catalog: &EquipmentCatalog, id: InventoryItemId) {
    let name = names(&studio.recipe.inventory, catalog, &[id]);
    let removed = studio.recipe.inventory.remove(id, catalog);
    studio.dirty |= !removed.is_empty();
    if studio.inventory.selected == Some(id) {
        studio.inventory.selected = None;
    }
    studio.status = format!("Discarded {name}");
}

fn shift(studio: &mut Studio, id: InventoryItemId, later: bool) {
    studio.recipe.inventory.shift(id, later);
    // Order only matters for draping worn cloth.
    studio.dirty |= studio
        .recipe
        .inventory
        .get(id)
        .is_some_and(|item| item.worn && matches!(item.article, Article::Draped(_)));
}

fn names(
    inventory: &adventuresim_character_creator::inventory::Inventory,
    catalog: &EquipmentCatalog,
    ids: &[InventoryItemId],
) -> String {
    ids.iter()
        .filter_map(|id| inventory.get(*id))
        .map(|item| item.article.name(catalog))
        .collect::<Vec<_>>()
        .join(", ")
}
