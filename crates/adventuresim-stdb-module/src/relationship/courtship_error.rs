//! Courtship admission failures and native reducer rejection encoding.

use super::{
    CourtshipRejection, CourtshipRejectionCode, FatherAdmissionError, TemporalScopeError,
    encode_courtship_rejection,
};
use crate::time::CharacterClockError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CourtshipPairError {
    Rejected(CourtshipRejection),
    InvalidState(String),
    CharacterClock(CharacterClockError),
    TemporalScope(TemporalScopeError),
    FatherApproval(FatherAdmissionError),
}

impl CourtshipPairError {
    pub(super) fn rejected(code: CourtshipRejectionCode, detail: impl Into<String>) -> Self {
        Self::Rejected(CourtshipRejection::new(code, detail))
    }

    pub(super) fn rejection_code(&self) -> Option<CourtshipRejectionCode> {
        match self {
            Self::Rejected(rejection) => Some(rejection.code),
            Self::FatherApproval(_) => Some(CourtshipRejectionCode::FatherApproval),
            _ => None,
        }
    }

    pub(crate) fn into_reducer_error(self) -> String {
        match self {
            Self::Rejected(rejection) => encode_courtship_rejection(&rejection),
            Self::FatherApproval(source) => encode_courtship_rejection(&CourtshipRejection::new(
                CourtshipRejectionCode::FatherApproval,
                source.to_string(),
            )),
            error => error.to_string(),
        }
    }
}

impl From<CharacterClockError> for CourtshipPairError {
    fn from(source: CharacterClockError) -> Self {
        Self::CharacterClock(source)
    }
}
impl From<TemporalScopeError> for CourtshipPairError {
    fn from(source: TemporalScopeError) -> Self {
        Self::TemporalScope(source)
    }
}
impl From<FatherAdmissionError> for CourtshipPairError {
    fn from(source: FatherAdmissionError) -> Self {
        Self::FatherApproval(source)
    }
}

impl From<String> for CourtshipPairError {
    fn from(detail: String) -> Self {
        Self::InvalidState(detail)
    }
}
impl From<&str> for CourtshipPairError {
    fn from(detail: &str) -> Self {
        Self::InvalidState(detail.to_owned())
    }
}

impl std::fmt::Display for CourtshipPairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(rejection) => f.write_str(&rejection.detail),
            Self::InvalidState(detail) => f.write_str(detail),
            Self::CharacterClock(source) => source.fmt(f),
            Self::TemporalScope(source) => source.fmt(f),
            Self::FatherApproval(source) => source.fmt(f),
        }
    }
}
impl std::error::Error for CourtshipPairError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CharacterClock(source) => Some(source),
            Self::TemporalScope(source) => Some(source),
            Self::FatherApproval(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::identity::CharacterId;
    use std::error::Error;

    #[test]
    fn chronology_failure_retains_the_scope_and_exact_character() {
        let character = CharacterId::from(17);
        let cause = CharacterClockError { character };
        let error = CourtshipPairError::from(TemporalScopeError::from(cause));
        assert_eq!(error.rejection_code(), None);
        let scope = error
            .source()
            .unwrap()
            .downcast_ref::<TemporalScopeError>()
            .unwrap();
        let clock = scope
            .source()
            .unwrap()
            .downcast_ref::<CharacterClockError>()
            .unwrap();
        assert_eq!(clock.character, character);
        assert_eq!(
            error.into_reducer_error(),
            "Character time record not found"
        );
    }

    #[test]
    fn father_rejection_keeps_its_code_context_and_nested_clock_until_encoding() {
        let child = CharacterId::from(7);
        let father = CharacterId::from(17);
        let source = FatherAdmissionError::Clock {
            child,
            requested: adventuresim_world_schema::calendar::StrategicMinute::new(100),
            source: CharacterClockError { character: father },
        };
        let error = CourtshipPairError::from(source);
        assert_eq!(
            error.rejection_code(),
            Some(CourtshipRejectionCode::FatherApproval)
        );
        let admission = error
            .source()
            .unwrap()
            .downcast_ref::<FatherAdmissionError>()
            .unwrap();
        assert_eq!(*admission, source);
        let clock = admission
            .source()
            .unwrap()
            .downcast_ref::<CharacterClockError>()
            .unwrap();
        assert_eq!(clock.character, father);
        assert_eq!(
            error.into_reducer_error(),
            "[courtship_rejection:father_approval] Character time record not found"
        );
    }
}
