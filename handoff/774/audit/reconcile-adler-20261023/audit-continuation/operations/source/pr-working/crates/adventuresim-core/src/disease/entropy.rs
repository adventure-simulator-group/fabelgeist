//! Deterministic acquisition and severity entropy at strategic identity boundaries.
use super::*;

pub fn severity_seed(e: InfectionEpisode) -> Seed {
    StreamId::new("disease.severity")
        .rng(
            e.id.into(),
            &[e.character_id, e.disease_id as u64, e.contracted_at.get()],
        )
        .next_seed()
}

/// Minute-specific contact draws are independent of neighboring exposures.
pub fn contact_exposure_seed(
    target_id: u64,
    source_id: u64,
    source_episode_id: u64,
    minute: StrategicMinute,
) -> Seed {
    StreamId::new("disease.contact-exposure")
        .rng(
            target_id.into(),
            &[source_id, source_episode_id, minute.get()],
        )
        .next_seed()
}
