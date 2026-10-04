//! Alcohol use retains measured-amount, custody, and serving admission causes.

use crate::{inventory_amount::InventoryAmountError, inventory_container::InventoryContainerError};
use adventuresim_core::{
    identity::InventoryItemId, inventory_measurement::MeasurementError,
    item_catalog::ItemDefinitionId,
};

#[derive(Debug)]
pub(crate) enum AlcoholConsumptionError {
    Container(Box<InventoryContainerError>),
    Amount(InventoryAmountError),
    InvalidServing(MeasurementError),
    MissingSelectedRow(InventoryItemId),
    MissingDefinition(ItemDefinitionId),
}

impl From<InventoryContainerError> for AlcoholConsumptionError {
    fn from(source: InventoryContainerError) -> Self {
        Self::Container(Box::new(source))
    }
}
impl From<InventoryAmountError> for AlcoholConsumptionError {
    fn from(source: InventoryAmountError) -> Self {
        Self::Amount(source)
    }
}
impl std::fmt::Display for AlcoholConsumptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container(source) => source.fmt(f),
            Self::Amount(source) => source.fmt(f),
            Self::InvalidServing(_) => f.write_str("Selected alcohol has an invalid serving size"),
            Self::MissingSelectedRow(_) => f.write_str("Selected alcohol is no longer available"),
            Self::MissingDefinition(_) => f.write_str("Selected alcohol definition is missing"),
        }
    }
}
impl std::error::Error for AlcoholConsumptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Container(source) => Some(source.as_ref()),
            Self::Amount(source) => Some(source),
            Self::InvalidServing(source) => Some(source),
            _ => None,
        }
    }
}
