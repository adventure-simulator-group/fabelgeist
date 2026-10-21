//! Foliage opacity at the shader-instance packing boundary.
use super::*;

use crate::units::Years;
use fabelgeist_math::units::{Metres, Ratio};

pub(super) struct LeafOpacity {
    pub(super) tip: Ratio,
    pub(super) total: Ratio,
}

pub(super) struct LeafRetention {
    pub(super) years: Years,
    pub(super) twig_diameter: Metres,
}

/// Opacity after physiological development and chronological aging.
pub(super) fn leaf_fade(
    plant: &Plant,
    module: &crate::plant::Module,
    prototype: &crate::prototype::Prototype,
    n: usize,
    tip: &crate::plant::Node,
    retention: LeafRetention,
    species: &PlantType,
) -> LeafOpacity {
    // Opacity formulas consume native years and metres at shader packing.
    let foliage_years = retention.years.years();
    let twig = retention.twig_diameter.metres();
    // Foliage at every shoot tip, along the last few years' growth
    // behind them, and on thin twigs however old. Each goes
    // gradually: a tip's as the shoot that buds from it takes over,
    // young growth's over its last year, a twig's as it comes to feed
    // more shoots -- its sapwood, which grows with them smoothly --
    // and a species that sheds its leaves by age (a palm's fronds)
    // over the last half year.
    let slot = prototype.terminal_slot[n];
    let tipness = if !prototype.children[n].is_empty() {
        0.0
    } else if slot == NONE || module.children[slot as usize].is_none() {
        1.0
    } else {
        module.children[slot as usize].map_or(1.0, |child| {
            let shoot = &plant.modules[child.index()];
            1.0 - smoothstep(0.0, 0.5, shoot.age.stage())
        })
    };
    let years = plant.age.years() - module.born.years();
    let youth = 1.0 - smoothstep(foliage_years - 1.0, foliage_years, years);
    let thin = 1.0 - smoothstep(0.75 * twig, twig, tip.sapwood);
    let kept = species
        .leaf_years
        .map_or(1.0, |limit| 1.0 - smoothstep(limit - 0.5, limit, years));
    let fade = tipness.max(youth).max(thin) * kept;
    LeafOpacity {
        tip: Ratio::from_ratio(tipness),
        total: Ratio::from_ratio(fade),
    }
}
