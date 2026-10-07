//! The profile tab: a preset, and every knob of it that trades detail for
//! speed.

use eframe::egui::{self, Slider, Ui};
use fabelgeist_botany::viewer::profile::Profile;

/// What the preset menu shows for a profile turned away from every preset.
const CUSTOM: &str = "Custom";

/// Show the profile's controls; true when any of them changed it.
pub fn show(ui: &mut Ui, profile: &mut Profile) -> bool {
    let before = *profile;
    preset(ui, profile);
    growth(ui, profile);
    junctions(ui, profile);
    distance(ui, profile);
    detail(ui, profile);
    ui.strong("Ground");
    ui.checkbox(&mut profile.litter, "Litter under the canopy");
    *profile != before
}

fn preset(ui: &mut Ui, profile: &mut Profile) {
    let current = profile
        .preset()
        .map_or(CUSTOM, |index| Profile::PRESETS[index].0);
    egui::ComboBox::from_id_salt("preset")
        .selected_text(current)
        .show_ui(ui, |ui| {
            for (name, preset) in Profile::PRESETS {
                if ui.selectable_label(current == name, name).clicked() {
                    *profile = preset;
                }
            }
        });
}

fn growth(ui: &mut Ui, p: &mut Profile) {
    ui.strong("Growth");
    ui.checkbox(&mut p.continuous, "Continuous (between steps too)");
    ui.weak(format!("Step: {:.1} years", p.step_years));
    ui.add(
        Slider::new(&mut p.step_years, 0.1..=2.0)
            .step_by(0.1)
            .show_value(false),
    );
    ui.weak(format!(
        "View refresh: at most every {:.0} ms",
        p.refresh_ms
    ));
    ui.add(
        Slider::new(&mut p.refresh_ms, 16.0..=1000.0)
            .step_by(1.0)
            .show_value(false),
    );
}

fn junctions(ui: &mut Ui, p: &mut Profile) {
    ui.strong("Junctions");
    ui.checkbox(&mut p.fillets, "Smooth where branches meet");
    let mut millimetres = p.fillet_thinnest * 1e3;
    ui.weak(format!(
        "Fillet: {:.2} x radius, branches over {:.0} mm",
        p.fillet_size, millimetres
    ));
    ui.add(
        Slider::new(&mut p.fillet_size, 0.1..=1.5)
            .step_by(0.05)
            .text("size"),
    );
    ui.add(
        Slider::new(&mut millimetres, 1.0..=100.0)
            .step_by(1.0)
            .text("thinnest"),
    );
    p.fillet_thinnest = millimetres * 1e-3;
}

fn distance(ui: &mut Ui, p: &mut Profile) {
    ui.strong("Distance");
    ui.checkbox(&mut p.proxies, "Far groves as proxies");
    ui.weak(format!("Proxy error: {:.2} px", p.proxy_tolerance));
    ui.add(
        Slider::new(&mut p.proxy_tolerance, 0.25..=10.0)
            .step_by(0.25)
            .show_value(false),
    );
    ui.weak(format!(
        "Full detail over {:.0} px, middle over {:.0} px",
        p.full_pixels, p.middle_pixels
    ));
    ui.add(
        Slider::new(&mut p.full_pixels, 20.0..=800.0)
            .step_by(5.0)
            .text("full"),
    );
    ui.add(
        Slider::new(&mut p.middle_pixels, 2.0..=200.0)
            .step_by(1.0)
            .text("middle"),
    );
    p.middle_pixels = p.middle_pixels.min(p.full_pixels);
}

fn detail(ui: &mut Ui, p: &mut Profile) {
    ui.strong("Full detail");
    ui.add(Slider::new(&mut p.tube_sides, 3..=16).text("tube sides"));
    ui.add(Slider::new(&mut p.tube_spans, 1..=12).text("tube spans"));
    ui.add(Slider::new(&mut p.leaf_blades, 1..=16).text("blades a cluster"));
}
