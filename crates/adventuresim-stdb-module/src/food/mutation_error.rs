//! Food material mutations preserve binding, revision, and admission failures.

use super::FoodLotCreationError;
use adventuresim_core::{
    identity::{FoodLotId, InventoryItemId},
    physical_object::CarriedInventoryScope,
};

#[derive(Debug)]
pub(crate) enum FoodLotMutationError {
    Creation(Box<FoodLotCreationError>),
    Medicine(crate::herbalism::MedicinalFoodSplitError),
    MissingLot {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    RevisionExhausted(FoodLotId),
    MissingContamination(FoodLotId),
    InvalidSplit,
    InvalidQuantityChange,
}

impl From<FoodLotCreationError> for FoodLotMutationError {
    fn from(source: FoodLotCreationError) -> Self {
        Self::Creation(Box::new(source))
    }
}

impl From<crate::herbalism::MedicinalFoodSplitError> for FoodLotMutationError {
    fn from(source: crate::herbalism::MedicinalFoodSplitError) -> Self {
        Self::Medicine(source)
    }
}

impl std::fmt::Display for FoodLotMutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creation(source) => source.fmt(f),
            Self::Medicine(source) => source.fmt(f),
            Self::MissingLot { .. } => f.write_str("Food lot metadata not found"),
            Self::RevisionExhausted(_) => f.write_str("Food material revision is exhausted"),
            Self::MissingContamination(_) => f.write_str("Food contamination state not found"),
            Self::InvalidSplit => f.write_str("Invalid food lot split"),
            Self::InvalidQuantityChange => f.write_str("Invalid food lot quantity change"),
        }
    }
}

impl std::error::Error for FoodLotMutationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Creation(source) => Some(source.as_ref()),
            Self::Medicine(source) => Some(source),
            _ => None,
        }
    }
}
