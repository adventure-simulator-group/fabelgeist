//! Bounded generated identities and open authored catalog keys.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuestIdentityError {
    BoundedId,
    OpenCatalogId,
}
impl std::fmt::Display for QuestIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BoundedId => "invalid bounded quest-generation ID",
            Self::OpenCatalogId => "invalid open catalog ID",
        })
    }
}
impl std::error::Error for QuestIdentityError {}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        pub struct $name(pub(crate) String);
        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, QuestIdentityError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 256
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.')
                    })
                {
                    return Err(QuestIdentityError::BoundedId);
                }
                Ok(Self(value))
            }
            pub(super) fn new(value: impl Into<String>) -> Self {
                Self::try_new(value).expect("static/generated quest ID")
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
                Self::try_new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
id_type!(ModuleId);
id_type!(RelationId);
id_type!(FactorId);
id_type!(BridgeId);
id_type!(SiteId);
id_type!(WitnessId);
id_type!(EvidenceId);
id_type!(ActionId);
id_type!(FinaleId);
id_type!(TrackTrailId);
id_type!(TrackSegmentId);

macro_rules! open_catalog_id {
    ($name:ident { $($constant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name { len: u8, bytes: [u8; 63] }
        impl $name {
            $(#[allow(
                non_upper_case_globals,
                reason = "the macro emits PascalCase constants as enum-like open-catalog values"
            )]
            pub const $constant: Self = Self::from_static($value);)+
            pub const fn from_static(value: &str) -> Self {
                let source = value.as_bytes();
                assert!(!source.is_empty() && source.len() <= 63);
                let mut bytes = [0; 63];
                let mut index = 0;
                while index < source.len() {
                    bytes[index] = source[index];
                    index += 1;
                }
                Self { len: source.len() as u8, bytes }
            }
            pub fn try_new(value: &str) -> Result<Self, QuestIdentityError> {
                if value.is_empty() || value.len() > 63 || !value.bytes().all(|byte|
                    byte.is_ascii_lowercase() || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-' | b'.' | b':'))
                {
                    return Err(QuestIdentityError::OpenCatalogId);
                }
                let mut bytes = [0; 63];
                bytes[..value.len()].copy_from_slice(value.as_bytes());
                Ok(Self { len: value.len() as u8, bytes })
            }
            pub fn as_str(&self) -> &str {
                core::str::from_utf8(&self.bytes[..usize::from(self.len)])
                    .expect("catalog IDs are validated ASCII")
            }
        }
        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                Self::try_new(&value).map_err(serde::de::Error::custom)
            }
        }
    };
}

open_catalog_id!(SiteKind {
    Cave => "cave", Crypt => "crypt", ForestCamp => "forest_camp",
    OccupiedHouse => "occupied_house", Riverside => "riverside",
    Graveyard => "graveyard", Roadside => "roadside", AbandonedFarm => "abandoned_farm",
    Well => "well"
});

open_catalog_id!(WitnessDemographic {
    Child => "child", Laborer => "laborer", Merchant => "merchant",
    Cleric => "cleric", Guard => "guard", Noble => "noble"
});

open_catalog_id!(Circumstance {
    NightWindow => "night_window", SecretRiversideMeeting => "secret_riverside",
    AdultVenue => "adult_venue", RoadJourney => "road",
    GraveDuty => "grave_duty", LivestockWatch => "livestock_watch"
});

open_catalog_id!(EvidenceKind {
    Footprints => "footprints", ClothScrap => "cloth_scrap", BoneDust => "bone_dust",
    BloodlessCorpse => "bloodless_corpse", DroppedToken => "dropped_token",
    DragMarks => "drag_marks", LedgerEntry => "ledger_entry"
});
