//! Source contracts complement pure errors; SDK transactions need DB tests.

#[test]
fn live_validation_keeps_readiness_encounters_and_leads_before_position() {
    let source = include_str!("live_prerequisites.rs");
    let terms = [
        "tracking_capability_chain_is_coherent(",
        "capability_has_live_support_reducer(",
        "require_party_ready(",
        "require_no_unresolved_encounter(",
        "party.camp_destination.is_some()",
        "living_party_member_ids(",
        "for member_id in &members",
    ];
    let mut previous = 0;
    for term in terms {
        let current = source.find(term).unwrap();
        assert!(current >= previous, "admission order for {term}");
        previous = current;
    }
    let predecessor = source
        .find("InvestigationAdmissionError::PredecessorMissing")
        .unwrap();
    let referral = source.find("lead_is_live_contact_referral").unwrap();
    let position = source
        .find("validate_action_position(ctx, actor, capability, kind)?")
        .unwrap();
    assert!(previous < predecessor && predecessor < referral && referral < position);
    assert!(source.contains("reducer_action_public_case_id(ctx, capability)"));
    assert!(source.contains("lead.case_id == observer_case_id"));
    assert!(source.contains("InvestigationAdmissionError::NoLiveContactReferral"));
}

#[test]
fn encounter_admission_reads_bound_challenges_before_choice_and_keeps_binding() {
    let source = include_str!("../../strategic/encounter_admission.rs");
    let check = source
        .split("pub(crate) fn require_no_unresolved_encounter")
        .nth(1)
        .unwrap()
        .split("pub(crate) fn require_character_no_unresolved_encounter")
        .next()
        .unwrap();
    assert!(check.find("let narrative_pending").unwrap() < check.find("let choice").unwrap());
    let compact = check.split_whitespace().collect::<String>();
    assert!(
        compact
            .contains("occurrence.is_open()&&party_at_bound_road_challenge(ctx,party,&occurrence)")
    );
    assert!(compact.contains("unresolved_encounter(ctx,party_id).is_some()"));
}

#[test]
fn referred_contact_position_races_are_coded_as_action_unavailable() {
    use super::InvestigationAdmissionError;
    use adventuresim_core::reducer_error::{ReducerErrorCode, parse_reducer_error};

    let source = include_str!("position.rs");
    for refusal in [
        "InvestigationAdmissionError::ContactPresenceMissing",
        "InvestigationAdmissionError::ContactElsewhere",
        "InvestigationAdmissionError::ContactNotPresent",
    ] {
        assert!(
            source.contains(refusal),
            "missing live contact refusal {refusal}"
        );
    }
    assert!(source.contains("npc_is_present(ctx, &presence, minute)"));
    let error = InvestigationAdmissionError::ContactNotPresent;
    assert_eq!(
        error.code(),
        Some(ReducerErrorCode::InvestigationActionUnavailable)
    );
    assert_eq!(parse_reducer_error(&error.to_string()), error.code());
}
