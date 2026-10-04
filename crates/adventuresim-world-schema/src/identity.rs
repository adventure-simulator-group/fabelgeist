//! Shared character identity for schema mathematics and strategic consumers.
//!
//! Identity admits the native storage width; existence, life and permission
//! remain authoritative policies of the consuming domain.

use serde::{Deserialize, Serialize};

/// Durable character identity; existence and permission require authority checks.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct CharacterId(u64);

impl From<u64> for CharacterId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<CharacterId> for u64 {
    fn from(value: CharacterId) -> Self {
        value.0
    }
}

impl std::fmt::Display for CharacterId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::CharacterId;

    #[test]
    fn character_identity_keeps_full_storage_width_and_strict_numeric_admission() {
        for raw in [0, 7, u64::MAX] {
            let character = CharacterId::from(raw);
            let wire = serde_json::to_string(&character).unwrap();
            assert_eq!(wire, raw.to_string());
            assert_eq!(character.to_string(), raw.to_string());
            assert_eq!(u64::from(character), raw);
            assert_eq!(
                serde_json::from_str::<CharacterId>(&wire).unwrap(),
                character
            );
        }
        for invalid in ["-1", "18446744073709551616", "1.5", "\"7\""] {
            assert!(serde_json::from_str::<CharacterId>(invalid).is_err());
        }
    }
}
