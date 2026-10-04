//! Execution refusals retain admission and planning causes until reducer presentation.

use adventuresim_core::reducer_error::{ReducerErrorCode, coded_reducer_error};

#[derive(Debug)]
pub(crate) enum InvestigationExecutionError {
    Authority(crate::strategic::StrategicCharacterAuthorityError),
    AttemptConflict,
    PartyRequired,
    PartyMissing,
    LeaderApprovalRequired,
    SiteChanged,
    PrerequisitesChanged,
    UnsupportedEffect,
    IntervalMissing,
    EffectsChanged,
    ParticipantBoundary,
    PartyDisappeared,
    LeaderClockMissing,
    ActionDisappeared,
    ActionUnavailable,
    ActionStale,
    Admission(super::InvestigationAdmissionError),
    Living(crate::character::LivingCharacterError),
    ActorIdentity(adventuresim_core::physical_object::CustodyIdentityError),
    RightsQuestion(adventuresim_core::rights::RightsQuestionError),
    Commit(adventuresim_core::strategic_action::CommitRejection),
}
impl From<crate::strategic::StrategicCharacterAuthorityError> for InvestigationExecutionError {
    fn from(source: crate::strategic::StrategicCharacterAuthorityError) -> Self {
        Self::Authority(source)
    }
}
impl From<super::InvestigationAdmissionError> for InvestigationExecutionError {
    fn from(source: super::InvestigationAdmissionError) -> Self {
        Self::Admission(source)
    }
}
impl From<crate::character::LivingCharacterError> for InvestigationExecutionError {
    fn from(source: crate::character::LivingCharacterError) -> Self {
        Self::Living(source)
    }
}
impl From<adventuresim_core::physical_object::CustodyIdentityError>
    for InvestigationExecutionError
{
    fn from(source: adventuresim_core::physical_object::CustodyIdentityError) -> Self {
        Self::ActorIdentity(source)
    }
}
impl From<adventuresim_core::rights::RightsQuestionError> for InvestigationExecutionError {
    fn from(source: adventuresim_core::rights::RightsQuestionError) -> Self {
        Self::RightsQuestion(source)
    }
}
impl From<adventuresim_core::strategic_action::CommitRejection> for InvestigationExecutionError {
    fn from(source: adventuresim_core::strategic_action::CommitRejection) -> Self {
        Self::Commit(source)
    }
}
impl std::fmt::Display for InvestigationExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authority(source) => source.fmt(f),
            Self::AttemptConflict => {
                f.write_str("Investigation attempt id conflicts with an earlier action")
            }
            Self::PartyRequired => f.write_str("Must be in a party"),
            Self::PartyMissing => f.write_str("Party not found"),
            Self::LeaderApprovalRequired => f.write_str("Party leader approval is required"),
            Self::SiteChanged => f.write_str("Investigation site authority changed before commit"),
            Self::PrerequisitesChanged => {
                f.write_str("Investigation prerequisites changed before commit")
            }
            Self::UnsupportedEffect => {
                f.write_str("Investigation planner emitted an unsupported effect")
            }
            Self::IntervalMissing => {
                f.write_str("Investigation planner omitted the party interval")
            }
            Self::EffectsChanged => {
                f.write_str("Investigation planner effects do not match domain authority")
            }
            Self::ParticipantBoundary => {
                f.write_str("Investigation interval crossed a planned participant boundary")
            }
            Self::PartyDisappeared => f.write_str("Party disappeared after investigation interval"),
            Self::LeaderClockMissing => f.write_str("Party leader strategic clock disappeared"),
            Self::ActionDisappeared => f.write_str("Investigation action disappeared"),
            Self::ActionUnavailable => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationActionUnavailable,
                "Investigation action is unavailable",
            )),
            Self::ActionStale => f.write_str(&coded_reducer_error(
                ReducerErrorCode::InvestigationActionStale,
                "Investigation action is stale or belongs to another observer",
            )),
            Self::Admission(source) => source.fmt(f),
            Self::Living(source) => source.fmt(f),
            Self::ActorIdentity(_) => f.write_str("Investigation actor identity is malformed"),
            Self::RightsQuestion(_) => f.write_str("Investigation rights question is inconsistent"),
            Self::Commit(_) => f.write_str("Investigation authority changed before commit"),
        }
    }
}
impl std::error::Error for InvestigationExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authority(source) => Some(source),
            Self::Admission(source) => Some(source),
            Self::Living(source) => Some(source),
            Self::ActorIdentity(source) => Some(source),
            Self::RightsQuestion(source) => Some(source),
            Self::Commit(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "execution_error_tests.rs"]
mod tests;
