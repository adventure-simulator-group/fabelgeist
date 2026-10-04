use super::*;
use crate::spacetimedb::{DatabaseOperation, RemoteDatabaseFailure};
use std::error::Error as _;

#[test]
fn member_failures_keep_identity_context_and_existing_notices() {
    let identity = CharacterId::from(u64::MAX);
    for (error, notice) in [
        (
            PartyReadinessError::MissingMember(identity),
            "Party member not found",
        ),
        (
            PartyReadinessError::MissingCondition(identity),
            "Party member condition not found",
        ),
        (
            PartyReadinessError::Incapacitated(identity),
            "An incapacitated party member must recover before the party can act",
        ),
    ] {
        let (PartyReadinessError::MissingMember(id)
        | PartyReadinessError::MissingCondition(id)
        | PartyReadinessError::Incapacitated(id)) = &error
        else {
            panic!("expected member context");
        };
        assert_eq!(*id, identity);
        assert_eq!(error.to_string(), notice);
    }
}

#[test]
fn readiness_query_failure_retains_stage_identity_and_concrete_cause() {
    let identity = CharacterId::from(8);
    let source = SpacetimeError::Remote(RemoteDatabaseFailure::from_response(
        DatabaseOperation::Query,
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        Ok("condition unavailable".into()),
    ));
    let original = source.to_string();
    let error = PartyReadinessError::Query {
        stage: ReadinessQueryStage::MemberCondition,
        character: Some(identity),
        source,
    };
    assert!(matches!(
        error,
        PartyReadinessError::Query {
            stage: ReadinessQueryStage::MemberCondition,
            character: Some(id),
            ..
        } if id == identity
    ));
    assert_eq!(error.to_string(), original);
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<SpacetimeError>()
            .is_some()
    );
}
