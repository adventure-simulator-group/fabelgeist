//! Inventory request identity and optimistic state shared across live adapters.

use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::strategic_place::StrategicIdentityComponent;

/// One client intent. A retry keeps this identity and its complete payload.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct InventoryCommandId(String);

/// SHA-256 identity of one actor's admission to its serving connection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct InventoryScopeId([u8; 32]);

/// SHA-256 digest of the complete durable inventory state read by commands.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct InventoryRevision([u8; 32]);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InventorySnapshotToken {
    pub scope: InventoryScopeId,
    pub revision: InventoryRevision,
}

/// Existing durable inventory-row identity, distinct from physical identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InventoryRowId(NonZeroU64);

/// Authored placement ordinal; the selected catalog validates its upper bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EquipmentPlacementIndex(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("inventory command identity is not a canonical opaque component")]
pub struct InventoryCommandIdentityError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("inventory scope and revision must contain a complete SHA-256 digest")]
pub struct InventoryDigestError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("inventory row identity must be positive")]
pub struct InventoryRowIdentityError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum EquipmentConflictPolicy {
    RejectOccupied,
    ReplaceOccupied,
}

impl InventoryRowId {
    pub fn try_new(value: u64) -> Result<Self, InventoryRowIdentityError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(InventoryRowIdentityError)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl EquipmentPlacementIndex {
    pub const fn new(index: u16) -> Self {
        Self(index)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl InventoryCommandId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, InventoryCommandIdentityError> {
        let component = StrategicIdentityComponent::try_new(value)
            .map_err(|_| InventoryCommandIdentityError)?;
        Ok(Self(component.as_str().to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl InventoryScopeId {
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl InventoryRevision {
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl TryFrom<String> for InventoryCommandId {
    type Error = InventoryCommandIdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<InventoryCommandId> for String {
    fn from(value: InventoryCommandId) -> Self {
        value.0
    }
}

impl TryFrom<Vec<u8>> for InventoryScopeId {
    type Error = InventoryDigestError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.try_into().map(Self).map_err(|_| InventoryDigestError)
    }
}

impl TryFrom<Vec<u8>> for InventoryRevision {
    type Error = InventoryDigestError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.try_into().map(Self).map_err(|_| InventoryDigestError)
    }
}
