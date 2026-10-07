//! The profile tab: how the simulation steps and how finely the forest is
//! built. The profile also holds branch fillets, grove proxies, screen
//! size bands and ground litter; the Bevy forest draws none of them, so
//! they are not offered here.

use bevy_egui::egui::{Slider, Ui};
use fabelgeist_botany::viewer::profile::Profile;

/// Show the profile's controls; true when any of them changed it.
pub fn show(ui: &mut Ui, profile: &mut Profile) -> bool {
    let before = *profile;
    growth(ui, profile);
    detail(ui, profile);
    *profile != before
}

/// The profile the Bevy forest draws: no fillets to blend and no proxies to
/// build, which this renderer does not draw.
pub fn drawable(mut profile: Profile) -> Profile {
    profile.fillets = false;
    profile.proxies = false;
    profile
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

fn detail(ui: &mut Ui, p: &mut Profile) {
    ui.strong("Detail");
    ui.add(Slider::new(&mut p.tube_sides, 3..=16).text("tube sides"));
    ui.add(Slider::new(&mut p.tube_spans, 1..=12).text("tube spans"));
    ui.add(Slider::new(&mut p.leaf_blades, 1..=16).text("blades a cluster"));
}
