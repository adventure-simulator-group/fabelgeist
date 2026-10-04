//! Isolated checks within one initialized social action or assessment batch.
use super::PersonalityAxis;
use fabelgeist_determinism::{Seed, StreamId};

pub(super) fn testimony_assessment(seed: u64, proposition: &str) -> f32 {
    Seed::derive(
        &seed.to_le_bytes(),
        StreamId::new("social.testimony-assessment"),
        &[proposition.as_bytes()],
    )
    .rng()
    .inclusive_unit_f32()
}

pub(super) fn check(seed: u64, participants: [adventuresim_core::identity::CharacterId; 2]) -> f32 {
    StreamId::new("social.claim-challenge")
        .rng(seed, &participants.map(u64::from))
        .inclusive_unit_f32()
}

pub(super) fn casual_chat(seed: u64) -> fabelgeist_determinism::DeterministicRng {
    StreamId::new("social.casual-chat").rng(seed, &[])
}

pub(super) fn presentation(
    seed: u64,
    observer: adventuresim_core::identity::CharacterId,
    subject: adventuresim_core::identity::CharacterId,
) -> f32 {
    StreamId::new("social.presentation-contact")
        .rng(seed, &[u64::from(observer), u64::from(subject)])
        .inclusive_unit_f32()
}

pub(super) struct ActionDraws {
    seed: u64,
    actor: adventuresim_core::identity::CharacterId,
    target: adventuresim_core::identity::CharacterId,
}
impl ActionDraws {
    pub(super) fn new(
        seed: u64,
        actor: adventuresim_core::identity::CharacterId,
        target: adventuresim_core::identity::CharacterId,
    ) -> Self {
        Self {
            seed,
            actor,
            target,
        }
    }
    pub(super) fn resolution(&self) -> f32 {
        StreamId::new("social.action")
            .rng(self.seed, &[u64::from(self.actor), u64::from(self.target)])
            .inclusive_unit_f32()
    }
    pub(super) fn discovery(&self, axis: PersonalityAxis) -> f32 {
        StreamId::new("social.discovery")
            .rng(
                self.seed,
                &[u64::from(self.actor), u64::from(self.target), axis as u64],
            )
            .inclusive_unit_f32()
    }
}
