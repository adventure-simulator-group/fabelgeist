//! Departure refuses changed authority without erasing participant readiness.

use crate::strategic::PartyReadinessError;
use adventuresim_core::{identity::IdentityError, strategic_place::PlaceIdentityError};

#[derive(Debug)]
pub(crate) enum DepartureRevalidationError {
    PartyChanged,
    Interrupted,
    MemberLocationChanged,
    Readiness(PartyReadinessError),
    StoredSettlement(IdentityError),
    StoredCaseSite(PlaceIdentityError),
}

impl From<IdentityError> for DepartureRevalidationError {
    fn from(source: IdentityError) -> Self {
        Self::StoredSettlement(source)
    }
}
impl From<PlaceIdentityError> for DepartureRevalidationError {
    fn from(source: PlaceIdentityError) -> Self {
        Self::StoredCaseSite(source)
    }
}

impl From<PartyReadinessError> for DepartureRevalidationError {
    fn from(source: PartyReadinessError) -> Self {
        Self::Readiness(source)
    }
}

impl std::fmt::Display for DepartureRevalidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartyChanged => f.write_str("Party changed during departure synchronization"),
            Self::Interrupted => {
                f.write_str("Travel was interrupted while the party synchronized its clocks")
            }
            Self::MemberLocationChanged => {
                f.write_str("A party member changed location during departure synchronization")
            }
            Self::Readiness(source) => source.fmt(f),
            Self::StoredSettlement(source) => {
                write!(f, "Stored departure settlement is malformed: {source}")
            }
            Self::StoredCaseSite(source) => {
                write!(f, "Stored departure case site is malformed: {source}")
            }
        }
    }
}

impl std::error::Error for DepartureRevalidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Readiness(source) => Some(source),
            Self::StoredSettlement(source) => Some(source),
            Self::StoredCaseSite(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{CharacterReadinessError, readiness_error::ReadinessParticipant};
    use adventuresim_core::identity::CharacterId;
    use std::error::Error;

    #[test]
    fn departure_retains_refused_member_and_concrete_readiness_chain() {
        let member = CharacterId::from(u64::MAX);
        let participant =
            CharacterReadinessError::Incapacitated(ReadinessParticipant::PartyMember(member));
        let error = DepartureRevalidationError::from(PartyReadinessError::from(participant));
        let party = error
            .source()
            .unwrap()
            .downcast_ref::<PartyReadinessError>()
            .unwrap();
        assert!(matches!(
            party.source().unwrap().downcast_ref::<CharacterReadinessError>(),
            Some(CharacterReadinessError::Incapacitated(
                ReadinessParticipant::PartyMember(actual)
            )) if *actual == member
        ));
        assert_eq!(
            error.to_string(),
            "A party member is incapacitated and must recover before acting"
        );
    }
}
