//! Lifecycle event sampling keyed by defined event sequence.

pub(super) fn lifecycle_entropy(seed: u64, domain: &str, ordinal: u64) -> u16 {
    fabelgeist_determinism::Seed::derive(
        &seed.to_le_bytes(),
        fabelgeist_determinism::StreamId::new("simulation.lifecycle"),
        &[domain.as_bytes(), &ordinal.to_le_bytes()],
    )
    .rng()
    .index(usize::from(
        adventuresim_world_schema::BASIS_POINTS_PER_WHOLE,
    )) as u16
}
