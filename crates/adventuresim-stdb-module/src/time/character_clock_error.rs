//! Missing authoritative personal-clock context, retained through consumers.

use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CharacterClockError {
    pub(crate) character: CharacterId,
}

impl std::fmt::Display for CharacterClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Character time record not found")
    }
}

impl std::error::Error for CharacterClockError {}
