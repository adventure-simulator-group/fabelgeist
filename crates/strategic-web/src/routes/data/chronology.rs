//! Decisions at the selected character's personal frontier.

use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicMinute;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MutableCharacterAccess {
    Available,
    Future,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FrontierAlignment {
    Aligned,
    Different,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObservedLife {
    Alive,
    Dead,
    MissingCharacter,
}

/// A missing clock is distinct from a character whose known clock is ahead.
pub(super) struct PersonalFrontier {
    character: CharacterId,
    minute: Option<StrategicMinute>,
}

impl PersonalFrontier {
    pub(super) fn new(character: CharacterId, minute: Option<StrategicMinute>) -> Self {
        Self { character, minute }
    }

    pub(super) fn mutable_access_for(&self, observer: &Self) -> MutableCharacterAccess {
        if self.character == observer.character {
            return MutableCharacterAccess::Available;
        }
        match (self.minute, observer.minute) {
            (Some(subject), Some(observer)) if subject <= observer => {
                MutableCharacterAccess::Available
            }
            (Some(_), Some(_)) => MutableCharacterAccess::Future,
            _ => MutableCharacterAccess::Unknown,
        }
    }

    pub(super) fn alignment_with(&self, other: &Self) -> FrontierAlignment {
        match (self.minute, other.minute) {
            (Some(first), Some(second)) if first == second => FrontierAlignment::Aligned,
            (Some(_), Some(_)) => FrontierAlignment::Different,
            _ => FrontierAlignment::Unknown,
        }
    }
}

impl ObservedLife {
    pub(crate) fn from_character(character: &crate::spacetimedb::CharacterView) -> Self {
        match character.alive {
            true => Self::Alive,
            false => Self::Dead,
        }
    }

    /// Unavailable chronology cannot establish a death at the observer's date.
    pub(super) fn at_date(
        observer: Option<StrategicMinute>,
        death: Option<StrategicMinute>,
    ) -> Self {
        match (observer, death) {
            (Some(observer), Some(death)) if death <= observer => Self::Dead,
            _ => Self::Alive,
        }
    }
}

#[cfg(test)]
mod tests;
