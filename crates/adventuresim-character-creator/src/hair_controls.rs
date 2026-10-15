//! The hair page of the character tab: a hairstyle, its colour, how it is
//! varied from its preset, and the level of detail it is drawn at.
//!
//! Hair changes no body, so these controls leave the studio clean: the hair
//! preview notices the recipe's new hair and grows or recolours it alone.
//! The level of detail is the studio's, not the recipe's: a game picks it
//! from how large the head is on screen.
use super::*;
use adventuresim_character_creator::hair::{
    CLUMPING, CURL, FRIZZ, HairDesign, HairPigment, HairShape, LENGTH, PART, VOLUME, WAVE,
    variant_label,
};
use fabelgeist_hair::lod::Form;
use fabelgeist_hair::{Arrangement, Cut, Growth, Preset};
use hair_preview::HairPreview;
use std::ops::RangeInclusive;
use studio_theme::stat_slider;

/// What the hairstyle menu calls no hair.
const BALD: &str = "Bald";
/// What a cut, arrangement or growth menu calls the preset's own.
const PRESETS_OWN: &str = "Preset's own";
/// The sliders' units: centimetres and millimetres a metre.
const CENTIMETRES: f32 = 100.0;
const MILLIMETRES: f32 = 1000.0;
/// The share of strands a strand level starts out drawing.
const START_SHARE: f32 = 0.25;
/// The fewest of a level's strands the share slider draws.
const MIN_SHARE: f32 = 0.01;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio, preview: &HairPreview) {
    let hair = &mut studio.recipe.hair;
    let selected = hair.map_or_else(|| BALD.to_owned(), |design| variant_label(design.preset));
    egui::ComboBox::from_label("Hairstyle")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui.selectable_label(hair.is_none(), BALD).clicked() {
                *hair = None;
            }
            for preset in Preset::ALL {
                let chosen = hair.is_some_and(|design| design.preset == preset);
                if ui.selectable_label(chosen, variant_label(preset)).clicked() {
                    // A new hairstyle keeps its colour, and starts as its
                    // preset has it.
                    let pigment = hair.map_or_else(HairPigment::default, |design| design.pigment);
                    *hair = Some(HairDesign {
                        preset,
                        shape: HairShape::default(),
                        pigment,
                    });
                }
            }
        });
    let Some(design) = hair else {
        return;
    };
    ui.add_space(6.0);
    hair_color_controls::show(ui, design);
    ui.add_space(6.0);
    shape(ui, design);
    ui.add_space(6.0);
    level_of_detail(ui, &mut studio.hair_form, preview);
    ui.add_space(6.0);
    layers(ui, &mut studio.hair_layers);
}

/// Draw the hair and the scalp under it apart, to look at either alone.
fn layers(ui: &mut egui::Ui, shown: &mut hair_preview::ShownLayers) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut shown.hair, "Show hair")
            .on_hover_text("Hide the strands or cards to see the scalp alone.");
        ui.checkbox(&mut shown.scalp, "Show scalp")
            .on_hover_text("Hide the painted scalp to see the hair alone.");
    });
}

/// How the hairstyle is varied from its preset: another cut, arrangement or
/// growth pattern, and its lengths, volume, part, curl, waves, frizz and
/// clumping. The sliders show centimetres and millimetres where the shape
/// keeps metres.
fn shape(ui: &mut egui::Ui, design: &mut HairDesign) {
    let shape = &mut design.shape;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Shape").color(studio_theme::PARCHMENT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let reset = ui
                .small_button(PRESETS_OWN)
                .on_hover_text("Grow the hairstyle as its preset has it.");
            if reset.clicked() {
                *shape = HairShape::default();
            }
        });
    });
    let cut = variant(ui, "Cut", shape.cut(), &Cut::ALL);
    let arrangement = variant(ui, "Arrangement", shape.arrangement(), &Arrangement::ALL);
    let growth = variant(ui, "Growth", shape.growth(), &Growth::ALL);
    *shape = shape
        .with_cut(cut)
        .with_arrangement(arrangement)
        .with_growth(growth);
    let neutral = HairShape::default().knobs();
    let mut knobs = shape.knobs();
    let mut changed = false;
    let mut slide = |label: &str,
                     hint: &str,
                     value: &mut f32,
                     range: RangeInclusive<f32>,
                     neutral: f32,
                     unit: f32| {
        let mut shown = *value * unit;
        let range = range.start() * unit..=range.end() * unit;
        if stat_slider(ui, label, &mut shown, range, neutral * unit)
            .on_hover_text(hint)
            .changed()
        {
            *value = shown / unit;
            changed = true;
        }
    };
    slide(
        "Length",
        "Every length of the cut, as a share of the preset's.",
        &mut knobs.length,
        LENGTH,
        neutral.length,
        1.0,
    );
    slide(
        "Volume, cm",
        "How much further the hair stands off the crown.",
        &mut knobs.volume,
        VOLUME,
        neutral.volume,
        CENTIMETRES,
    );
    slide(
        "Part, cm",
        "Where the hair parts: towards the character's left or right.",
        &mut knobs.part,
        PART,
        neutral.part,
        CENTIMETRES,
    );
    slide(
        "Curl",
        "Added to how much the strands curl, from straight to tight coils.",
        &mut knobs.curl,
        CURL,
        neutral.curl,
        1.0,
    );
    slide(
        "Waves, mm",
        "Added to how far the strands wave either side.",
        &mut knobs.wave,
        WAVE,
        neutral.wave,
        MILLIMETRES,
    );
    slide(
        "Frizz",
        "How far stray strands wander, and how many fly away, as a share of the preset's.",
        &mut knobs.frizz,
        FRIZZ,
        neutral.frizz,
        1.0,
    );
    slide(
        "Clumping",
        "Added to how much the strands gather into locks.",
        &mut knobs.clumping,
        CLUMPING,
        neutral.clumping,
        1.0,
    );
    if changed {
        *shape = shape.with_knobs(knobs);
    }
}

/// A menu of `options`, or the preset's own when none is picked.
fn variant<T: Copy + PartialEq + std::fmt::Debug>(
    ui: &mut egui::Ui,
    label: &str,
    picked: Option<T>,
    options: &[T],
) -> Option<T> {
    let mut picked = picked;
    let selected = picked.map_or_else(|| PRESETS_OWN.to_owned(), variant_label);
    egui::ComboBox::from_label(label)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            if ui.selectable_label(picked.is_none(), PRESETS_OWN).clicked() {
                picked = None;
            }
            for &option in options {
                let chosen = picked == Some(option);
                if ui.selectable_label(chosen, variant_label(option)).clicked() {
                    picked = Some(option);
                }
            }
        });
    picked
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
