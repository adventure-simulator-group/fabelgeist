//! Read-only inventory authority shared by admission, live refresh and load.

use adventuresim_core::{
    inventory_measurement::InventoryMassKg, physical_object::PhysicalObjectId,
};
use spacetimedb::SpacetimeType;

pub(crate) use mass::personal_row_mass;

mod custody;
mod mass;

pub(crate) struct InventoryRowMass {
    pub object_id: Option<PhysicalObjectId>,
    pub unit: InventoryMassKg,
    /// This row's remaining construction and liquid, excluding child rows.
    pub carried: InventoryMassKg,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryProjectionError {
    #[error("inventory row has no catalog definition")]
    MissingDefinition,
    #[error("inventory row has an invalid quantity")]
    InvalidQuantity,
    #[error("inventory row has an invalid physical identity")]
    InvalidIdentity,
    #[error("inventory construction does not match its authenticated recipe")]
    InvalidConstruction,
    #[error("inventory row has invalid remaining material")]
    InvalidRemainingMaterial,
    #[error("inventory mass must be finite and nonnegative")]
    InvalidMass,
    #[error("contained liquid has an invalid authored volume")]
    InvalidLiquidVolume,
    #[error("inventory containment does not resolve to its character or a stored fixture")]
    InvalidContainment,
}

#[cfg(feature = "authority-tests")]
mod authority_tests;
