//! The profile's settings in the menu: how the simulation steps, and how
//! finely the forest is built. The profile also holds branch fillets,
//! grove proxies, screen size bands and ground litter; the Bevy forest draws
//! none of them, so they are not offered here.

use bevy_egui::egui::{Slider, Ui};
use fabelgeist_botany::viewer::profile::Profile;

use crate::widgets;

/// The profile the Bevy forest draws: no fillets to blend and no proxies to
/// build, which this renderer does not draw.
pub fn drawable(mut profile: Profile) -> Profile {
    profile.fillets = false;
    profile.proxies = false;
    profile
}

/// How the simulation steps; true when it changed.
pub fn growth(ui: &mut Ui, p: &mut Profile) -> bool {
    let before = *p;
    widgets::switch(ui, &mut p.continuous, "Grow between steps too");
    let mut step_years = p.step_years.duration().years();
    widgets::setting(
        ui,
        "Step",
        format!("{step_years:.1} years"),
        Slider::new(&mut step_years, 0.1..=2.0).step_by(0.1),
    );
    if let Ok(duration) = fabelgeist_botany::units::StepDuration::try_new(
        fabelgeist_botany::units::Years::from_years(step_years),
    ) {
        p.step_years = duration;
    }
    widgets::setting(
        ui,
        "View refresh",
        format!("every {:.0} ms", p.refresh_ms),
        Slider::new(&mut p.refresh_ms, 16.0..=1000.0).step_by(1.0),
    );
    *p != before
}

/// How finely branches and leaves are built; true when it changed.
pub fn detail(ui: &mut Ui, p: &mut Profile) -> bool {
    let before = *p;
    widgets::setting(
        ui,
        "Branch roundness",
        format!("{} sides", p.tube_sides),
        Slider::new(&mut p.tube_sides, 3..=16),
    );
    widgets::setting(
        ui,
        "Branch smoothness",
        format!("{} spans", p.tube_spans),
        Slider::new(&mut p.tube_spans, 1..=12),
    );
    widgets::setting(
        ui,
        "Leaf fullness",
        format!("{} blades", p.leaf_blades),
        Slider::new(&mut p.leaf_blades, 1..=16),
    );
    *p != before
}
