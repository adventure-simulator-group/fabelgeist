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

pub(super) fn check(seed: u64, participants: [u64; 2]) -> f32 {
    StreamId::new("social.claim-challenge")
        .rng(seed, &participants)
        .inclusive_unit_f32()
}

pub(super) fn casual_chat(seed: u64) -> fabelgeist_determinism::DeterministicRng {
    StreamId::new("social.casual-chat").rng(seed, &[])
}

pub(super) fn presentation(seed: u64, observer: u64, subject: u64) -> f32 {
    StreamId::new("social.presentation-contact")
        .rng(seed, &[observer, subject])
        .inclusive_unit_f32()
}

pub(super) struct ActionDraws {
    seed: u64,
    actor: u64,
    target: u64,
}
impl ActionDraws {
    pub(super) fn new(seed: u64, actor: u64, target: u64) -> Self {
        Self {
            seed,
            actor,
            target,
        }
    }
    pub(super) fn resolution(&self) -> f32 {
        StreamId::new("social.action")
            .rng(self.seed, &[self.actor, self.target])
            .inclusive_unit_f32()
    }
    pub(super) fn discovery(&self, axis: PersonalityAxis) -> f32 {
        StreamId::new("social.discovery")
            .rng(self.seed, &[self.actor, self.target, axis as u64])
            .inclusive_unit_f32()
    }
}
