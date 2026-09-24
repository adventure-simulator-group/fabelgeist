//! Deterministic managed land estimates where imported coverage is absent.
use super::*;

pub(super) fn fallback_profile(settlement: &ElevatedSettlementDraft) -> LandUseProfile {
    let seed = settlement.settlement.source_node_id;
    let cropland = 1_500
        + fabelgeist_determinism::StreamId::new("world.fallback-cropland")
            .rng(seed, &[])
            .index(1_501) as u16;
    let grazing = 1_000
        + fabelgeist_determinism::StreamId::new("world.fallback-grazing")
            .rng(seed, &[])
            .index(1_501) as u16;
    let built_up = (settlement.settlement.population_level.max(1) as u16) * 20;
    let natural = BASIS_POINTS_PER_WHOLE - cropland - grazing - built_up;
    LandUseProfile::new(
        LandUseFraction::new(cropland).unwrap(),
        LandUseFraction::new(grazing).unwrap(),
        LandUseFraction::new(built_up).unwrap(),
        LandUseFraction::new(natural).unwrap(),
    )
    .unwrap()
}
