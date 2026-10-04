//! Food-lot admission preserves the failed inventory binding and concrete cause.

use crate::{inventory_container::InventoryObjectError, item::MissingInventoryFoodDefinition};
use adventuresim_core::{identity::InventoryItemId, physical_object::CarriedInventoryScope};

#[derive(Debug)]
pub(crate) enum FoodLotCreationError {
    MissingDefinition(MissingInventoryFoodDefinition),
    Object(InventoryObjectError),
    MissingRow {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    NonIndividual {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
    MismatchedObject {
        scope: CarriedInventoryScope,
        row: InventoryItemId,
    },
}
impl From<MissingInventoryFoodDefinition> for FoodLotCreationError {
    fn from(source: MissingInventoryFoodDefinition) -> Self {
        Self::MissingDefinition(source)
    }
}
impl From<InventoryObjectError> for FoodLotCreationError {
    fn from(source: InventoryObjectError) -> Self {
        Self::Object(source)
    }
}
impl std::fmt::Display for FoodLotCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDefinition(source) => source.fmt(f),
            Self::Object(source) => source.fmt(f),
            Self::MissingRow { scope, row } => {
                write!(f, "Food inventory row is missing ({scope:?} row {row})")
            }
            Self::NonIndividual { scope, row } => write!(
                f,
                "Every food lot requires a quantity-one stable inventory object ({scope:?} row {row})"
            ),
            Self::MismatchedObject { scope, row } => write!(
                f,
                "Food inventory row has a mismatched stable object identity ({scope:?} row {row})"
            ),
        }
    }
}
impl std::error::Error for FoodLotCreationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingDefinition(source) => Some(source),
            Self::Object(source) => Some(source),
            _ => None,
        }
    }
}
