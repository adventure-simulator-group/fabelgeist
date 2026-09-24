mod composition;
mod drawing;
mod files;
pub(crate) mod mixer;
mod paint;
mod references;
mod surface;
use crate::app::{Channel, Studio};
use adventuresim_heraldry::{bake::Resolution, document::*};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
pub(super) fn combo<T: Copy + PartialEq + std::fmt::Debug>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    choices: &[T],
) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(label)
            .selected_text(format!("{value:?}"))
            .show_ui(ui, |ui| {
                for choice in choices {
                    ui.selectable_value(value, *choice, format!("{choice:?}"));
                }
            });
    });
}
pub(super) fn slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    ui.add(egui::Slider::new(value, range).text(label));
}
pub(super) fn tincture(ui: &mut egui::Ui, label: &str, t: &mut Tincture) {
    combo(ui, label, t, &Tincture::ALL);
}
pub(crate) fn draw(mut contexts: EguiContexts, mut studio: ResMut<Studio>, time: Res<Time>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_visuals(egui::Visuals::dark());
    let mut root = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("studio-root"),
        egui::UiBuilder::new()
            .max_rect(ctx.content_rect())
            .layer_id(egui::LayerId::background()),
    );
    let now = time.elapsed_secs_f64();
    let mut d = studio.document.clone();
    toolbar(&mut root, &mut studio, &mut d, now);
    egui::Panel::bottom("status").show_inside(&mut root, |ui| {
        ui.horizontal(|ui| {
            ui.label(&studio.status);
            if !studio.ready() {
                ui.weak("Preview is updating");
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak("FABELGEIST  /  HERALDRY");
            });
        });
    });
    egui::Panel::left("arms")
        .default_size(340.0)
        .size_range(280.0..=480.0)
        .show_inside(&mut root, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("arms-scroll")
                .show(ui, |ui| {
                    ui.heading("Arms");
                    ui.text_edit_singleline(&mut d.name);
                    ui.add_space(8.0);
                    references::recipes(ui, &mut d);
                    ui.separator();
                    composition::arms(ui, &mut d.arms, 0);
                    ui.separator();
                    for advice in d.advice() {
                        ui.colored_label(egui::Color32::from_rgb(224, 181, 98), advice);
                    }
                    references::sources(ui);
                });
        });
    egui::Panel::right("interpretation")
        .default_size(310.0)
        .size_range(280.0..=430.0)
        .show_inside(&mut root, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("style-scroll")
                .show(ui, |ui| {
                    drawing::controls(ui, &mut d.drawing);
                    ui.separator();
                    surface::controls(ui, &mut d.surface, &mut studio.mixer);
                    ui.separator();
                    ui.heading("Viewing");
                    slider(ui, "Turn", &mut d.view.yaw.0, -180.0..=180.0);
                    slider(ui, "Tilt", &mut d.view.pitch.0, -80.0..=80.0);
                    slider(ui, "Light", &mut d.view.light.0, -180.0..=180.0);
                    slider(ui, "Exposure", &mut d.view.exposure, -3.0..=3.0);
                    slider(ui, "Zoom", &mut d.view.zoom.0, 0.4..=3.0);
                    ui.weak("Viewing changes preserve the material bake.");
                });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(&mut root, |ui| {
            studio.viewport = ui.max_rect();
            let response = ui.allocate_rect(ui.max_rect(), egui::Sense::drag());
            if response.dragged() {
                let delta = ctx.input(|i| i.pointer.delta());
                d.view.yaw.0 = (d.view.yaw.0 + delta.x * 0.35).clamp(-180.0, 180.0);
                d.view.pitch.0 = (d.view.pitch.0 + delta.y * 0.35).clamp(-80.0, 80.0);
            }
            if response.hovered() {
                let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                d.view.zoom.0 = (d.view.zoom.0 * (scroll * 0.001).exp()).clamp(0.4, 3.0);
            }
            if studio.pinned.is_some() {
                ui.label("Pinned                                                      Current");
            }
        });
    files::json_window(ctx, &mut studio, &mut d);
    studio.mixer.show(ctx, &mut d.surface.palette);
    accept(&mut studio, d, now, ctx.input(|i| i.pointer.any_down()));
}
fn toolbar(root: &mut egui::Ui, s: &mut Studio, d: &mut Document, now: f64) {
    egui::Panel::top("toolbar").show_inside(root, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Heraldry Studio");
            ui.separator();
            if ui
                .add_enabled(!s.undo.is_empty(), egui::Button::new("Undo"))
                .clicked()
            {
                s.redo.push(d.clone());
                *d = s.undo.pop().unwrap();
                s.replace(d.clone(), now);
            }
            if ui
                .add_enabled(!s.redo.is_empty(), egui::Button::new("Redo"))
                .clicked()
            {
                s.undo.push(d.clone());
                *d = s.redo.pop().unwrap();
                s.replace(d.clone(), now);
            }
            files::buttons(ui, s, d);
            if ui.button("Export bundle").clicked() {
                s.export_requested = true;
            }
            if ui
                .add_enabled(s.ready(), egui::Button::new("Capture PNG"))
                .clicked()
            {
                s.screenshot = true;
            }
            let old = s.channel;
            combo(
                ui,
                "View",
                &mut s.channel,
                &[
                    Channel::Physical,
                    Channel::Flat,
                    Channel::BaseColor,
                    Channel::Normal,
                    Channel::Roughness,
                    Channel::Metallic,
                    Channel::Coating,
                    Channel::Height,
                ],
            );
            if s.channel != old {
                s.scene_dirty = true;
            }
            combo(ui, "Quality", &mut s.resolution, &Resolution::ALL);
            if ui
                .add_enabled(s.ready(), egui::Button::new("Pin comparison"))
                .clicked()
                && let Some(b) = &s.current
            {
                s.pinned = Some((s.document.clone(), b.clone()));
                s.scene_dirty = true;
            }
            if s.pinned.is_some() && ui.button("Unpin").clicked() {
                s.pinned = None;
                s.scene_dirty = true;
            }
        });
    });
}
fn accept(s: &mut Studio, d: Document, now: f64, dragging: bool) {
    if d != s.document {
        match d.validate() {
            Ok(()) => {
                if s.transaction.is_none() {
                    s.transaction = Some(s.document.clone());
                }
                s.replace(d, now);
            }
            Err(error) => s.status = format!("Edit rejected: {error}"),
        }
    }
    if !dragging && let Some(before) = s.transaction.take() {
        s.remember(before);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_drag_is_one_undo_step_and_invalid_edits_leave_the_document_unchanged() {
        let mut s = Studio::new(Document::default());
        let start = s.document.clone();
        let mut d = start.clone();
        d.drawing.lion.body_width = Ratio(1.1);
        accept(&mut s, d.clone(), 1.0, true);
        d.drawing.lion.body_width = Ratio(1.2);
        accept(&mut s, d.clone(), 2.0, true);
        assert!(s.undo.is_empty());
        accept(&mut s, d.clone(), 3.0, false);
        assert_eq!(s.undo, vec![start]);
        d.surface.width.0 = -1.0;
        accept(&mut s, d, 4.0, false);
        assert_eq!(s.document.surface.width.0, 450.0);
        assert_eq!(s.undo.len(), 1);
    }
}
