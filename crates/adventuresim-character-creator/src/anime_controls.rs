//! Solid or horizontally articulated breastplate construction controls.
use adventuresim_armor_model::{AnimeDesign, BreastplateConstruction, BreastplateDesign};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, design: &mut BreastplateDesign) -> bool {
    let original_construction = design.construction.clone();
    let original_gauge = design.wall_thickness;
    let mut articulated = matches!(design.construction, BreastplateConstruction::Anime(_));
    ui.horizontal(|ui| {
        ui.label("Torso construction");
        ui.selectable_value(&mut articulated, false, "Solid plate");
        ui.selectable_value(&mut articulated, true, "Anime lames");
    });
    if articulated != matches!(design.construction, BreastplateConstruction::Anime(_)) {
        design.construction = if articulated {
            BreastplateConstruction::Anime(AnimeDesign::default())
        } else {
            BreastplateConstruction::Solid
        };
    }
    let maximum_gauge = if articulated { 6 } else { 20 };
    design.wall_thickness.0 = design.wall_thickness.0.min(maximum_gauge);
    super::armor_controls::number(
        ui,
        &mut design.wall_thickness.0,
        1..=maximum_gauge,
        "Wall thickness (mm)",
    );
    if let BreastplateConstruction::Anime(shape) = &mut design.construction {
        let minimum_lift = 4.max(design.wall_thickness.0 * 2);
        shape.lap_lift.0 = shape.lap_lift.0.max(minimum_lift);
        ui.add(egui::Slider::new(&mut shape.lame_count, 3..=10).text("Lower torso lames"));
        for (value, range, label) in [
            (
                &mut shape.articulated_height.0,
                450..=950,
                "Articulated torso height",
            ),
            (&mut shape.overlap.0, 4..=15, "Course overlap (mm)"),
            (&mut shape.chevron_slope.0, 0..=500, "Front chevron slope"),
            (
                &mut shape.rear_chevron_slope.0,
                0..=500,
                "Rear chevron slope",
            ),
            (
                &mut shape.lap_lift.0,
                minimum_lift..=12,
                "Overlapping edge lift (mm)",
            ),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
    }
    design.construction != original_construction || design.wall_thickness != original_gauge
}
