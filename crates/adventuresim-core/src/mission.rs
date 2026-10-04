use serde::{Deserialize, Serialize};
use std::fmt;

mod enemy_roster;
mod roster;
pub use enemy_roster::{EnemyRosterError, MissionEnemyRoster};
pub use roster::{TacticalPartyRoster, TacticalRosterError};

/// Hard limits for authenticated tactical terminal consequence receipts.
pub const MAX_TACTICAL_RECEIPT_PARTICIPANTS: usize = 16;
pub const MAX_TACTICAL_INJURIES_PER_PARTICIPANT: usize = 64;
pub const MAX_TACTICAL_EQUIPMENT_CONTACTS: usize = 128;
pub const MAX_TACTICAL_DAMAGE_PER_HIT: f32 = 1.0;
pub const MAX_TACTICAL_CONTACT_STRESS: f32 = 10_000.0;
pub const MAX_TACTICAL_AMMUNITION_USED: u32 = 1_024;

/// Invalid canonical identity for a mission authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityIdError {
    pub expected_prefix: &'static str,
}

impl fmt::Display for AuthorityIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "authority ID must be bounded canonical ASCII starting with {}",
            self.expected_prefix
        )
    }
}

impl std::error::Error for AuthorityIdError {}

macro_rules! authority_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, AuthorityIdError> {
                let value = value.into();
                if value.len() > 128
                    || !value.starts_with($prefix)
                    || value.len() == $prefix.len()
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || matches!(byte, b':' | b'-' | b'_')
                    })
                {
                    return Err(AuthorityIdError {
                        expected_prefix: $prefix,
                    });
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
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

authority_id!(MissionId, "mission:");
authority_id!(BattleId, "battle:");
authority_id!(HostileGroupId, "hostile-group:");
authority_id!(OutcomeSourceId, "outcome:");

/// A mission may be deliberately unbound (for example, a random encounter).
/// Only the bound form is eligible to defeat a persistent hostile group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostileGroupBinding {
    Unbound,
    Bound(HostileGroupId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategicBattleVictor {
    Allies,
    Enemies,
    Stalemate,
}

/// Typed persistent result a trusted tactical mission may report. The
/// strategic mission authority pre-binds which non-failure result is valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategicHostileResolution {
    Defeated,
    DrivenOff,
    Captured,
    CaptureTargetKilled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_ids_reject_empty_values() {
        assert!(MissionId::new("").is_err());
        assert_eq!(BattleId::new("battle:1").unwrap().as_str(), "battle:1");
        assert!(MissionId::new("battle:1").is_err());
        assert!(MissionId::new("mission:UPPER").is_err());
        assert!(MissionId::new(format!("mission:{}", "x".repeat(129))).is_err());
    }

    #[test]
    fn received_authority_ids_cannot_bypass_their_kind_or_bounds() {
        for raw in ["", "battle:1", "mission:UPPER", "mission:", "mission:a\n"] {
            let json = serde_json::to_string(raw).unwrap();
            assert!(serde_json::from_str::<MissionId>(&json).is_err());
        }
        let id: MissionId = serde_json::from_str("\"mission:1\"").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"mission:1\"");
        assert!(serde_json::from_str::<BattleId>("\"mission:1\"").is_err());
        assert!(serde_json::from_str::<HostileGroupId>("\"outcome:1\"").is_err());
        assert!(serde_json::from_str::<OutcomeSourceId>("\"hostile-group:1\"").is_err());
    }

    #[test]
    fn unbound_encounters_cannot_alias_a_hostile_group() {
        let random = HostileGroupBinding::Unbound;
        let bound = HostileGroupBinding::Bound(HostileGroupId::new("hostile-group:1").unwrap());
        assert_ne!(random, bound);
    }
}
