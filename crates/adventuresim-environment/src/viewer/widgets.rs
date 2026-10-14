//! The viewer's own controls: switches, labelled sliders and square icon
//! buttons, styled after [`theme`](crate::theme).

use bevy_egui::egui::{
    self, Align, Color32, Layout, Painter, Rect, Response, RichText, Sense, Slider, Ui, Vec2,
};

use crate::theme;

/// A switch's size, logical pixels.
const SWITCH: Vec2 = Vec2::new(40.0, 22.0);
/// Gap between a switch's knob and its rail, logical pixels.
const KNOB_INSET: f32 = 3.0;

/// A labelled on/off switch across the row.
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| rail(ui, on))
            .inner
    })
    .inner
}

fn rail(ui: &mut Ui, on: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(SWITCH, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let radius = rect.height() * 0.5;
    let fill = if *on { theme::ACCENT } else { theme::HOVERED };
    let painter = ui.painter();
    painter.rect_filled(rect, radius, fill);
    let x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
    painter.circle_filled(
        egui::pos2(x, rect.center().y),
        radius - KNOB_INSET,
        Color32::WHITE,
    );
    response
}

/// A slider under its label, with its value read out on the right.
pub fn setting(ui: &mut Ui, label: &str, value: String, slider: Slider) -> Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).color(theme::MUTED));
        });
    });
    let width = ui.available_width();
    ui.spacing_mut().slider_width = width;
    ui.add(slider.show_value(false))
}

/// A heading inside a panel.
pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .small()
            .strong()
            .color(theme::ACCENT),
    );
}

/// A square button showing a painted icon; lit while `selected`.
pub fn icon_button(
    ui: &mut Ui,
    size: f32,
    selected: bool,
    paint: impl FnOnce(&Painter, Rect, Color32),
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let fill = if selected {
        theme::ACCENT.gamma_multiply(0.3)
    } else if response.hovered() {
        theme::HOVERED
    } else {
        theme::RAISED
    };
    let painter = ui.painter();
    painter.rect_filled(rect, theme::ROUNDING_SMALL, fill);
    if selected {
        painter.rect_stroke(
            rect,
            theme::ROUNDING_SMALL,
            egui::Stroke::new(1.5_f32, theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    let ink = if selected { theme::ACCENT } else { theme::TEXT };
    paint(painter, rect, ink);
    response
}
