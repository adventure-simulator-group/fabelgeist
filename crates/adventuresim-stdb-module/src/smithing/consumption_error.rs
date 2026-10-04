//! Smithing consumption retains missing measured state and remaining stock need.

use crate::inventory_amount::InventoryAmountError;
use adventuresim_core::{
    inventory_measurement::MeasuredItemAmountMicros, item_catalog::ItemDefinitionId,
};

#[derive(Debug)]
pub(crate) enum SmithingConsumptionError {
    Amount(InventoryAmountError),
    Insufficient {
        item: ItemDefinitionId,
        missing: MeasuredItemAmountMicros,
    },
}

impl From<InventoryAmountError> for SmithingConsumptionError {
    fn from(source: InventoryAmountError) -> Self {
        Self::Amount(source)
    }
}

impl std::fmt::Display for SmithingConsumptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Amount(source) => source.fmt(f),
            Self::Insufficient { item, .. } => write!(f, "Insufficient {item}"),
        }
    }
}

impl std::error::Error for SmithingConsumptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Amount(source) => Some(source),
            _ => None,
        }
    }
}
