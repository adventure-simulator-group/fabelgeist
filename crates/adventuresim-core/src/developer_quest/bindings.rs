//! Current persistent-NPC binding validation for authored pattern targets.
use super::*;

pub(super) fn validate_pattern_targets(
    definition: &DeveloperQuestDefinition,
    context: &DeveloperGenerationContext,
    candidates: &BTreeMap<u64, &qg::WitnessCandidate>,
    diagnostics: &mut Vec<DeveloperQuestDiagnostic>,
) {
    for (index, target) in definition.pattern_targets.iter().enumerate() {
        let matches_current =
            candidates
                .get(&target.resident_character_id)
                .is_some_and(|candidate| {
                    qg::pattern_target_matches(
                        target,
                        candidate,
                        context.base.settlement_id.as_str(),
                    )
                });
        if !matches_current {
            diagnostics.push(diagnostic(
                format!("pattern_targets.{index}"),
                "stale_pattern_target",
                "Pattern target is not the same current, persistent settlement NPC",
                DiagnosticTier::Structural,
            ));
        }
    }
}
