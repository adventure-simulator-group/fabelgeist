//! Admission policy for the latest stored character life state.
//!
//! This policy does not infer life at an observer's historical frontier.

use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LivingCharacterError {
    Missing { character: CharacterId },
    Dead { character: CharacterId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StoredCharacterLifeState {
    Living,
    Dead,
}

impl From<bool> for StoredCharacterLifeState {
    fn from(alive: bool) -> Self {
        if alive { Self::Living } else { Self::Dead }
    }
}

impl StoredCharacterLifeState {
    pub(super) fn require_living(self, character: CharacterId) -> Result<(), LivingCharacterError> {
        match self {
            Self::Living => Ok(()),
            Self::Dead => Err(LivingCharacterError::Dead { character }),
        }
    }
}

impl std::fmt::Display for LivingCharacterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Missing { .. } => "Character not found",
            Self::Dead { .. } => "Dead characters cannot perform this action",
        })
    }
}

impl std::error::Error for LivingCharacterError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn stored_living_state_allows_action_without_restricting_identity_width() {
        for character in [CharacterId::from(0), CharacterId::from(u64::MAX)] {
            assert_eq!(
                StoredCharacterLifeState::from(true).require_living(character),
                Ok(())
            );
        }
    }

    #[test]
    fn stored_dead_state_retains_the_requested_character_and_refusal() {
        let character = CharacterId::from(u64::MAX);
        let error = StoredCharacterLifeState::from(false)
            .require_living(character)
            .unwrap_err();
        assert_eq!(error, LivingCharacterError::Dead { character });
        assert_eq!(
            error.to_string(),
            "Dead characters cannot perform this action"
        );
        assert!(error.source().is_none());
    }
}
