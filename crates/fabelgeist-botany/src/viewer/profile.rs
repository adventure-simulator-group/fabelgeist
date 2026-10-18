//! How the viewer trades detail for speed.
//!
//! A [`Profile`] holds every knob between the simulation and the screen:
//! how growth advances, what a snapshot packs, and how much of it is drawn.
//! The presets are starting points; any knob can be turned from there.

/// Every knob between the simulation and the screen.
use crate::units::StepDuration;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    // Growth.
    /// Plants grow between steps, a frame's worth at a time -- rather than a
    /// step's worth at once, when the step runs.
    pub continuous: bool,
    /// Years a step's regulation covers: light, vigour, budding and
    /// shedding, death and seeding (`EcosystemSettings::dt`).
    pub step_years: StepDuration,
    /// The view is refreshed at most this often, milliseconds -- and never
    /// more often than twice what packing a snapshot costs, so a large
    /// forest spends its time growing.
    pub refresh_ms: f32,

    // Junctions.
    /// Branches blend into one surface where they meet.
    pub fillets: bool,
    /// How far a fillet reaches, in radii of the branch it grows from.
    pub fillet_size: f32,
    /// Branches thinner than this radius, metres, grow none: their fillets
    /// would not show, and twigs are most of a forest.
    pub fillet_thinnest: f32,

    // Distance.
    /// Far-off groves are drawn as proxies standing in for their plants.
    pub proxies: bool,
    /// The error a proxy may show: pixels on screen, texels in the shadow
    /// map.
    pub proxy_tolerance: f32,
    /// A plant whose radius spans at least this many pixels is drawn in
    /// full detail ...
    pub full_pixels: f32,
    /// ... and at least this many, in middle detail; smaller, coarsest.
    pub middle_pixels: f32,

    // Detail up close.
    /// Around a branch tube in full detail, and along each of its segments.
    pub tube_sides: u32,
    pub tube_spans: u32,
    /// Blades per leaf cluster in full detail.
    pub leaf_blades: u32,

    // Ground.
    /// The ground under a canopy turns to its litter.
    pub litter: bool,
}

impl Profile {
    /// Growth in steps, plain junctions, every plant drawn on its own: a
    /// snapshot costs little and is packed once a step.
    pub const FAST: Profile = Profile {
        continuous: false,
        step_years: StepDuration::HALF_YEAR,
        refresh_ms: 33.0,
        fillets: false,
        fillet_size: 0.6,
        fillet_thinnest: 0.015,
        proxies: false,
        proxy_tolerance: 2.0,
        full_pixels: 220.0,
        middle_pixels: 40.0,
        tube_sides: 10,
        tube_spans: 6,
        leaf_blades: 10,
        litter: true,
    };

    /// Continuous growth, smooth junctions, far-off groves as proxies.
    pub const DETAILED: Profile = Profile {
        continuous: true,
        fillets: true,
        proxies: true,
        ..Profile::FAST
    };

    pub const PRESETS: [(&'static str, Profile); 2] =
        [("Fast", Profile::FAST), ("Detailed", Profile::DETAILED)];

    /// The preset this is, if it is one.
    pub fn preset(&self) -> Option<usize> {
        Self::PRESETS.iter().position(|(_, p)| p == self)
    }

    /// The preset called `name`, in any case.
    pub fn named(name: &str) -> Option<Profile> {
        Self::PRESETS
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, p)| *p)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Profile::FAST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turned_knob_leaves_the_preset() {
        assert_eq!(Profile::default().preset(), Some(0));
        assert_eq!(Profile::DETAILED.preset(), Some(1));
        assert_eq!(Profile::named("detailed"), Some(Profile::DETAILED));
        let tweaked = Profile {
            continuous: true,
            ..Profile::FAST
        };
        assert_eq!(tweaked.preset(), None);
    }
}
