//! The hair page of the character tab: a hairstyle, its colour, and the
//! level of detail it is drawn at.
//!
//! Hair changes no body, so these controls leave the studio clean: the hair
//! preview notices the recipe's new hair and grows or recolours it alone.
//! The level of detail is the studio's, not the recipe's: a game picks it
//! from how large the head is on screen.
use super::*;
use adventuresim_character_creator::hair::{HairDesign, HairPigment, preset_label};
use fabelgeist_hair::Preset;
use fabelgeist_hair::lod::Form;
use hair_preview::HairPreview;
use studio_theme::stat_slider;

/// What the hairstyle menu calls no hair.
const BALD: &str = "Bald";
/// The share of strands a strand level starts out drawing.
const START_SHARE: f32 = 0.25;
/// The fewest of a level's strands the share slider draws.
const MIN_SHARE: f32 = 0.01;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio, preview: &HairPreview) {
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
    ui.add_space(6.0);
    level_of_detail(ui, &mut studio.hair_form, preview);
}

/// Pick a strand level or a cut of cards, and how many of a level's strands
/// to draw.
fn level_of_detail(ui: &mut egui::Ui, form: &mut Form, preview: &HairPreview) {
    let choices = preview.form_choices();
    if preview.preparing() || choices.is_empty() {
        ui.label(egui::RichText::new("Growing the hair…").color(studio_theme::MUTED));
        return;
    }
    let current = choices
        .iter()
        .find(|choice| same_level(choice.form, *form))
        .map_or_else(String::new, |choice| choice.label.clone());
    egui::ComboBox::from_label("Level of detail")
        .selected_text(current)
        .show_ui(ui, |ui| {
            for choice in &choices {
                let chosen = same_level(choice.form, *form);
                if ui.selectable_label(chosen, &choice.label).clicked() && !chosen {
                    *form = match choice.form {
                        Form::Strands { level, .. } => Form::Strands {
                            level,
                            share: START_SHARE,
                        },
                        cards => cards,
                    };
                }
            }
        });
    let Form::Strands { level, mut share } = *form else {
        return;
    };
    let resized = stat_slider(
        ui,
        "Strands drawn",
        &mut share,
        MIN_SHARE..=1.0,
        START_SHARE,
    )
    .changed();
    if resized {
        *form = Form::Strands { level, share };
    }
    if let Some(count) = preview.strands_drawn(*form) {
        ui.label(
            egui::RichText::new(format!(
                "{} of {} strands, widened to keep the head covered",
                count.drawn, count.all
            ))
            .color(studio_theme::MUTED),
        );
    }
}

/// Whether two forms draw the same level, whatever share of its strands.
fn same_level(a: Form, b: Form) -> bool {
    match (a, b) {
        (Form::Strands { level: a, .. }, Form::Strands { level: b, .. })
        | (Form::Cards { level: a }, Form::Cards { level: b }) => a == b,
        _ => false,
    }
}
