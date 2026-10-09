//! The armory tab: browse every catalog piece and reshape its catalog default.
use super::*;
use adventuresim_character_creator::decoration::LibraryName;
use armory::{Armory, ArmoryView, Frame, Region};

pub(super) fn show(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &mut EquipmentCatalog,
    armory: &mut Armory,
) {
    ui.small(
        "Every parametric catalog piece, fitted to this character's body. \
         Shape edits change the catalog defaults.",
    );
    view_controls(ui, armory);
    if !armory.ready() {
        ui.weak("Fitting the catalog…");
        return;
    }
    ui.separator();
    egui::CollapsingHeader::new(format!("Pieces ({})", armory.exhibits.len()))
        .default_open(true)
        .show(ui, |ui| list(ui, armory));
    ui.separator();
    editor(ui, studio, catalog, armory);
}

fn view_controls(ui: &mut egui::Ui, armory: &mut Armory) {
    ui.horizontal(|ui| {
        let before = armory.view;
        ui.selectable_value(&mut armory.view, ArmoryView::Wall, "Wall")
            .on_hover_text("Every piece side by side.");
        ui.add_enabled_ui(armory.selected.is_some(), |ui| {
            ui.selectable_value(&mut armory.view, ArmoryView::OnBody, "On body")
                .on_hover_text("The selected piece where it is worn.")
                .on_disabled_hover_text("Select a piece first.");
        });
        if armory.view != before {
            armory.frame = Some(Frame::Selected);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Refit all")
                .on_hover_text("Fit every piece to the current body again.")
                .clicked()
            {
                armory.refit_all = true;
            }
        });
    });
    ui.horizontal(|ui| match armory.view {
        ArmoryView::Wall => {
            ui.checkbox(&mut armory.both_sides, "Both sides")
                .on_hover_text("Show left and right placements of paired pieces.");
            ui.checkbox(&mut armory.solo, "Selected only");
        }
        ArmoryView::OnBody => {
            ui.checkbox(&mut armory.ghost_body, "See-through body")
                .on_hover_text("Show where the piece clears or cuts into the body.");
        }
    });
    ui.horizontal(|ui| {
        if ui.button("Frame all").clicked() {
            armory.view = ArmoryView::Wall;
            armory.frame = Some(Frame::Wall);
        }
        if ui
            .add_enabled(
                armory.selected.is_some(),
                egui::Button::new("Frame selected"),
            )
            .clicked()
        {
            armory.frame = Some(Frame::Selected);
        }
        step_buttons(ui, armory);
    });
}

/// Previous and next piece; the arrow keys do the same over the viewport.
fn step_buttons(ui: &mut egui::Ui, armory: &mut Armory) {
    let count = armory.exhibits.len();
    if count == 0 {
        return;
    }
    let mut step = 0isize;
    if ui.button("◀").on_hover_text("Previous piece (←)").clicked() {
        step = -1;
    }
    if ui.button("▶").on_hover_text("Next piece (→)").clicked() {
        step = 1;
    }
    if !ui.ctx().egui_wants_keyboard_input() {
        ui.input(|input| {
            step += isize::from(input.key_pressed(egui::Key::ArrowRight));
            step -= isize::from(input.key_pressed(egui::Key::ArrowLeft));
        });
    }
    if step != 0 {
        let current = armory
            .selected
            .map_or(if step > 0 { -1 } else { 0 }, |i| i as isize);
        armory.select((current + step).rem_euclid(count as isize) as usize);
    }
}

fn list(ui: &mut egui::Ui, armory: &mut Armory) {
    let mut chosen = None;
    for region in Region::ALL {
        let pieces: Vec<usize> = (0..armory.exhibits.len())
            .filter(|i| armory.exhibits[*i].region == region)
            .collect();
        if pieces.is_empty() {
            continue;
        }
        ui.small(region.label());
        ui.horizontal_wrapped(|ui| {
            for i in pieces {
                let exhibit = &armory.exhibits[i];
                let problems: Vec<String> = exhibit
                    .errors()
                    .map(|(placement, error)| format!("{placement}: {error}"))
                    .collect();
                let mut text = egui::RichText::new(&exhibit.name);
                if !problems.is_empty() {
                    text = text.color(studio_theme::PROBLEM);
                }
                let hover = if problems.is_empty() {
                    format!("{} · {} triangles", exhibit.item_id, exhibit.triangles())
                } else {
                    problems.join("\n")
                };
                if ui
                    .selectable_label(armory.selected == Some(i), text)
                    .on_hover_text(hover)
                    .clicked()
                {
                    chosen = Some(i);
                }
            }
        });
    }
    if let Some(i) = chosen {
        armory.select(i);
    }
}

