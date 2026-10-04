//! Investigation admission preserves stable refusals and concrete lower causes.

use adventuresim_core::identity::IdentityError;
use adventuresim_core::investigation_action::ParseInvestigationActionKindError;
use adventuresim_core::reducer_error::{ReducerErrorCode, coded_reducer_error};

#[derive(Debug)]
pub(crate) enum InvestigationAdmissionError {
    CohortCaseMismatch,
    AreaMissing,
    OutsideSearchArea,
    SiteRequired,
    UnsupportedJournalRoute,
    PartyMissing,
    JourneyOrCamp,
    InsufficientMembers {
        members: Vec<adventuresim_core::identity::CharacterId>,
        kind: adventuresim_core::investigation_action::InvestigationActionKind,
    },
    MemberMissing {
        member: adventuresim_core::identity::CharacterId,
    },
    MembersSeparated {
        member: adventuresim_core::identity::CharacterId,
    },
    PredecessorMissing,
    PredecessorIncomplete,
    ObserverCaseMissing,
    NoLiveContactReferral,
    NoApproximateDestination,
    NoTrackSource,
    InvalidRoute,
    ContactPresenceMissing,
    ContactElsewhere,
    ContactNotPresent,
    CohortAuthorityMissing,
    CohortTargetUnavailable,
    CohortMoved,
    PartyReadiness(crate::strategic::PartyReadinessError),
    PendingEncounter(crate::strategic::PendingEncounterError),
    Method(ParseInvestigationActionKindError),
    PartyIdentity(IdentityError),
    ContactIdentity(std::num::ParseIntError),
}
impl From<crate::strategic::PartyReadinessError> for InvestigationAdmissionError {
    fn from(source: crate::strategic::PartyReadinessError) -> Self {
        Self::PartyReadiness(source)
    }
}
impl From<crate::strategic::PendingEncounterError> for InvestigationAdmissionError {
    fn from(source: crate::strategic::PendingEncounterError) -> Self {
        Self::PendingEncounter(source)
    }
}
impl From<ParseInvestigationActionKindError> for InvestigationAdmissionError {
    fn from(source: ParseInvestigationActionKindError) -> Self {
        Self::Method(source)
    }
}
impl From<IdentityError> for InvestigationAdmissionError {
    fn from(source: IdentityError) -> Self {
        Self::PartyIdentity(source)
    }
}
impl From<std::num::ParseIntError> for InvestigationAdmissionError {
    fn from(source: std::num::ParseIntError) -> Self {
        Self::ContactIdentity(source)
    }
}
impl InvestigationAdmissionError {
    pub(crate) fn code(&self) -> Option<ReducerErrorCode> {
        match self {
            Self::InvalidRoute => Some(ReducerErrorCode::InvestigationRouteInvalid),
            Self::ContactPresenceMissing => Some(ReducerErrorCode::InvestigationActionUnavailable),
            Self::ContactElsewhere => Some(ReducerErrorCode::InvestigationActionUnavailable),
            Self::ContactNotPresent => Some(ReducerErrorCode::InvestigationActionUnavailable),
            Self::CohortAuthorityMissing => Some(ReducerErrorCode::VictimCohortStateChanged),
            Self::CohortTargetUnavailable => Some(ReducerErrorCode::VictimCohortStateChanged),
            Self::CohortMoved => Some(ReducerErrorCode::VictimCohortStateChanged),
            _ => None,
        }
    }
}
impl std::fmt::Display for InvestigationAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CohortCaseMismatch => f.write_str("Victim cohort belongs to another case"),
            Self::AreaMissing => f.write_str("Investigation area no longer exists"),
            Self::OutsideSearchArea => {
                f.write_str("The party is not near the approximate search area")
            }
            Self::SiteRequired => {
                f.write_str("The party must occupy the action's authoritative site")
            }
            Self::UnsupportedJournalRoute => {
                f.write_str("The current journal no longer supports this investigation route")
            }
            Self::PartyMissing => f.write_str("Party not found"),
            Self::JourneyOrCamp => {
                f.write_str("Investigation cannot begin during a journey or camp")
            }
            Self::InsufficientMembers { .. } => {
                f.write_str("Not enough living party members for this action")
            }
            Self::MemberMissing { .. } => f.write_str("Party member no longer exists"),
            Self::MembersSeparated { .. } => {
                f.write_str("Every living party member must be co-located")
            }
            Self::PredecessorMissing => f.write_str("Required investigation lead no longer exists"),
            Self::PredecessorIncomplete => {
                f.write_str("The preceding investigation lead is not complete")
            }
            Self::ObserverCaseMissing => {
                f.write_str("Investigation action has no observer-safe case binding")
            }
            Self::NoLiveContactReferral => {
                f.write_str("No live witness referral supports this action")
            }
            Self::NoApproximateDestination => {
                f.write_str("No current approximate destination supports this action")
            }
            Self::NoTrackSource => {
                f.write_str("No authoritative track source supports this action")
            }
            Self::InvalidRoute => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationRouteInvalid,
                "Investigation track origin no longer matches the projected route",
            )),
            Self::ContactPresenceMissing => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationActionUnavailable,
                "Referred contact no longer has an authoritative presence",
            )),
            Self::ContactElsewhere => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationActionUnavailable,
                "The referred contact is in another settlement",
            )),
            Self::ContactNotPresent => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationActionUnavailable,
                "The referred contact is not currently present",
            )),
            Self::CohortAuthorityMissing => f.write_str(&coded_reducer_error(
                ReducerErrorCode::VictimCohortStateChanged,
                "Victim cohort authority no longer exists",
            )),
            Self::CohortTargetUnavailable => f.write_str(&coded_reducer_error(
                ReducerErrorCode::VictimCohortStateChanged,
                "Victim cohort target is unavailable",
            )),
            Self::CohortMoved => f.write_str(&coded_reducer_error(
                ReducerErrorCode::VictimCohortStateChanged,
                "Victim cohort target moved from the learned location",
            )),
            Self::PartyReadiness(source) => source.fmt(f),
            Self::PendingEncounter(source) => source.fmt(f),
            Self::Method(source) => source.fmt(f),
            Self::PartyIdentity(source) => source.fmt(f),
            Self::ContactIdentity(_) => f.write_str("Referred contact identity is invalid"),
        }
    }
}
impl std::error::Error for InvestigationAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PartyReadiness(source) => Some(source),
            Self::PendingEncounter(source) => Some(source),
            Self::Method(source) => Some(source),
            Self::PartyIdentity(source) => Some(source),
            Self::ContactIdentity(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{CharacterReadinessError, readiness_error::ReadinessParticipant};
    use adventuresim_core::{identity::CharacterId, reducer_error::parse_reducer_error};
    use std::error::Error;

    #[test]
    fn admission_preserves_incapacitated_member_through_the_party_cause() {
        let member = CharacterId::from(u64::MAX);
        let error = InvestigationAdmissionError::from(crate::strategic::PartyReadinessError::from(
            CharacterReadinessError::Incapacitated(ReadinessParticipant::PartyMember(member)),
        ));
        let party = error
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
        assert_eq!(error.code(), None);
    }

    #[test]
    fn position_refusals_keep_machine_codes_through_transport_decoration() {
        for (error, expected) in [
            (
                InvestigationAdmissionError::ContactPresenceMissing,
                ReducerErrorCode::InvestigationActionUnavailable,
            ),
            (
                InvestigationAdmissionError::ContactElsewhere,
                ReducerErrorCode::InvestigationActionUnavailable,
            ),
            (
                InvestigationAdmissionError::ContactNotPresent,
                ReducerErrorCode::InvestigationActionUnavailable,
            ),
            (
                InvestigationAdmissionError::CohortAuthorityMissing,
                ReducerErrorCode::VictimCohortStateChanged,
            ),
            (
                InvestigationAdmissionError::CohortTargetUnavailable,
                ReducerErrorCode::VictimCohortStateChanged,
            ),
            (
                InvestigationAdmissionError::CohortMoved,
                ReducerErrorCode::VictimCohortStateChanged,
            ),
            (
                InvestigationAdmissionError::InvalidRoute,
                ReducerErrorCode::InvestigationRouteInvalid,
            ),
        ] {
            assert_eq!(error.code(), Some(expected));
            assert_eq!(
                parse_reducer_error(&format!("operation failed: {error}")),
                Some(expected)
            );
        }
    }

    #[test]
    fn member_refusals_retain_order_duplicates_and_full_width_identity() {
        use adventuresim_core::investigation_action::InvestigationActionKind;
        let member = CharacterId::from(u64::MAX);
        let members = vec![member, CharacterId::from(0), member];
        let error = InvestigationAdmissionError::InsufficientMembers {
            members: members.clone(),
            kind: InvestigationActionKind::Patrol,
        };
        assert_eq!(
            error.to_string(),
            "Not enough living party members for this action"
        );
        assert!(
            matches!(error, InvestigationAdmissionError::InsufficientMembers {
            members: actual, kind: InvestigationActionKind::Patrol,
        } if actual == members)
        );
        for error in [
            InvestigationAdmissionError::MemberMissing { member },
            InvestigationAdmissionError::MembersSeparated { member },
        ] {
            assert_eq!(error.code(), None);
            assert!(error.source().is_none());
            assert!(matches!(error,
                InvestigationAdmissionError::MemberMissing { member: actual }
                | InvestigationAdmissionError::MembersSeparated { member: actual }
                if actual == member));
        }
    }

    #[test]
    fn invalid_native_contact_identity_retains_its_parse_cause() {
        let source = "not-a-character".parse::<u64>().unwrap_err();
        let error = InvestigationAdmissionError::ContactIdentity(source);
        assert!(error.source().unwrap().is::<std::num::ParseIntError>());
        assert_eq!(error.to_string(), "Referred contact identity is invalid");
    }
}

#[cfg(test)]
#[path = "admission_source_tests.rs"]
mod source_tests;
