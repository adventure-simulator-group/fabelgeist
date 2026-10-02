//! Separate distal plates beneath a knee or elbow cup.
use adventuresim_armor_model::JointExtension;
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, extension: &mut Option<JointExtension>) -> bool {
    let original = extension.clone();
    let mut enabled = extension.is_some();
    let toggle = ui.checkbox(&mut enabled, "Distal joint plates");
    if toggle.changed() {
        *extension = enabled.then(JointExtension::default);
    }
    if let Some(design) = extension {
        ui.add(egui::Slider::new(&mut design.lame_count, 1..=4).text("Distal plate count"));
        for (value, range, label) in [
            (&mut design.distal_taper.0, 700..=1100, "Distal plate taper"),
            (
                &mut design.length.0,
                40..=180,
                "Distal extension length (mm)",
            ),
            (
                &mut design.terminal_share.0,
                350..=850,
                "Terminal plate share",
            ),
            (
                &mut design.wrap.0,
                900..=1550,
                "Distal plate half-wrap (mrad)",
            ),
            (
                &mut design.hem_rounding.0,
                0..=25,
                "Distal hem rounding (mm)",
            ),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
    }
    *extension != original
}
