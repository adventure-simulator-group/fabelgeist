//! A departure clock retains its requested members, party and clock cause.

use crate::time::WorldClockError;
use adventuresim_core::identity::{CharacterId, IdentityError, PartyId};

#[derive(Debug)]
pub(crate) enum DepartureClockError {
    NoLivingMembers,
    NoParty { members: Vec<CharacterId> },
    PartyMissing { party: PartyId },
    Uninitialized { party: PartyId },
    PartyIdentity(IdentityError),
    Clock(WorldClockError),
}

impl From<WorldClockError> for DepartureClockError {
    fn from(source: WorldClockError) -> Self {
        Self::Clock(source)
    }
}

impl std::fmt::Display for DepartureClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLivingMembers => f.write_str("Party has no living members"),
            Self::NoParty { .. } => f.write_str("Party members have no party"),
            Self::PartyMissing { .. } | Self::PartyIdentity(_) => f.write_str("Party not found"),
            Self::Uninitialized { .. } => f.write_str("Party wilderness clock was not initialized"),
            Self::Clock(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for DepartureClockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PartyIdentity(source) => Some(source),
            Self::Clock(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn missing_membership_retains_order_duplicates_and_full_width() {
        let members = vec![
            CharacterId::from(u64::MAX),
            CharacterId::from(0),
            CharacterId::from(u64::MAX),
        ];
        let error = DepartureClockError::NoParty {
            members: members.clone(),
        };
        assert!(
            matches!(&error, DepartureClockError::NoParty { members: actual } if *actual == members)
        );
        assert_eq!(error.to_string(), "Party members have no party");
        assert!(error.source().is_none());
    }

    #[test]
    fn official_clock_refusal_remains_a_concrete_cause() {
        let error = DepartureClockError::from(WorldClockError::NotInitialized);
        assert_eq!(
            error.source().unwrap().downcast_ref::<WorldClockError>(),
            Some(&WorldClockError::NotInitialized)
        );
        assert_eq!(error.to_string(), "World clock is not initialized");
    }
}
