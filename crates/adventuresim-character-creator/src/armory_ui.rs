//! The armory tab: browse every catalog piece and reshape its catalog default.
use super::*;
use armory::{Armory, ArmoryView, Frame, Region};

const PROBLEM_COLOR: egui::Color32 = egui::Color32::from_rgb(235, 120, 110);

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
                    text = text.color(PROBLEM_COLOR);
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
        ui.colored_label(PROBLEM_COLOR, format!("{placement}: {error}"));
    }
    let Some(mut design) = catalog.design(&item_id) else {
        return;
    };
    let loaded = exhibit.loaded.clone();
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
            text = text.color(PROBLEM_COLOR);
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
