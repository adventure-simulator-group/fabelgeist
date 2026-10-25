//! The hair page's colours: natural colours and dyes to pick at a click,
//! any dye from a colour picker, and the melanin and redness sliders that
//! set a natural colour finely.
//!
//! A dye tints the strands' own colour, as a real dye does: on dark hair it
//! barely shows, so each dye swatch also lightens the hair, as bleaching
//! before dyeing does.
use super::*;
use adventuresim_character_creator::hair::{HairDesign, HairPigment, NO_DYE};
use studio_theme::stat_slider;

/// The hair crate's named natural colours, darkest first.
const NATURAL: [&str; 8] = [
    "black",
    "dark brown",
    "brown",
    "auburn",
    "red",
    "blond",
    "platinum",
    "grey",
];

/// A dye swatch: its name, the hair it is put on, and the dye.
struct Dye {
    name: &'static str,
    melanin: f32,
    redness: f32,
    dye: [f32; 3],
}

/// Dyes as hair is commonly dyed, on hair bleached light enough to show
/// them.
const DYES: [Dye; 10] = [
    Dye {
        name: "Crimson",
        melanin: 0.3,
        redness: 0.6,
        dye: [1.0, 0.12, 0.15],
    },
    Dye {
        name: "Copper",
        melanin: 0.25,
        redness: 0.6,
        dye: [1.0, 0.5, 0.2],
    },
    Dye {
        name: "Rose",
        melanin: 0.12,
        redness: 0.3,
        dye: [1.0, 0.35, 0.55],
    },
    Dye {
        name: "Pastel pink",
        melanin: 0.06,
        redness: 0.2,
        dye: [1.0, 0.62, 0.78],
    },
    Dye {
        name: "Lavender",
        melanin: 0.06,
        redness: 0.1,
        dye: [0.75, 0.62, 1.0],
    },
    Dye {
        name: "Violet",
        melanin: 0.12,
        redness: 0.1,
        dye: [0.5, 0.22, 1.0],
    },
    Dye {
        name: "Electric blue",
        melanin: 0.1,
        redness: 0.05,
        dye: [0.12, 0.38, 1.0],
    },
    Dye {
        name: "Teal",
        melanin: 0.1,
        redness: 0.05,
        dye: [0.1, 0.75, 0.7],
    },
    Dye {
        name: "Emerald",
        melanin: 0.12,
        redness: 0.05,
        dye: [0.15, 0.8, 0.3],
    },
    Dye {
        name: "Silver",
        melanin: 0.04,
        redness: 0.02,
        dye: [0.85, 0.9, 1.0],
    },
];

/// A swatch's side, its corners' radius, and the outline of the picked
/// and of the hovered swatch, points.
const SWATCH: f32 = 22.0;
const SWATCH_ROUNDING: f32 = 3.0;
const PICKED_OUTLINE: f32 = 2.0;
const HOVERED_OUTLINE: f32 = 1.0;

/// Pick the hair's colour.
pub(super) fn show(ui: &mut egui::Ui, design: &mut HairDesign) {
    ui.label(egui::RichText::new("Natural").color(studio_theme::PARCHMENT));
    ui.horizontal_wrapped(|ui| {
        for name in NATURAL {
            let Some(pigment) = HairPigment::named(name) else {
                continue;
            };
            if swatch(ui, pigment, design.pigment, &capitalized(name)) {
                design.pigment = pigment;
            }
        }
    });
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Dyed").color(studio_theme::PARCHMENT));
    ui.horizontal_wrapped(|ui| {
        for dye in &DYES {
            let pigment = HairPigment::new(dye.melanin, dye.redness).dyed(dye.dye);
            if swatch(ui, pigment, design.pigment, dye.name) {
                design.pigment = pigment;
            }
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let mut dye = design.pigment.dye();
        let picked = egui::color_picker::color_edit_button_rgb(ui, &mut dye)
            .on_hover_text("Any dye: it tints the hair's own colour.")
            .changed();
        if picked {
            design.pigment = design.pigment.dyed(dye);
        }
        ui.label(egui::RichText::new("Dye").color(studio_theme::PARCHMENT));
        let undyed = design.pigment.dye() == NO_DYE;
        let clear = ui.add_enabled(!undyed, egui::Button::new("No dye").small());
        if clear.clicked() {
            design.pigment = design.pigment.dyed(NO_DYE);
        }
    });
    ui.add_space(4.0);
    sliders(ui, design);
}

/// The melanin and redness sliders, keeping any dye.
fn sliders(ui: &mut egui::Ui, design: &mut HairDesign) {
    let neutral = HairPigment::default();
    let mut melanin = design.pigment.melanin();
    let mut redness = design.pigment.redness();
    let darker = stat_slider(ui, "Melanin", &mut melanin, 0.0..=1.0, neutral.melanin())
        .on_hover_text("From white through blond and brown to black.")
        .changed();
    let redder = stat_slider(ui, "Redness", &mut redness, 0.0..=1.0, neutral.redness())
        .on_hover_text("From ashen to red.")
        .changed();
    if darker || redder {
        design.pigment = HairPigment::new(melanin, redness).dyed(design.pigment.dye());
    }
}

/// A square of `pigment`'s colour, outlined when it is the `current` one;
/// true when clicked.
fn swatch(ui: &mut egui::Ui, pigment: HairPigment, current: HairPigment, name: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(SWATCH, SWATCH), egui::Sense::click());
    let [red, green, blue] = pigment.color();
    let fill = egui::Color32::from(egui::Rgba::from_rgb(red, green, blue));
    let painter = ui.painter();
    painter.rect_filled(rect, SWATCH_ROUNDING, fill);
    if pigment == current {
        painter.rect_stroke(
            rect.expand(PICKED_OUTLINE),
            SWATCH_ROUNDING + PICKED_OUTLINE,
            egui::Stroke::new(PICKED_OUTLINE, studio_theme::GOLD),
            egui::StrokeKind::Outside,
        );
    } else if response.hovered() {
        painter.rect_stroke(
            rect,
            SWATCH_ROUNDING,
            egui::Stroke::new(HOVERED_OUTLINE, studio_theme::PARCHMENT),
            egui::StrokeKind::Outside,
        );
    }
    response.on_hover_text(name).clicked()
}

/// `name` with its first letter capitalized, as the interface shows it.
fn capitalized(name: &str) -> String {
    let mut letters = name.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}
