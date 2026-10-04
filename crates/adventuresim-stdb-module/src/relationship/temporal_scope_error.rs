//! Concrete participant and clock failures for relationship chronology.

use crate::time::{CharacterClockError, WorldClockError};
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TemporalScopeError {
    CharacterClock(CharacterClockError),
    OfficialClock(WorldClockError),
    TargetNotNpcPolicy {
        actor: CharacterId,
        target: CharacterId,
    },
    MissingParticipant {
        actor: CharacterId,
        participant: CharacterId,
    },
}

impl From<CharacterClockError> for TemporalScopeError {
    fn from(source: CharacterClockError) -> Self {
        Self::CharacterClock(source)
    }
}
impl From<WorldClockError> for TemporalScopeError {
    fn from(source: WorldClockError) -> Self {
        Self::OfficialClock(source)
    }
}
impl std::fmt::Display for TemporalScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CharacterClock(source) => source.fmt(f),
            Self::OfficialClock(source) => source.fmt(f),
            Self::TargetNotNpcPolicy { .. } => {
                f.write_str("NPC-canonical scope requires an NPC-policy character")
            }
            Self::MissingParticipant { .. } => {
                f.write_str("Exclusive scope requires an existing second participant")
            }
        }
    }
}
impl std::error::Error for TemporalScopeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CharacterClock(source) => Some(source),
            Self::OfficialClock(source) => Some(source),
            _ => None,
        }
    }
}
