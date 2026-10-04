//! Container operations retain object, custody, identity, and graph failures.

use super::InventoryObjectError;
use crate::object_custody::ObjectCustodyError;
use adventuresim_core::{
    inventory_containers::ContainmentError, item_catalog::ItemDefinitionId, material::Milliliters,
    physical_object::CustodyIdentityError,
};

#[derive(Debug)]
pub(crate) enum InventoryContainerError {
    Object(Box<InventoryObjectError>),
    Custody(Box<ObjectCustodyError>),
    Identity(CustodyIdentityError),
    Graph(ContainmentError),
    Food(Box<crate::food::FoodLotMutationError>),
    Amount(crate::inventory_amount::InventoryAmountError),
    MissingDefinition(ItemDefinitionId),
    CapacityExceeded {
        used: Milliliters,
        capacity: Milliliters,
    },
    FireplaceSubtreeDeletion,
    MissingWaterContainer,
    ContainedWaterOverflow,
    AncestryDepthExceeded,
    TinctureLocked,
    CookingLocked,
    ContentsExceedCapacity,
    MissingContainerDefinition,
    MissingContainer,
    StackOverflow,
    VolumeOverflow,
    DestinationCharacterUnavailable,
    DestinationPartyUnavailable,
    MissingEmptiedPersonalRow,
    MissingEmptiedPartyRow,
    InvalidSubtreeDestination,
    InvalidRowDestination,
    UnsupportedDeletionLocation,
    UnsupportedLocationTransition,
    MissingSubtreeObject,
    MissingSubtreeRoot,
    MissingPersonalSubtreeRow,
    NonCarriedMerge,
    MissingPartySubtreeRow,
    NonCarriedBacking,
    BackingIdentityMismatch,
    RootCustodyMismatch,
    MissingBackingIdentity,
    MissingBackingRow,
    RepairContainment,
    DuplicateRepairObject,
    FireplaceModification,
    FireplaceMovement,
    FireplaceConsumption,
    NonIndividualRow,
    NotContainer,
    WaterOverflow,
}

