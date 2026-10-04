//! A party must settle either encounter authority before starting new activity.

use adventuresim_core::identity::{IdentityError, PartyId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EncounterChoiceState {
    Clear,
    AwaitingChoice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundRoadChallengeState {
    Clear,
    Open,
}

#[derive(Debug)]
pub(crate) enum PendingEncounterError {
    PartyIdentity(IdentityError),
    AwaitingChoice { party: PartyId },
    BoundRoadChallenge { party: PartyId },
}

impl From<IdentityError> for PendingEncounterError {
    fn from(source: IdentityError) -> Self {
        Self::PartyIdentity(source)
    }
}

impl std::fmt::Display for PendingEncounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartyIdentity(source) => source.fmt(f),
            Self::AwaitingChoice { .. } | Self::BoundRoadChallenge { .. } => {
                f.write_str("Resolve the strategic encounter before changing or continuing travel")
            }
        }
    }
}

impl std::error::Error for PendingEncounterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PartyIdentity(source) => Some(source),
            _ => None,
        }
    }
}

pub(crate) struct PartyEncounterAdmission {
    party: PartyId,
    choice: EncounterChoiceState,
    road_challenge: BoundRoadChallengeState,
}

impl PartyEncounterAdmission {
    pub(crate) fn new(
        party: PartyId,
        choice: EncounterChoiceState,
        road_challenge: BoundRoadChallengeState,
    ) -> Self {
        Self {
            party,
            choice,
            road_challenge,
        }
    }

    pub(crate) fn require_clear(self) -> Result<(), PendingEncounterError> {
        match (self.choice, self.road_challenge) {
            (EncounterChoiceState::AwaitingChoice, _) => {
                Err(PendingEncounterError::AwaitingChoice { party: self.party })
            }
            (_, BoundRoadChallengeState::Open) => {
                Err(PendingEncounterError::BoundRoadChallenge { party: self.party })
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn either_authority_refuses_activity_and_choice_retains_precedence() {
        let party = PartyId::try_new("party:one").unwrap();
        for (choice, challenge) in [
            (
                EncounterChoiceState::AwaitingChoice,
                BoundRoadChallengeState::Clear,
            ),
            (
                EncounterChoiceState::AwaitingChoice,
                BoundRoadChallengeState::Open,
            ),
        ] {
            let error = PartyEncounterAdmission::new(party.clone(), choice, challenge)
                .require_clear()
                .unwrap_err();
            assert!(
                matches!(&error, PendingEncounterError::AwaitingChoice { party: actual } if actual == &party)
            );
            assert_eq!(
                error.to_string(),
                "Resolve the strategic encounter before changing or continuing travel"
            );
        }
        let error = PartyEncounterAdmission::new(
            party.clone(),
            EncounterChoiceState::Clear,
            BoundRoadChallengeState::Open,
        )
        .require_clear()
        .unwrap_err();
        assert!(
            matches!(error, PendingEncounterError::BoundRoadChallenge { party: actual } if actual == party)
        );
        PartyEncounterAdmission::new(
            party,
            EncounterChoiceState::Clear,
            BoundRoadChallengeState::Clear,
        )
        .require_clear()
        .unwrap();
    }

    #[test]
    fn invalid_stored_party_key_keeps_its_identity_cause() {
        let cause = PartyId::try_new("party\ninvalid").unwrap_err();
        let error = PendingEncounterError::from(cause);
        assert_eq!(
            error.source().unwrap().downcast_ref::<IdentityError>(),
            Some(&cause)
        );
    }
}
