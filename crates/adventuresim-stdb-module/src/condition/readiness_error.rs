//! Readiness admission retains the requested population and concrete causes.

use super::StrategicConditionError;
use crate::character::LivingCharacterError;
use adventuresim_core::{identity::CharacterId, morale::IncapacitationStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadinessParticipant {
    Individual(CharacterId),
    PartyMember(CharacterId),
}

impl ReadinessParticipant {
    pub(super) fn require_status(
        self,
        status: IncapacitationStatus,
    ) -> Result<(), CharacterReadinessError> {
        match status {
            IncapacitationStatus::Ready | IncapacitationStatus::Staggered => Ok(()),
            IncapacitationStatus::Incapacitated => {
                Err(CharacterReadinessError::Incapacitated(self))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum CharacterReadinessError {
    Living {
        participant: ReadinessParticipant,
        source: LivingCharacterError,
    },
    CharacterProjection {
        character: CharacterId,
        source: StrategicConditionError,
    },
    PartyProjection {
        members: Vec<CharacterId>,
        source: StrategicConditionError,
    },
    Incapacitated(ReadinessParticipant),
}

impl std::fmt::Display for CharacterReadinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Living { source, .. } => source.fmt(f),
            Self::CharacterProjection { source, .. } | Self::PartyProjection { source, .. } => {
                source.fmt(f)
            }
            Self::Incapacitated(ReadinessParticipant::Individual(_)) => {
                f.write_str("Character is incapacitated and must recover before acting")
            }
            Self::Incapacitated(ReadinessParticipant::PartyMember(_)) => {
                f.write_str("A party member is incapacitated and must recover before acting")
            }
        }
    }
}

impl std::error::Error for CharacterReadinessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Living { source, .. } => Some(source),
            Self::CharacterProjection { source, .. } | Self::PartyProjection { source, .. } => {
                Some(source)
            }
            Self::Incapacitated(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn readiness_allows_staggered_participants_and_rejects_only_incapacitation() {
        for participant in [
            ReadinessParticipant::Individual(CharacterId::from(7)),
            ReadinessParticipant::PartyMember(CharacterId::from(17)),
        ] {
            for status in [IncapacitationStatus::Ready, IncapacitationStatus::Staggered] {
                participant.require_status(status).unwrap();
            }
            let error = participant
                .require_status(IncapacitationStatus::Incapacitated)
                .unwrap_err();
            assert!(
                matches!(error, CharacterReadinessError::Incapacitated(actual) if actual==participant)
            );
            let expected = match participant {
                ReadinessParticipant::Individual(_) => {
                    "Character is incapacitated and must recover before acting"
                }
                ReadinessParticipant::PartyMember(_) => {
                    "A party member is incapacitated and must recover before acting"
                }
            };
            assert_eq!(error.to_string(), expected);
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn living_refusal_retains_requested_role_and_concrete_missing_or_dead_cause() {
        for source in [
            LivingCharacterError::Missing {
                character: CharacterId::from(7),
            },
            LivingCharacterError::Dead {
                character: CharacterId::from(17),
            },
        ] {
            let character = match source {
                LivingCharacterError::Missing { character }
                | LivingCharacterError::Dead { character } => character,
            };
            let participant = ReadinessParticipant::PartyMember(character);
            let error = CharacterReadinessError::Living {
                participant,
                source,
            };
            assert!(
                matches!(error, CharacterReadinessError::Living { participant: actual, .. } if actual == participant)
            );
            assert_eq!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<LivingCharacterError>(),
                Some(&source)
            );
            assert_eq!(error.to_string(), source.to_string());
        }
    }

    #[test]
    fn character_projection_keeps_requested_actor_separate_from_a_failing_party_member() {
        let actor = CharacterId::from(7);
        let member = CharacterId::from(17);
        let error = CharacterReadinessError::CharacterProjection {
            character: actor,
            source: StrategicConditionError::Missing {
                character: member,
                component: super::super::error::ConditionComponent::Needs,
            },
        };
        assert!(
            matches!(&error, CharacterReadinessError::CharacterProjection { character, .. } if *character == actor)
        );
        assert!(
            matches!(error.source().unwrap().downcast_ref::<StrategicConditionError>(), Some(StrategicConditionError::Missing { character, component: super::super::error::ConditionComponent::Needs }) if *character == member)
        );
        assert_eq!(error.to_string(), "Character needs not found");
    }

    #[test]
    fn party_projection_failure_preserves_order_duplicates_and_nested_clock_cause() {
        let members = vec![
            CharacterId::from(17),
            CharacterId::from(7),
            CharacterId::from(17),
        ];
        let error = CharacterReadinessError::PartyProjection {
            members: members.clone(),
            source: StrategicConditionError::from(crate::time::WorldClockError::NotInitialized),
        };
        assert!(
            matches!(&error, CharacterReadinessError::PartyProjection { members: actual, .. } if actual == &members)
        );
        let condition = error
            .source()
            .unwrap()
            .downcast_ref::<StrategicConditionError>()
            .unwrap();
        assert_eq!(
            condition
                .source()
                .unwrap()
                .downcast_ref::<crate::time::WorldClockError>(),
            Some(&crate::time::WorldClockError::NotInitialized)
        );
        assert_eq!(error.to_string(), "World clock is not initialized");
    }
}
