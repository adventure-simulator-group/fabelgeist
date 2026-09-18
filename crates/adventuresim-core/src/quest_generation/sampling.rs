//! Canonical candidate ordering and proportional integer-weighted draws.
use super::*;

pub(super) fn choose<T: Copy>(
    seed: u64,
    module: &str,
    relation: &str,
    candidates: &[Candidate<T>],
    trace: &mut Vec<FactorTrace>,
) -> Result<(T, Option<&'static str>), GenerationError> {
    if candidates.len() > MAX_SOLVER_CANDIDATES {
        return Err(GenerationError::CandidateLimit);
    }
    let mut ordered: Vec<_> = (0..candidates.len()).collect();
    canonicalize(candidates, &mut ordered)?;
    let mut accepted_indices = Vec::new();
    for index in ordered {
        let c = &candidates[index];
        let accepted = c.impossible.is_none() && c.weight.combined() > 0;
        trace.push(FactorTrace {
            module_id: ModuleId::new(module),
            relation_id: RelationId::new(relation),
            factor_ids: c.factors.iter().map(|f| FactorId::new(*f)).collect(),
            candidate_id: c.id.into(),
            plausibility: c.weight.plausibility,
            curation: c.weight.curation,
            accepted,
            hard_zero_reason: c.impossible.map(str::to_owned),
            required_bridge: c.bridge.map(BridgeId::new),
            decision: TraceDecision::Candidate,
        });
        if accepted {
            accepted_indices.push(index);
        }
    }
    if accepted_indices.is_empty() {
        return Err(GenerationError::NoCandidates {
            module: ModuleId::new(module),
            diagnostics: trace.clone(),
        });
    }
    let weights: Vec<_> = accepted_indices
        .iter()
        .map(|index| candidates[*index].weight.combined())
        .collect();
    let mut random = fabelgeist_determinism::Seed::derive(
        &seed.to_le_bytes(),
        fabelgeist_determinism::StreamId::new("quest.choice"),
        &[module.as_bytes(), relation.as_bytes()],
    )
    .rng();
    let selected = random
        .weighted_index(&weights)
        .map_err(GenerationError::Sampling)?;
    let candidate = &candidates[accepted_indices[selected]];
    Ok((candidate.value, candidate.bridge))
}

pub(super) fn weighted_order<T: Copy>(
    seed: u64,
    domain: &str,
    candidates: &[Candidate<T>],
) -> Result<Vec<usize>, GenerationError> {
    if candidates.len() > MAX_SOLVER_CANDIDATES {
        return Err(GenerationError::CandidateLimit);
    }
    let mut indices: Vec<_> = (0..candidates.len()).collect();
    canonicalize(candidates, &mut indices)?;
    indices.retain(|index| {
        let candidate = &candidates[*index];
        candidate.impossible.is_none() && candidate.weight.combined() > 0
    });
    if indices.is_empty() {
        return Ok(indices);
    }
    let weights: Vec<_> = indices
        .iter()
        .map(|index| candidates[*index].weight.combined())
        .collect();
    let mut random = fabelgeist_determinism::Seed::derive(
        &seed.to_le_bytes(),
        fabelgeist_determinism::StreamId::new("quest.order"),
        &[domain.as_bytes()],
    )
    .rng();
    Ok(random
        .weighted_order(&weights)
        .map_err(GenerationError::Sampling)?
        .into_iter()
        .map(|index| indices[index])
        .collect())
}

pub(super) fn canonicalize<T>(
    candidates: &[Candidate<T>],
    indices: &mut [usize],
) -> Result<(), GenerationError> {
    indices.sort_by_key(|index| candidates[*index].id);
    if indices
        .windows(2)
        .any(|pair| candidates[pair[0]].id == candidates[pair[1]].id)
    {
        return Err(GenerationError::InvalidManifest(vec![
            "candidate identities must be unique".into(),
        ]));
    }
    Ok(())
}

pub(super) fn canonical_context(
    context: &GenerationContext,
) -> Result<GenerationContext, GenerationError> {
    if context.witness_candidates.len() > MAX_SOLVER_CANDIDATES {
        return Err(GenerationError::CandidateLimit);
    }
    let mut canonical = context.clone();
    canonical
        .witness_candidates
        .sort_by_key(|candidate| candidate.resident_character_id);
    if canonical
        .witness_candidates
        .windows(2)
        .any(|pair| pair[0].resident_character_id == pair[1].resident_character_id)
    {
        return Err(GenerationError::InvalidManifest(vec![
            "witness identities must be unique".into(),
        ]));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate(id: &'static str, weight: u32) -> Candidate<&'static str> {
        Candidate {
            id,
            value: id,
            weight: Weight::new(weight, 1),
            bridge: None,
            impossible: None,
            factors: vec![],
        }
    }
    #[test]
    fn permutations_keep_weighted_draws_orders_and_diagnostics_identical() {
        let forward = [
            candidate("common", 3),
            candidate("rare", 1),
            candidate("excluded", 0),
        ];
        let reverse = [forward[2].clone(), forward[1].clone(), forward[0].clone()];
        let mut common = 0;
        for seed in 0..4096 {
            let mut left_trace = vec![];
            let mut right_trace = vec![];
            let left = choose(seed, "fixture", "relation", &forward, &mut left_trace).unwrap();
            let right = choose(seed, "fixture", "relation", &reverse, &mut right_trace).unwrap();
            assert_eq!(left, right);
            assert_eq!(left_trace, right_trace);
            common += usize::from(left.0 == "common");
            let left: Vec<_> = weighted_order(seed, "fixture", &forward)
                .unwrap()
                .into_iter()
                .map(|index| forward[index].id)
                .collect();
            let right: Vec<_> = weighted_order(seed, "fixture", &reverse)
                .unwrap()
                .into_iter()
                .map(|index| reverse[index].id)
                .collect();
            assert_eq!(left, right);
            assert_eq!(left.len(), 2);
            assert_ne!(left[0], left[1]);
            assert!(!left.contains(&"excluded"));
        }
        assert!(
            (2900..3250).contains(&common),
            "proportional 3:1 sampling: {common}"
        );
    }
    #[test]
    fn duplicate_identity_is_an_error_even_when_one_copy_is_excluded() {
        let candidates = [candidate("same", 1), candidate("same", 0)];
        assert!(matches!(
            weighted_order(7, "fixture", &candidates),
            Err(GenerationError::InvalidManifest(_))
        ));
    }
    #[test]
    fn overflowing_weights_return_the_shared_error() {
        let candidates =
            [candidate("a", u32::MAX), candidate("b", u32::MAX)].map(|mut candidate| {
                candidate.weight.curation = u32::MAX;
                candidate
            });
        assert!(matches!(
            weighted_order(7, "fixture", &candidates),
            Err(GenerationError::Sampling(
                fabelgeist_determinism::SamplingError::WeightOverflow
            ))
        ));
    }
}
