//! Shared plate relief controls.
use adventuresim_armor_model::PlateFluting;
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, fluting: &mut Option<PlateFluting>) -> bool {
    let mut changed = false;
    let mut enabled = fluting.is_some();
    let fluting_toggle = ui.checkbox(&mut enabled, "Fluting");
    if fluting_toggle.changed() {
        *fluting = enabled.then(PlateFluting::default);
        changed = true;
    }
    if let Some(pattern) = fluting {
        changed |= ui
            .add(
                egui::Slider::new(&mut pattern.count.0, PlateFluting::COUNT_RANGE)
                    .text("Flute count"),
            )
            .changed();
        for (value, range, label, unit) in [
            (
                &mut pattern.width.0,
                PlateFluting::WIDTH_RANGE,
                "Flute width / spacing",
                " ‰",
            ),
            (
                &mut pattern.depth.0,
                PlateFluting::DEPTH_RANGE,
                "Flute depth",
                " mm",
            ),
            (
                &mut pattern.spread.0,
                PlateFluting::SPREAD_RANGE,
                "Pattern spread",
                " ‰",
            ),
            (
                &mut pattern.lower_spread.0,
                PlateFluting::LOWER_SPREAD_RANGE,
                "Lower spread / upper spread",
                " ‰",
            ),
        ] {
            changed |= ui
                .add(egui::Slider::new(value, range).text(label).suffix(unit))
                .changed();
        }
        let max_start = pattern.end.0 - pattern.fade.0 * 2;
        changed |= ui
            .add(
                egui::Slider::new(&mut pattern.start.0, PlateFluting::MIN_START..=max_start)
                    .text("Pattern start")
                    .suffix(" ‰"),
            )
            .changed();
        // Recompute after the start control so the end bound remains valid
        // even when both controls receive input in the same frame.
        let min_end = pattern.start.0 + pattern.fade.0 * 2;
        changed |= ui
            .add(
                egui::Slider::new(&mut pattern.end.0, min_end..=PlateFluting::MAX_END)
                    .text("Pattern end")
                    .suffix(" ‰"),
            )
            .changed();
        let max_fade = ((pattern.end.0 - pattern.start.0) / 2).min(*PlateFluting::FADE_RANGE.end());
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut pattern.fade.0,
                    *PlateFluting::FADE_RANGE.start()..=max_fade,
                )
                .text("Flute end taper")
                .suffix(" ‰"),
            )
            .changed();
    }
    changed
}
