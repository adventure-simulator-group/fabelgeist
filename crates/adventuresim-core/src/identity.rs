//! Strategic identity values, independent of storage and tactical ECS state.
//!
//! An identity does not prove existence, custody, or permission. Storage and
//! transport adapters construct these values; grants retain their own proofs.
//!
//! ```compile_fail
//! use adventuresim_core::identity::{CharacterId, InventoryItemId};
//! fn select_character(_: CharacterId) {}
//! select_character(InventoryItemId::new(7));
//! ```
//!
//! A lot retains its identity when its carried row changes:
//!
//! ```compile_fail
//! use adventuresim_core::identity::{FoodLotId, InventoryItemId};
//! fn inspect_food_lot(_: FoodLotId) {}
//! inspect_food_lot(InventoryItemId::new(7));
//! ```
//!
//! Episode history cannot be addressed with a character identity:
//!
//! ```compile_fail
//! use adventuresim_core::identity::{CharacterId, InfectionEpisodeId};
//! fn inspect_episode(_: InfectionEpisodeId) {}
//! inspect_episode(CharacterId::from(7));
//! ```

use serde::{Deserialize, Serialize};

macro_rules! numeric_identity {
    ($name:ident, $documentation:literal) => {
        #[doc = $documentation]
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            Eq,
            Hash,
            Ord,
            PartialEq,
            PartialOrd,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.get()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

pub use adventuresim_world_schema::identity::CharacterId;

numeric_identity!(
    InventoryItemId,
    "Inventory-row identity, distinct from a physical object's identity."
);
numeric_identity!(
    FoodLotId,
    "Food-material lot identity, distinct from its carried inventory binding."
);
numeric_identity!(
    InfectionEpisodeId,
    "Infection-episode identity, distinct from the affected character or disease."
);

#[cfg(test)]
mod tests {
    #[test]
    fn serialized_numeric_seeds_and_exact_text_keys_keep_their_distinct_meaning() {
        use fabelgeist_determinism::{Seed, SeedKey, StreamId};
        let raw = 42_u64;
        let seed = Seed::from_u64(raw);
        assert_eq!(serde_json::to_string(&seed).unwrap(), "42");
        assert_eq!(serde_json::from_str::<Seed>("42").unwrap(), seed);
        let key: SeedKey = "42".into();
        assert_eq!(serde_json::to_string(&key).unwrap(), "\"42\"");
        let stream = StreamId::new("identity.seed-test");
        assert_eq!(
            key.derive(stream, &[b"a", b"bc"]),
            Seed::derive(b"42", stream, &[b"a", b"bc"])
        );
        assert_ne!(
            key.derive(stream, &[b"a", b"bc"]),
            key.derive(stream, &[b"ab", b"c"])
        );
        assert_ne!(key.derive(stream, &[]), seed.child(stream, &[]));
    }

    #[test]
    fn decoded_text_identities_reject_noncanonical_values() {
        for invalid in ["", " leading", "trailing ", "control\n", &"x".repeat(257)] {
            let json = serde_json::to_string(invalid).unwrap();
            assert!(serde_json::from_str::<super::PartyId>(&json).is_err());
            assert!(serde_json::from_str::<super::SettlementId>(&json).is_err());
        }
        let json = "\"party:one\"";
        let id: super::PartyId = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), json);
    }

    use super::*;

    #[test]
    fn transport_keeps_numeric_identity_representation() {
        let character: CharacterId = serde_json::from_str("42").unwrap();
        assert_eq!(u64::from(character), 42);
        assert_eq!(serde_json::to_string(&character).unwrap(), "42");
        assert!(serde_json::from_str::<CharacterId>("-1").is_err());
        assert!(serde_json::from_str::<InventoryItemId>("\"42\"").is_err());
    }
}

/// Invalid durable text identity at an API or serialized boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    InvalidParty,
    InvalidSettlement,
}
impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidParty => {
                "party identity must be nonempty, bounded, trimmed and control-free"
            }
            Self::InvalidSettlement => {
                "settlement identity must be nonempty, bounded, trimmed and control-free"
            }
        })
    }
}
impl std::error::Error for IdentityError {}

macro_rules! text_identity {
    ($name:ident, $invalid:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, IdentityError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 256
                    || value.trim() != value
                    || value.chars().any(char::is_control)
                {
                    return Err(IdentityError::$invalid);
                }
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
            pub fn into_inner(self) -> String {
                self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                Self::try_new(String::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
text_identity!(PartyId, InvalidParty);
text_identity!(SettlementId, InvalidSettlement);
