//! The character tab: skeletal build, identity shape, and expression.
use super::*;
use adventuresim_core::character_proportions::{BodyProportion, CharacterProportions};
use studio_scene::Shot;
use studio_theme::stat_slider;

/// MHR identity coefficients span about three standard deviations.
const IDENTITY_LIMIT: f32 = 3.0;
/// Expression blend weights run from full negative to full positive.
const EXPRESSION_LIMIT: f32 = 1.0;
/// Randomized identity stays within a plausible range of the population.
pub(super) const RANDOM_IDENTITY_LIMIT: f32 = 1.35;
pub(super) const IDENTITY_STREAM: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("character-creator.identity");

/// One page of the character tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CharacterPage {
    Build,
    Identity(IdentityGroup),
    Expression,
}

impl CharacterPage {
    const ALL: [Self; 5] = [
        Self::Build,
        Self::Identity(IdentityGroup::Body),
        Self::Identity(IdentityGroup::Head),
        Self::Identity(IdentityGroup::Hands),
        Self::Expression,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Identity(group) => group.label(),
            Self::Expression => "Expression",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Build => "Bone lengths and skeletal proportions.",
            Self::Identity(IdentityGroup::Body) => "The shape of the torso and limbs.",
            Self::Identity(IdentityGroup::Head) => "The shape of the head and face.",
            Self::Identity(IdentityGroup::Hands) => "The shape of the hands.",
            Self::Expression => "The resting facial expression.",
        }
    }

    /// The camera shot that shows what this page edits.
    fn shot(self) -> Shot {
        match self {
            Self::Identity(IdentityGroup::Head) | Self::Expression => Shot::Portrait,
            Self::Build | Self::Identity(_) => Shot::FullBody,
        }
    }

    /// Return this page's values to neutral.
    fn reset(self, recipe: &mut CharacterRecipe) {
        match self {
            Self::Build => recipe.proportions = CharacterProportions::default(),
            Self::Identity(group) => recipe.identity[group.range()].fill(0.0),
            Self::Expression => recipe.reset_face(),
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio, shot: &mut Option<Shot>) {
    ui.horizontal_wrapped(|ui| {
        for page in CharacterPage::ALL {
            if ui
                .selectable_label(studio.page == page, page.label())
                .clicked()
                && studio.page != page
            {
                studio.page = page;
                *shot = Some(page.shot());
            }
        }
    });
    ui.add_space(6.0);
    let page = studio.page;
    studio_theme::card(ui, page.label(), |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(page.description()).color(studio_theme::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("Reset")
                    .on_hover_text("Return this page to neutral.")
                    .clicked()
                {
                    page.reset(&mut studio.recipe);
                    studio.dirty = true;
                }
            });
        });
        ui.add_space(6.0);
        match page {
            CharacterPage::Build => build(ui, studio),
            CharacterPage::Identity(group) => identity(ui, studio, group),
            CharacterPage::Expression => expression(ui, studio),
        }
    });
}

fn build(ui: &mut egui::Ui, studio: &mut Studio) {
    for proportion in BodyProportion::ALL {
        let mut value = studio.recipe.proportions.get(proportion);
        let limit = proportion.limit();
        if stat_slider(ui, proportion.label(), &mut value, -limit..=limit, 0.0).changed() {
            studio
                .recipe
                .proportions
                .set(proportion, value)
                .expect("slider respects MHR limits");
            studio.dirty = true;
        }
    }
}

fn identity(ui: &mut egui::Ui, studio: &mut Studio, group: IdentityGroup) {
    let start = group.range().start;
    for index in group.range() {
        let label = format!("{} shape {:02}", group.label(), index - start + 1);
        let value = &mut studio.recipe.identity[index];
        studio.dirty |=
            stat_slider(ui, &label, value, -IDENTITY_LIMIT..=IDENTITY_LIMIT, 0.0).changed();
    }
}

fn expression(ui: &mut egui::Ui, studio: &mut Studio) {
    for (index, value) in studio.recipe.expression.iter_mut().enumerate() {
        let label = format!("Expression {:02}", index + 1);
        studio.dirty |=
            stat_slider(ui, &label, value, -EXPRESSION_LIMIT..=EXPRESSION_LIMIT, 0.0).changed();
    }
}

/// A new random build and identity; the expression is kept.
pub(super) fn randomize(studio: &mut Studio) {
    studio.seed = studio.seed.wrapping_add(1);
    studio.recipe.proportions = CharacterProportions::from_character_id(studio.seed);
    let mut rng = IDENTITY_STREAM.rng(studio.seed.into(), &[]);
    for value in &mut studio.recipe.identity {
        *value = rng.range_f32(-RANDOM_IDENTITY_LIMIT, RANDOM_IDENTITY_LIMIT);
    }
    studio.dirty = true;
    studio.status = "Rolled a new appearance".into();
}

/// The canonical MHR body with a neutral face.
pub(super) fn neutral(studio: &mut Studio) {
    studio.recipe.reset_body();
    studio.recipe.reset_face();
    studio.dirty = true;
    studio.status = "Appearance reset to neutral".into();
}
