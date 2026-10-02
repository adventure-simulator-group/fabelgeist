//! Horizontal visor folds, independently selectable from decorative fluting.
use adventuresim_armor_model::VisorBellows;
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, bellows: &mut Option<VisorBellows>) -> bool {
    let original = *bellows;
    let mut enabled = bellows.is_some();
    let toggle = ui.checkbox(&mut enabled, "Bellows visor folds");
    if toggle.changed() {
        *bellows = enabled.then(VisorBellows::default);
    }
    if let Some(design) = bellows {
        ui.add(egui::Slider::new(&mut design.count, 1..=5).text("Horizontal fold count"));
        for (value, range, label) in [
            (&mut design.depth.0, 2..=14, "Fold depth (mm)"),
            (
                &mut design.start.0,
                350..=550,
                "Folded interval start from brow",
            ),
            (
                &mut design.end.0,
                800..=950,
                "Folded interval end from brow",
            ),
            (&mut design.sharpness.0, 0..=1000, "Fold sharpness"),
            (
                &mut design.cheek_rise.0,
                0..=35,
                "Fold rise toward cheeks (mm)",
            ),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
    }
    *bellows != original
}
