use super::*;
use crate::condition::{CharacterReadinessError, readiness_error::ReadinessParticipant};
use adventuresim_core::{
    identity::CharacterId, physical_object::CustodyIdentityError,
    reducer_error::parse_reducer_error, rights::RightsQuestionError,
    strategic_action::CommitRejection,
};
use std::error::Error;

#[test]
fn execution_keeps_both_failed_authorities_and_the_simulation_cause() {
    let character = CharacterId::from(u64::MAX);
    let simulation = crate::simulation::SimulationCharacterAuthorityError::Unclaimed { character };
    let authority = crate::strategic::StrategicCharacterAuthorityError::Denied {
        character,
        gateway: crate::strategic::GatewayAdmissionError::Unregistered,
        simulation: Box::new(simulation),
    };
    let execution = InvestigationExecutionError::from(authority.clone());
    let cause = execution
        .source()
        .unwrap()
        .downcast_ref::<crate::strategic::StrategicCharacterAuthorityError>()
        .unwrap();
    assert_eq!(cause, &authority);
    assert_eq!(
        cause
            .source()
            .unwrap()
            .downcast_ref::<crate::simulation::SimulationCharacterAuthorityError>(),
        Some(&simulation)
    );
    assert_eq!(execution.to_string(), authority.to_string());
}

#[test]
fn execution_preserves_the_member_through_admission_and_party_causes() {
    let member = CharacterId::from(u64::MAX);
    let error = InvestigationExecutionError::from(super::super::InvestigationAdmissionError::from(
        crate::strategic::PartyReadinessError::from(CharacterReadinessError::Incapacitated(
            ReadinessParticipant::PartyMember(member),
        )),
    ));
    let admission = error
        .source()
        .unwrap()
        .downcast_ref::<super::super::InvestigationAdmissionError>()
        .unwrap();
    let party = admission
        .source()
        .unwrap()
        .downcast_ref::<crate::strategic::PartyReadinessError>()
        .unwrap();
    assert!(
        matches!(party.source().unwrap().downcast_ref::<CharacterReadinessError>(),
        Some(CharacterReadinessError::Incapacitated(ReadinessParticipant::PartyMember(actual))) if *actual == member)
    );
    assert_eq!(
        error.to_string(),
        "A party member is incapacitated and must recover before acting"
    );
}

#[test]
fn execution_keeps_safe_refusal_text_and_inspectable_planning_causes() {
    for cause in [
        CommitRejection::ForgedPlan,
        CommitRejection::IdempotencyConflict,
        CommitRejection::StaleSnapshot,
        CommitRejection::PrerequisitesChanged,
        CommitRejection::CalculationChanged,
    ] {
        let error = InvestigationExecutionError::from(cause);
        assert_eq!(
            error.source().unwrap().downcast_ref::<CommitRejection>(),
            Some(&cause)
        );
        assert_eq!(
            error.to_string(),
            "Investigation authority changed before commit"
        );
    }
    for cause in [
        RightsQuestionError::ResourceJurisdictionMismatch,
        RightsQuestionError::SelfContainment,
    ] {
        let error = InvestigationExecutionError::from(cause);
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<RightsQuestionError>(),
            Some(&cause)
        );
        assert_eq!(
            error.to_string(),
            "Investigation rights question is inconsistent"
        );
    }
    let cause = CustodyIdentityError::ZeroCharacterId;
    let error = InvestigationExecutionError::from(cause);
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<CustodyIdentityError>(),
        Some(&cause)
    );
    assert_eq!(
        error.to_string(),
        "Investigation actor identity is malformed"
    );
}

#[test]
fn unavailable_and_stale_execution_remain_distinct_transport_codes() {
    for (error, code) in [
        (
            InvestigationExecutionError::ActionUnavailable,
            ReducerErrorCode::InvestigationActionUnavailable,
        ),
        (
            InvestigationExecutionError::ActionStale,
            ReducerErrorCode::InvestigationActionStale,
        ),
    ] {
        assert_eq!(
            parse_reducer_error(&format!("transaction failed: {error}")),
            Some(code)
        );
    }
}

#[test]
fn execution_preserves_replay_revalidation_and_interrupted_write_order() {
    let source = include_str!("execution.rs");
    let replay = source.find("if let Some(attempt)").unwrap();
    let living = source.find("require_living_character").unwrap();
    let admission = source.find("validate_live_action_prerequisites").unwrap();
    let synchronization = source.find("synchronize_party_activity_time").unwrap();
    let revalidation = source.rfind("validate_live_action_prerequisites").unwrap();
    let commit = source.find("validate_commit").unwrap();
    let interval = source.find("advance_investigation_time").unwrap();
    let interruption = source.find("if !interval_completed").unwrap();
    let consequence = source.find("commit_action_consequence").unwrap();
    assert!(replay < living && living < admission && admission < synchronization);
    assert!(synchronization < revalidation && revalidation < commit && commit < interval);
    assert!(interval < interruption && interruption < consequence);
    let clipped = source
        .split("if !interval_completed")
        .nth(1)
        .unwrap()
        .split("if !permits_resolution")
        .next()
        .unwrap();
    assert!(clipped.contains("private_interrupted_action_resolution_json"));
    assert!(clipped.contains("investigation_action_attempt()"));
    assert!(clipped.contains("return Ok(())"));
    assert!(!source.contains("error.to_string()"));
    assert!(!source.contains("PartyIdentity(source).to_string()"));
    let native = include_str!("../actions.rs")
        .split("#[reducer]\npub fn perform_investigation_action(")
        .nth(1)
        .unwrap();
    assert!(native.contains(".map_err(|error: InvestigationExecutionError| error.to_string())"));
}
