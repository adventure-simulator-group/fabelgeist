//! Controls for long wrapping tassets and their detachable lower section.
use adventuresim_armor_model::WrappedTassetDesign;
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, shape: &mut WrappedTassetDesign, count: u8) -> bool {
    let original = *shape;
    let maximum_break = count.saturating_sub(1);
    shape.section_break = shape.section_break.min(maximum_break);
    for (value, range, label) in [
        (&mut shape.inner_wrap.0, 150..=450, "Inner thigh return"),
        (&mut shape.outer_wrap.0, 450..=750, "Outer thigh return"),
        (
            &mut shape.inner_gap.0,
            0..=80,
            "Half-width between tassets (mm)",
        ),
        (
            &mut shape.upper_edge_slope.0,
            0..=500,
            "Suspension chevron slope",
        ),
        (&mut shape.knee_reach.0, 650..=1100, "Reach toward knee"),
        (
            &mut shape.inner_cutaway.0,
            0..=50,
            "Inner upper cutaway (mm)",
        ),
        (
            &mut shape.hem_rounding.0,
            0..=35,
            "Hem corner rounding (mm)",
        ),
        (&mut shape.section_gap.0, 0..=8, "Section separation (mm)"),
    ] {
        super::armor_controls::number(ui, value, range, label);
    }
    ui.add(
        egui::Slider::new(&mut shape.section_break, 0..=maximum_break)
            .text("Lower section begins after course (0: unsplit)"),
    );
    *shape != original
}
