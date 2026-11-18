//! Checked equipment intent shared by browser, native and durable adapters.
//!
//! These values describe a selection, not permission or a successful effect.
//! Live admission must check control, ability and expected inventory state;
//! a panel adapter must preserve the token of its rendered view. Durable effects
//! resolve catalog bounds, ownership, fit and occupancy.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    equipment_placement::{
        EquipmentAttachmentPointId, EquipmentHand, EquipmentParentRequirementIndex,
    },
    inventory_command::{EquipmentConflictPolicy, EquipmentPlacementIndex, InventoryRowId},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentIntent {
    pub changes: EquipmentChanges,
    pub conflicts: EquipmentConflictPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentChange {
    pub inventory: InventoryRowId,
    pub destination: EquipmentDestination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentExchange {
    pub source: EquipmentChange,
    pub destination: EquipmentChange,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredEquipmentDestination {
    pub placement: EquipmentPlacementIndex,
    pub targets: ParentSelections,
}

/// Each authored parent requirement is selected at most once.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Vec<ParentSelection>", into = "Vec<ParentSelection>")]
pub struct ParentSelections(Vec<ParentSelection>);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParentSelection {
    pub requirement: EquipmentParentRequirementIndex,
    pub parent: InventoryRowId,
    pub point: EquipmentAttachmentPointId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EquipmentChanges {
    Single(EquipmentChange),
    Exchange(EquipmentExchange),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EquipmentDestination {
    Inventory,
    Held(EquipmentHand),
    Authored(AuthoredEquipmentDestination),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("Attachment parent requirement was selected more than once")]
pub struct DuplicateParentRequirement;

impl EquipmentDestination {
    pub fn authored(placement: EquipmentPlacementIndex, targets: ParentSelections) -> Self {
        Self::Authored(AuthoredEquipmentDestination { placement, targets })
    }
}

impl EquipmentChanges {
    pub fn into_changes(self) -> Vec<EquipmentChange> {
        match self {
            Self::Single(change) => vec![change],
            Self::Exchange(exchange) => vec![exchange.source, exchange.destination],
        }
    }
}

impl ParentSelections {
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParentSelection> {
        self.0.iter()
    }
}

impl TryFrom<Vec<ParentSelection>> for ParentSelections {
    type Error = DuplicateParentRequirement;

    fn try_from(targets: Vec<ParentSelection>) -> Result<Self, Self::Error> {
        let mut requirements = BTreeSet::new();
        if targets
            .iter()
            .any(|target| !requirements.insert(target.requirement))
        {
            return Err(DuplicateParentRequirement);
        }
        Ok(Self(targets))
    }
}

impl From<ParentSelections> for Vec<ParentSelection> {
    fn from(targets: ParentSelections) -> Self {
        targets.0
    }
}

impl From<ParentSelection> for ParentSelections {
    fn from(target: ParentSelection) -> Self {
        Self(vec![target])
    }
}
