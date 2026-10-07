//! The hair page of the character tab: a hairstyle, and its colour.
//!
//! Hair changes no body, so these controls leave the studio clean: the hair
//! preview notices the recipe's new hair and grows or recolours it alone.
use super::*;
use adventuresim_character_creator::hair::{HairDesign, HairPigment, preset_label};
use fabelgeist_hair::Preset;
use studio_theme::stat_slider;

/// What the hairstyle menu calls no hair.
const BALD: &str = "Bald";

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio) {
    let hair = &mut studio.recipe.hair;
    let selected = hair.map_or_else(|| BALD.to_owned(), |design| preset_label(design.preset));
    egui::ComboBox::from_label("Hairstyle")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui.selectable_label(hair.is_none(), BALD).clicked() {
                *hair = None;
            }
            for preset in Preset::ALL {
                let chosen = hair.is_some_and(|design| design.preset == preset);
                if ui.selectable_label(chosen, preset_label(preset)).clicked() {
                    let pigment = hair.map_or_else(HairPigment::default, |design| design.pigment);
                    *hair = Some(HairDesign { preset, pigment });
                }
            }
        });
    let Some(design) = hair else {
        return;
    };
    ui.add_space(6.0);
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
        design.pigment = HairPigment::new(melanin, redness);
    }
}
