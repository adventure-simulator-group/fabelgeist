use super::*;
use adventuresim_core::character_proportions::BodyProportion;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.collapsing("Skeletal proportions", |ui| {
        for proportion in BodyProportion::ALL {
            let mut value = studio.recipe.proportions.get(proportion);
            if ui
                .add(
                    egui::Slider::new(&mut value, -proportion.limit()..=proportion.limit())
                        .text(proportion.label())
                        .fixed_decimals(2),
                )
                .changed()
            {
                studio
                    .recipe
                    .proportions
                    .set(proportion, value)
                    .expect("slider respects MHR limits");
                studio.dirty = true;
            }
        }
    });
}

pub(super) fn show_identity(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.horizontal(|ui| {
        for group in IdentityGroup::ALL {
            ui.selectable_value(&mut studio.selected, group, group.label());
        }
    });
    let selected = studio.selected;
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for index in selected.range() {
                let response = ui.add(
                    egui::Slider::new(&mut studio.recipe.identity[index], -3.0..=3.0)
                        .text(format!(
                            "{} {:02}",
                            selected.label(),
                            index - selected.range().start + 1
                        ))
                        .fixed_decimals(2),
                );
                studio.dirty |= response.changed();
            }
        });

    ui.collapsing("Expression laboratory", |ui| {
        ui.checkbox(
            &mut studio.show_expressions,
            "Show all 72 expression channels",
        );
        if studio.show_expressions {
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    let mut changed = false;
                    for (index, value) in studio.recipe.expression.iter_mut().enumerate() {
                        changed |= ui
                            .add(
                                egui::Slider::new(value, -1.0..=1.0)
                                    .text(format!("Expression {:02}", index + 1)),
                            )
                            .changed();
                    }
                    studio.dirty |= changed;
                });
        }
    });
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Randomize body").clicked() {
            studio.seed = studio.seed.wrapping_add(1);
            studio.recipe.proportions =
                adventuresim_core::character_proportions::CharacterProportions::from_character_id(
                    studio.seed,
                );
            let mut rng = StdRng::seed_from_u64(studio.seed);
            for value in &mut studio.recipe.identity {
                *value = rng.random_range(-1.35..=1.35);
            }
            studio.dirty = true;
        }
        if ui.button("Neutral").clicked() {
            studio.recipe.reset_body();
            studio.recipe.reset_face();
            studio.dirty = true;
        }
    });
}
