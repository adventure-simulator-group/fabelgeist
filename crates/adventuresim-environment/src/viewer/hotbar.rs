//! The hotbar along the bottom: the tool in hand. Look moves only the
//! camera; each plant slot sows that species where one clicks; Clear takes
//! the plants out of a ring. A hint under it says what a click will do.

use bevy_egui::egui::{
    self, Align2, Color32, Context, FontId, Id, Painter, Rect, Response, RichText, Sense, Ui, pos2,
    vec2,
};

use crate::live::Viewer;
use crate::tools::Tool;
use crate::{hud, icons, theme};

/// The keys that pick the plant slots, in order.
pub const SPECIES_KEYS: [egui::Key; 9] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
];
/// A slot's size, logical pixels.
const SLOT: egui::Vec2 = egui::vec2(74.0, 82.0);
/// The icon's share of a slot's height.
const ICON_SHARE: f32 = 0.56;
/// A slot's small print: its key and its count, and its name.
const SMALL_PRINT: f32 = 11.0;
const NAME_PRINT: f32 = 12.0;
/// Room round a slot's small print, logical pixels.
const SLOT_INSET: f32 = 6.0;

pub fn show(ctx: &Context, viewer: &mut Viewer) {
    let hint = viewer.tool.hint(viewer.stats.species());
    egui::Area::new(Id::new("hotbar"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -theme::GUTTER))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if !hint.is_empty() {
                    theme::panel()
                        .inner_margin(egui::Margin::symmetric(12, 5))
                        .show(ui, |ui| {
                            ui.label(RichText::new(hint).color(theme::MUTED));
                        });
                }
                theme::panel().show(ui, |ui| {
                    ui.horizontal(|ui| slots(ui, viewer));
                });
            });
        });
}

fn slots(ui: &mut Ui, viewer: &mut Viewer) {
    let look = slot(
        ui,
        viewer.tool == Tool::Look,
        "Esc",
        "Look",
        None,
        icons::eye,
    );
    let look = look.on_hover_text("Look around: the mouse only moves the camera");
    if look.clicked() {
        viewer.tool = Tool::Look;
    }
    ui.separator();
    let mut picked = None;
    for (index, species) in viewer.stats.species().iter().enumerate() {
        let key = SPECIES_KEYS
            .get(index)
            .map_or(String::new(), |key| key.symbol_or_name().to_owned());
        let selected = viewer.tool == Tool::Sow(index);
        let count = hud::grouped(species.plants);
        let response = slot(
            ui,
            selected,
            &key,
            &species.name,
            Some(&count),
            |p, r, _| {
                icons::plant(p, r, species);
            },
        );
        let response = response.on_hover_text(format!("Sow {}: {} growing", species.name, count));
        if response.clicked() {
            picked = Some(Tool::Sow(index));
        }
    }
    if let Some(tool) = picked {
        viewer.tool = tool;
    }
    ui.separator();
    let clear = slot(
        ui,
        viewer.tool == Tool::Clear,
        "G",
        "Clear",
        None,
        |p, r, _| {
            icons::clear(p, r, theme::DANGER);
        },
    );
    let clear = clear.on_hover_text("Clear a ring of ground of every plant");
    if clear.clicked() {
        viewer.tool = Tool::Clear;
    }
}

/// One slot: its icon, its name, the key that picks it and maybe a count.
fn slot(
    ui: &mut Ui,
    selected: bool,
    key: &str,
    name: &str,
    count: Option<&str>,
    paint: impl FnOnce(&Painter, Rect, Color32),
) -> Response {
    let (rect, response) = ui.allocate_exact_size(SLOT, Sense::click());
    let fill = if selected {
        theme::ACCENT.gamma_multiply(0.25)
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
            egui::Stroke::new(2.0_f32, theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    let icon = Rect::from_center_size(
        pos2(
            rect.center().x,
            rect.top() + rect.height() * ICON_SHARE * 0.5 + 8.0,
        ),
        egui::Vec2::splat(rect.height() * ICON_SHARE * 0.8),
    );
    let ink = if selected { theme::ACCENT } else { theme::TEXT };
    paint(painter, icon, ink);
    let small = FontId::proportional(SMALL_PRINT);
    painter.text(
        rect.left_top() + vec2(SLOT_INSET, SLOT_INSET * 0.6),
        Align2::LEFT_TOP,
        key,
        small.clone(),
        theme::MUTED,
    );
    if let Some(count) = count {
        painter.text(
            rect.right_top() + vec2(-SLOT_INSET, SLOT_INSET * 0.6),
            Align2::RIGHT_TOP,
            count,
            small,
            theme::MUTED,
        );
    }
    let name = elide(painter, name, rect.width() - SLOT_INSET);
    painter.text(
        pos2(rect.center().x, rect.bottom() - SLOT_INSET),
        Align2::CENTER_BOTTOM,
        name,
        FontId::proportional(NAME_PRINT),
        ink,
    );
    response
}

/// `name`, cut short with an ellipsis to fit `width`.
fn elide(painter: &Painter, name: &str, width: f32) -> String {
    let font = FontId::proportional(NAME_PRINT);
    let fits = |text: &str| {
        painter
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
            <= width
    };
    if fits(name) {
        return name.to_owned();
    }
    let mut cut: String = name.to_owned();
    while cut.pop().is_some() {
        let shown = format!("{}…", cut.trim_end());
        if fits(&shown) {
            return shown;
        }
    }
    String::new()
}