impl From<InventoryObjectError> for InventoryContainerError {
    fn from(source: InventoryObjectError) -> Self {
        Self::Object(Box::new(source))
    }
}
impl From<ObjectCustodyError> for InventoryContainerError {
    fn from(source: ObjectCustodyError) -> Self {
        Self::Custody(Box::new(source))
    }
}
impl From<CustodyIdentityError> for InventoryContainerError {
    fn from(source: CustodyIdentityError) -> Self {
        Self::Identity(source)
    }
}
impl From<ContainmentError> for InventoryContainerError {
    fn from(source: ContainmentError) -> Self {
        Self::Graph(source)
    }
}
impl From<crate::food::FoodLotMutationError> for InventoryContainerError {
    fn from(source: crate::food::FoodLotMutationError) -> Self {
        Self::Food(Box::new(source))
    }
}
impl From<crate::inventory_amount::InventoryAmountError> for InventoryContainerError {
    fn from(source: crate::inventory_amount::InventoryAmountError) -> Self {
        Self::Amount(source)
    }
}
impl std::fmt::Display for InventoryContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object(source) => source.fmt(f),
            Self::Custody(source) => source.fmt(f),
            Self::Identity(source) => source.fmt(f),
            Self::Graph(source) => source.fmt(f),
            Self::Food(source) => source.fmt(f),
            Self::Amount(source) => source.fmt(f),
            Self::MissingDefinition(key) => write!(f, "Unknown item {key}"),
            Self::CapacityExceeded { used, capacity } => write!(
                f,
                "Container capacity exceeded: {} ml used of {} ml",
                used.get(),
                capacity.get()
            ),
            Self::FireplaceSubtreeDeletion => {
                f.write_str("A fixture-held subtree must be retrieved before deletion")
            }
            Self::MissingWaterContainer => {
                f.write_str("Contained water has no physical container object")
            }
            Self::ContainedWaterOverflow => f.write_str("Contained water volume overflow"),
            Self::AncestryDepthExceeded => {
                f.write_str("Container ancestry exceeds the maximum depth")
            }
            Self::TinctureLocked => {
                f.write_str("Container contents are locked while a tincture is macerating")
            }
            Self::CookingLocked => f.write_str("Container contents are locked while cooking"),
            Self::ContentsExceedCapacity => {
                f.write_str("Container contents exceed authored capacity")
            }
            Self::MissingContainerDefinition => f.write_str("Container definition not found"),
            Self::MissingContainer => f.write_str("Container object not found"),
            Self::StackOverflow => f.write_str("Container stack quantity overflow"),
            Self::VolumeOverflow => f.write_str("Container volume overflow"),
            Self::DestinationCharacterUnavailable => {
                f.write_str("Destination character custody is unavailable")
            }
            Self::DestinationPartyUnavailable => {
                f.write_str("Destination party custody is unavailable")
            }
            Self::MissingEmptiedPersonalRow => f.write_str("Emptied container row is missing"),
            Self::MissingEmptiedPartyRow => f.write_str("Emptied party container row is missing"),
            Self::InvalidSubtreeDestination => f.write_str("Invalid subtree destination"),
            Self::InvalidRowDestination => {
                f.write_str("Inventory row destination must be a character or party")
            }
            Self::UnsupportedDeletionLocation => {
                f.write_str("Inventory subtree has an unsupported deletion location")
            }
            Self::UnsupportedLocationTransition => {
                f.write_str("Inventory subtree has an unsupported location transition")
            }
            Self::MissingSubtreeObject => f.write_str("Inventory subtree object is missing"),
            Self::MissingSubtreeRoot => f.write_str("Inventory subtree root object is missing"),
            Self::MissingPersonalSubtreeRow => f.write_str("Inventory subtree row is missing"),
            Self::NonCarriedMerge => {
                f.write_str("Only carried containers can merge into inventory stacks")
            }
            Self::MissingPartySubtreeRow => f.write_str("Party inventory subtree row is missing"),
            Self::NonCarriedBacking => {
                f.write_str("Physical object backing is not a carried inventory")
            }
            Self::BackingIdentityMismatch => {
                f.write_str("Physical object conflicts with its backing identity")
            }
            Self::RootCustodyMismatch => {
                f.write_str("Physical object has conflicting authenticated root custody")
            }
            Self::MissingBackingIdentity => f.write_str("Physical object has no backing identity"),
            Self::MissingBackingRow => f.write_str("Physical object has no backing inventory row"),
            Self::RepairContainment => {
                f.write_str("Repair escrow object has an invalid containment edge")
            }
            Self::DuplicateRepairObject => {
                f.write_str("Repair inventory row has duplicate physical objects")
            }
            Self::FireplaceModification => f.write_str(
                "Retrieve the container from its fireplace before changing its contents",
            ),
            Self::FireplaceMovement => {
                f.write_str("Retrieve the container from its fireplace before moving its contents")
            }
            Self::FireplaceConsumption => {
                f.write_str("Retrieve the container from its fireplace before using its contents")
            }
            Self::NonIndividualRow => f.write_str("Stable inventory objects must be quantity one"),
            Self::NotContainer => f.write_str("That item is not a container"),
            Self::WaterOverflow => f.write_str("Water volume overflow"),
        }
    }
}
impl std::error::Error for InventoryContainerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Object(source) => Some(source.as_ref()),
            Self::Custody(source) => Some(source.as_ref()),
            Self::Identity(source) => Some(source),
            Self::Graph(source) => Some(source),
            Self::Food(source) => Some(source.as_ref()),
            Self::Amount(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn container_failures_retain_concrete_identity_and_custody_causes() {
        let identity =
            adventuresim_core::physical_object::PhysicalObjectId::try_new(0).unwrap_err();
        let error = InventoryContainerError::from(identity);
        assert!(matches!(
            error
                .source()
                .unwrap()
                .downcast_ref::<CustodyIdentityError>(),
            Some(CustodyIdentityError::ZeroObjectId)
        ));

        let custody = crate::object_custody::decode_custody(
            &crate::object_custody::PersistedOperationalCustody::Character { character_id: 0 },
        )
        .unwrap_err();
        let error = InventoryContainerError::from(custody);
        let custody = error
            .source()
            .unwrap()
            .downcast_ref::<ObjectCustodyError>()
            .unwrap();
        assert!(matches!(
            custody
                .source()
                .unwrap()
                .downcast_ref::<CustodyIdentityError>(),
            Some(CustodyIdentityError::ZeroCharacterId)
        ));
    }
}
