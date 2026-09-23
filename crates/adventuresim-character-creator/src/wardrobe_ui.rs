//! The wardrobe tab: design and drape a garment on the character's body, save
//! it once it settles, and fit saved garments to the body or wear them.
use super::*;
use adventuresim_character_creator::{inventory::Article, wardrobe::LibraryName};
use wardrobe_tab::{Showing, WardrobeTab};

pub(super) fn show(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    model: &BodyModel,
    wardrobe: &mut WardrobeTab,
) {
    ui.small(
        "Drape a garment on this character's bare body and save it once it \
         settles. Saved garments fit any body without draping again.",
    );
    if !wardrobe.ready() {
        ui.weak("Preparing the body…");
        return;
    }
    if let Some(problems) = wardrobe.problems() {
        ui.colored_label(studio_theme::PROBLEM, problems);
    }
    let title = format!("Saved garments ({})", studio.wardrobe.library.len());
    studio_theme::card(ui, &title, |ui| saved(ui, studio, catalog, wardrobe));
    studio_theme::card(ui, "Design", |ui| design(ui, studio, model, wardrobe));
}

/// The wardrobe's garments; the selected one is fitted to the body.
fn saved(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    wardrobe: &mut WardrobeTab,
) {
    if studio.wardrobe.library.is_empty() {
        ui.weak("Nothing saved yet. Drape a design below and save it.");
        return;
    }
    let mut chosen = None;
    ui.horizontal_wrapped(|ui| {
        for (name, garment) in studio.wardrobe.library.iter() {
            let selected = wardrobe.showing == Showing::Saved(name.clone());
            let hover = format!(
                "{} · {} · {} vertices",
                garment.selection.fabric.label(),
                garment.selection.layer.label(),
                garment.drape.vertex_count()
            );
            if ui
                .selectable_label(selected, name.as_ref())
                .on_hover_text(hover)
                .clicked()
            {
                chosen = Some(name.clone());
            }
        }
    });
    if let Some(name) = chosen {
        wardrobe.show(Showing::Saved(name));
    }
    let Showing::Saved(name) = wardrobe.showing.clone() else {
        ui.small("Select one to fit it to this body from its saved drape.");
        return;
    };
    let Some(garment) = studio.wardrobe.library.get(&name).cloned() else {
        wardrobe.show(Showing::Design);
        return;
    };
    ui.small("Fitted to this body from its saved drape, without simulating.");
    ui.horizontal(|ui| {
        if ui
            .add(studio_theme::primary_button("Add to inventory"))
            .on_hover_text("Wear a copy of it; the recipe keeps its own copy.")
            .clicked()
        {
            inventory_ui::acquire(studio, catalog, vec![Article::Settled(garment.clone())]);
            studio.status = format!("Added {name} to the inventory");
        }
        if ui
            .button("Edit")
            .on_hover_text("Design from its settings and drape it again.")
            .clicked()
        {
            wardrobe.edit(&garment);
        }
        if ui
            .button("Delete")
            .on_hover_text("Remove it from the wardrobe.")
            .clicked()
        {
            delete(studio, wardrobe, &name);
        }
    });
}

fn delete(studio: &mut Studio, wardrobe: &mut WardrobeTab, name: &LibraryName) {
    studio.status = match studio.wardrobe.delete(name) {
        Ok(()) => format!("Deleted {name} from the wardrobe"),
        Err(error) => format!("Could not delete {name}: {error:#}"),
    };
    wardrobe.show(Showing::Design);
}

/// The garment being designed: its pattern, fabric and drape, its progress,
/// and saving it once settled.
fn design(ui: &mut egui::Ui, studio: &mut Studio, model: &BodyModel, wardrobe: &mut WardrobeTab) {
    if wardrobe.showing != Showing::Design && ui.button("Show the design").clicked() {
        wardrobe.show(Showing::Design);
    }
    progress(ui, wardrobe);
    ui.horizontal(|ui| {
        let saved = wardrobe.settled().is_some();
        if ui
            .add_enabled(saved, studio_theme::primary_button("Save to wardrobe"))
            .on_hover_text(format!(
                "Save it as {} to {}, replacing a garment of that name.",
                wardrobe.garment.name, studio.wardrobe.path
            ))
            .on_disabled_hover_text("Wait for the garment to settle.")
            .clicked()
        {
            studio.status = match wardrobe.save(model, &mut studio.wardrobe) {
                Ok(name) => {
                    wardrobe.show(Showing::Saved(name.clone()));
                    format!("Saved {name} to {}", studio.wardrobe.path)
                }
                Err(error) => format!("Could not save the garment: {error:#}"),
            };
        }
        if ui
            .button("Drape again")
            .on_hover_text("Simulate every stage again from the placed panels.")
            .clicked()
        {
            wardrobe.restart();
        }
    });
    ui.separator();
    let id = egui::Id::new("wardrobe_design");
    let before = wardrobe.garment.clone();
    garment_controls::show(ui, id, &mut wardrobe.garment);
    ui.separator();
    fabric_controls::show(ui, id, &mut wardrobe.garment);
    garment_controls::layer(ui, id, &mut wardrobe.garment);
    ui.separator();
    drape_controls::show(ui, &mut wardrobe.garment);
    if wardrobe.garment != before {
        wardrobe.redrape |= !before.same_simulation(&wardrobe.garment);
        wardrobe.show(Showing::Design);
    }
}

fn progress(ui: &mut egui::Ui, wardrobe: &WardrobeTab) {
    let text = match (wardrobe.draping(), wardrobe.stage(), wardrobe.settled()) {
        (_, _, Some(garment)) => format!("Settled · {} vertices", garment.positions.len()),
        (true, Some(stage), _) => format!("Draping: {stage}"),
        (true, None, _) => "Measuring the body and preparing cloth…".into(),
        (false, _, None) => "Not draped as designed; press Drape again.".into(),
    };
    ui.horizontal(|ui| {
        if wardrobe.draping() {
            ui.spinner();
        }
        ui.label(text);
    });
}
