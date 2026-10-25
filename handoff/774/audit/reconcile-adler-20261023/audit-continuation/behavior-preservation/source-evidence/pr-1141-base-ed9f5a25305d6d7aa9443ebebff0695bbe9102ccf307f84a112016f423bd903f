//! Stable draw ownership for recurring generated incidents.

use adventuresim_core::quest_generation::{
    Circumstance, GeneratedCase, GeneratedSite, GenerationContext, WitnessCandidate,
};
use fabelgeist_determinism::{Seed, StreamId};

pub(super) const WITNESS: StreamId = StreamId::new("incident.witness");
pub(super) const VICTIM: StreamId = StreamId::new("incident.victim");
pub(super) const SITE: StreamId = StreamId::new("incident.site");
pub(super) const CIRCUMSTANCE: StreamId = StreamId::new("incident.circumstance");
pub(super) const EVIDENCE: StreamId = StreamId::new("incident.evidence");

pub(super) struct IncidentDraws<'a> {
    case_id: &'a str,
    ordinal: u16,
}

impl<'a> IncidentDraws<'a> {
    pub(super) fn new(case_id: &'a str, ordinal: u16) -> Self {
        Self { case_id, ordinal }
    }

    pub(super) fn index(&self, stream: StreamId, length: usize) -> usize {
        self.rng(stream).index(length)
    }

    pub(super) fn word(&self, stream: StreamId) -> u64 {
        self.rng(stream).next_u64()
    }

    fn rng(&self, stream: StreamId) -> fabelgeist_determinism::DeterministicRng {
        Seed::derive(
            self.case_id.as_bytes(),
            stream,
            &[&self.ordinal.to_le_bytes()],
        )
        .rng()
    }
}

pub(super) fn candidates(context: &GenerationContext) -> Vec<&WitnessCandidate> {
    let mut candidates = context.witness_candidates.iter().collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| candidate.resident_character_id);
    candidates
}

pub(super) fn sites(case: &GeneratedCase) -> Vec<&GeneratedSite> {
    let mut sites = case.sites.iter().collect::<Vec<_>>();
    sites.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    sites
}

pub(super) fn circumstances(witness: &WitnessCandidate) -> Vec<Circumstance> {
    let mut circumstances = witness
        .allowed_circumstances
        .iter()
        .copied()
        .collect::<Vec<_>>();
    circumstances.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    circumstances
}