fn editor(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &mut EquipmentCatalog,
    armory: &mut Armory,
) {
    let Some(index) = armory.selected else {
        ui.weak("Select a piece here or by its label on the wall.");
        return;
    };
    let exhibit = &armory.exhibits[index];
    let item_id = exhibit.item_id.clone();
    ui.heading(&exhibit.name);
    let material = catalog
        .material(&item_id)
        .map_or_else(|_| String::new(), |material| format!(" · {material:?}"));
    ui.small(format!(
        "{item_id}{material} · {} · {} triangles",
        exhibit.placements.join(", "),
        exhibit.triangles()
    ));
    for (placement, error) in exhibit.errors() {
        ui.colored_label(studio_theme::PROBLEM, format!("{placement}: {error}"));
    }
    let loaded = exhibit.loaded.clone();
    if exhibit.steel {
        construction(ui, armory);
        decoration(ui, studio, armory);
    }
    let Some(mut design) = catalog.design(&item_id) else {
        return;
    };
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(design != loaded, egui::Button::new("Revert"))
            .on_hover_text("Back to the default the armory opened with.")
            .clicked()
        {
            design = loaded;
            changed = true;
        }
        if ui
            .button("Save catalog designs")
            .on_hover_text("Write every catalog default to the paths on the Output tab.")
            .clicked()
        {
            studio_ui::save_designs(studio, catalog);
        }
    });
    changed |= armor_controls::design(ui, &mut design);
    if changed {
        match catalog.designs.set_default(&item_id, design) {
            Ok(()) => armory.refit(index),
            Err(error) => studio.status = format!("Could not change {item_id}: {error:#}"),
        }
    }
}

/// Try a construction on the selected piece.
fn construction(ui: &mut egui::Ui, armory: &mut Armory) {
    studio_theme::card(ui, "Construction", |ui| {
        ui.small(
            "Shown on this piece. Each plate-steel article in the inventory \
             chooses its own construction.",
        );
        construction_controls::edit(ui, &mut armory.construction);
    });
}

/// Design an engraving and trim on the selected piece, and save it to the
/// library that inventory articles choose from.
fn decoration(ui: &mut egui::Ui, studio: &mut Studio, armory: &mut Armory) {
    studio_theme::card(ui, "Decoration", |ui| {
        ui.small(
            "Engraving and trim, shown on this piece. Saved decorations can be \
             chosen for any plate-steel article in the inventory.",
        );
        ui.horizontal(|ui| {
            ui.label("Start from");
            if let Some(choice) = decoration_controls::choose(
                ui,
                "armory_decoration",
                &studio.decorations.library,
                &mut armory.decoration,
            ) {
                armory.decoration_name = choice.map_or_else(String::new, |name| name.to_string());
            }
        });
        decoration_controls::edit(ui, &mut armory.decoration);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut armory.decoration_name)
                    .hint_text("Decoration name")
                    .desired_width(150.0),
            );
            let savable =
                !armory.decoration.is_plain() && !armory.decoration_name.trim().is_empty();
            if ui
                .add_enabled(savable, studio_theme::primary_button("Save to library"))
                .on_hover_text(format!("Write it to {}", studio.decorations.path))
                .on_disabled_hover_text("Name an engraving or trim to save it.")
                .clicked()
            {
                studio.status = match studio
                    .decorations
                    .save(&armory.decoration_name, &armory.decoration)
                {
                    Ok(name) => format!("Saved decoration {name} to {}", studio.decorations.path),
                    Err(error) => format!("Could not save the decoration: {error:#}"),
                };
            }
            let saved = LibraryName::try_from(armory.decoration_name.clone())
                .ok()
                .filter(|name| studio.decorations.library.get(name).is_some());
            if ui
                .add_enabled(saved.is_some(), egui::Button::new("Delete"))
                .on_hover_text("Remove it from the library.")
                .clicked()
                && let Some(name) = saved
            {
                studio.status = match studio.decorations.delete(&name) {
                    Ok(()) => format!("Deleted decoration {name}"),
                    Err(error) => format!("Could not delete the decoration: {error:#}"),
                };
            }
        });
    });
}

/// Name every piece under where it hangs; clicking a name selects the piece.
pub(super) fn labels(
    mut contexts: EguiContexts,
    studio: Res<Studio>,
    mut armory: ResMut<Armory>,
    panel: Res<CreatorPanelRight>,
    camera: Query<(&Camera, &GlobalTransform), With<studio_scene::OrbitCamera>>,
) {
    if studio.tab != studio_ui::StudioTab::Armory || armory.view != ArmoryView::Wall {
        return;
    }
    let (Ok(ctx), Ok((camera, transform))) = (contexts.ctx_mut(), camera.single()) else {
        return;
    };
    let mut chosen = None;
    for (i, exhibit) in armory.exhibits.iter().enumerate() {
        let selected = armory.selected == Some(i);
        if armory.solo && armory.selected.is_some() && !selected {
            continue;
        }
        let Some((lo, hi)) = exhibit.bounds(armory.both_sides) else {
            continue;
        };
        let anchor = Vec3::new((lo.x + hi.x) * 0.5, lo.y - 0.02, (lo.z + hi.z) * 0.5);
        let Ok(position) = camera.world_to_viewport(transform, anchor + exhibit.offset) else {
            continue;
        };
        if position.x <= panel.0 + 20.0 {
            continue;
        }
        let mut text = egui::RichText::new(&exhibit.name).small();
        if exhibit.errors().next().is_some() {
            text = text.color(studio_theme::PROBLEM);
        }
        egui::Area::new(egui::Id::new(("armory_label", i)))
            .fixed_pos(egui::pos2(position.x, position.y))
            .pivot(egui::Align2::CENTER_TOP)
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                if ui.selectable_label(selected, text).clicked() {
                    chosen = Some(i);
                }
            });
    }
    if let Some(i) = chosen {
        armory.select(i);
    }
}
