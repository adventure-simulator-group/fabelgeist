//! Soap consumption distinguishes planned binding and quantity failures.

use crate::inventory_amount::InventoryAmountError;
use adventuresim_core::{identity::InventoryItemId, physical_object::CarriedInventoryScope};

#[derive(Debug)]
pub(crate) enum SoapConsumptionError {
    Amount(InventoryAmountError),
    MissingStack {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    NotSoap {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    PlannedAmountOverflow(CarriedInventoryScope),
    RequestedAmountOverflow,
    InsufficientSoap(InventoryItemId),
}

impl From<InventoryAmountError> for SoapConsumptionError {
    fn from(source: InventoryAmountError) -> Self {
        Self::Amount(source)
    }
}
impl std::fmt::Display for SoapConsumptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CarriedInventoryScope::{Party, Personal};
        f.write_str(match self {
            Self::Amount(source) => return source.fmt(f),
            Self::MissingStack {
                scope: Personal, ..
            } => "Planned personal soap stack is missing",
            Self::MissingStack { scope: Party, .. } => "Planned shared soap stack is missing",
            Self::NotSoap {
                scope: Personal, ..
            } => "Planned personal stack is not soap",
            Self::NotSoap { scope: Party, .. } => "Planned shared stack is not soap",
            Self::PlannedAmountOverflow(Personal) => "Planned personal soap amount overflow",
            Self::PlannedAmountOverflow(Party) => "Planned shared soap amount overflow",
            Self::RequestedAmountOverflow => "Requested soap amount overflow",
            Self::InsufficientSoap(_) => "Not enough soap remains for the requested use",
        })
    }
}
impl std::error::Error for SoapConsumptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Amount(source) => Some(source),
            _ => None,
        }
    }
}
