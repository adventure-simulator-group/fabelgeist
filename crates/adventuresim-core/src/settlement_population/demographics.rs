use super::*;

const BUILDING_PLAN: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("settlement.building-plan");

/// Canonical city and business-demand seed for one settlement identity.
pub fn settlement_building_seed(settlement_id: &str) -> u64 {
    fabelgeist_determinism::Seed::derive(settlement_id.as_bytes(), BUILDING_PLAN, &[]).to_u64()
}

pub(super) fn choose_age(
    input: &GenerationInput,
    context: &str,
    candidates: &[RelationCandidate<AgeBand>],
) -> Result<(AgeBand, RelationDecision), String> {
    if let Some(age) = input.age {
        return Ok((
            age,
            RelationDecision {
                relation: PopulationRelation::AgeAtLocation.stable_id().into(),
                context: "household-plan".into(),
                decision: age.stable_id().into(),
                plausibility: 100,
                curation: 0,
                bridge: None,
            },
        ));
    }
    choose(
        &input.seed,
        PopulationRelation::AgeAtLocation,
        context,
        &input.available_bridges,
        candidates,
    )
}
