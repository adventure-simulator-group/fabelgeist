//! Domain-separated corpse handling and permission attempt draws.
use super::StrategicCorpse;

pub(super) fn opening_quality(corpse: &StrategicCorpse, actor_id: u64, receipt_id: &str) -> u16 {
    fabelgeist_determinism::Seed::derive(
        corpse.id.as_bytes(),
        fabelgeist_determinism::StreamId::new("corpse.opening-quality"),
        &[
            &actor_id.to_le_bytes(),
            &corpse.revision.to_le_bytes(),
            receipt_id.as_bytes(),
        ],
    )
    .rng()
    .index(usize::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE) + 1) as u16
}

pub(super) fn permission(attempt_id: &str) -> f32 {
    fabelgeist_determinism::Seed::derive(
        attempt_id.as_bytes(),
        fabelgeist_determinism::StreamId::new("corpse.permission"),
        &[],
    )
    .rng()
    .index(usize::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE) + 1) as f32
        / f32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE)
}
