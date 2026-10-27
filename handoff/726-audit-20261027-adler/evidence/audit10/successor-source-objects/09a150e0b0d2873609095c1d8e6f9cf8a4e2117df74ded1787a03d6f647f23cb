//! Authored room ordinal admitted to the existing packed u16 representation.
use super::storey_index::{OrdinalError, RecipeOrdinalRole};
use serde::{Deserialize, Serialize};
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    bevy::reflect::Reflect,
)]
#[reflect(opaque)]
#[repr(transparent)]
#[serde(transparent)]
pub struct RoomIndex(u16);
impl RoomIndex {
    pub fn from_ordinal(ordinal: usize) -> Result<Self, OrdinalError> {
        u16::try_from(ordinal)
            .map(Self)
            .map_err(|cause| OrdinalError {
                role: RecipeOrdinalRole::Room,
                ordinal,
                cause,
            })
    }
    pub const fn from_serialized(ordinal: u16) -> Self {
        Self(ordinal)
    }
    pub const fn index(self) -> usize {
        self.0 as usize
    }
    pub const fn serialized_ordinal(self) -> u16 {
        self.0
    }
}
impl std::fmt::Display for RoomIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
