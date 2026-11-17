//! Durable equipment distinguishes generic held items from authored placement.

use serde::{Deserialize, Serialize};

use crate::{item_catalog::EquipmentLocation, strategic_place::StrategicIdentityComponent};

/// Stable identity within one item's authored placement catalog.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct AuthoredPlacementId {
    value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("authored equipment placement identity is not a canonical component")]
pub struct AuthoredPlacementIdentityError;

/// A named attachment point within its owning item's authored catalog.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EquipmentAttachmentPointId {
    value: String,
}

/// Authored parent requirement ordinal; its selected placement checks the bound.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EquipmentParentRequirementIndex(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("equipment attachment point identity is not a canonical component")]
pub struct EquipmentAttachmentPointIdentityError;

/// A hand is a generic equipment destination, independent of an item's kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum EquipmentHand {
    Left,
    Right,
}

/// Held clothing does not select its catalog's worn surface or protection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum EquippedPlacement {
    Authored(AuthoredPlacementId),
    Held(EquipmentHand),
}

impl AuthoredPlacementId {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for AuthoredPlacementId {
    type Error = AuthoredPlacementIdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let component = StrategicIdentityComponent::try_new(value)
            .map_err(|_| AuthoredPlacementIdentityError)?;
        Ok(Self {
            value: component.as_str().to_owned(),
        })
    }
}

impl From<AuthoredPlacementId> for String {
    fn from(value: AuthoredPlacementId) -> Self {
        value.value
    }
}

impl EquipmentAttachmentPointId {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for EquipmentAttachmentPointId {
    type Error = EquipmentAttachmentPointIdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let component = StrategicIdentityComponent::try_new(value)
            .map_err(|_| EquipmentAttachmentPointIdentityError)?;
        Ok(Self {
            value: component.as_str().to_owned(),
        })
    }
}

impl From<EquipmentAttachmentPointId> for String {
    fn from(value: EquipmentAttachmentPointId) -> Self {
        value.value
    }
}

impl EquipmentParentRequirementIndex {
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl EquipmentHand {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }

    pub const fn location(self) -> EquipmentLocation {
        match self {
            Self::Left => EquipmentLocation::LeftHand,
            Self::Right => EquipmentLocation::RightHand,
        }
    }
}

impl EquippedPlacement {
    pub const fn authored_id(&self) -> Option<&AuthoredPlacementId> {
        match self {
            Self::Authored(id) => Some(id),
            Self::Held(_) => None,
        }
    }

    pub const fn hand(&self) -> Option<EquipmentHand> {
        match self {
            Self::Authored(_) => None,
            Self::Held(hand) => Some(*hand),
        }
    }
}
