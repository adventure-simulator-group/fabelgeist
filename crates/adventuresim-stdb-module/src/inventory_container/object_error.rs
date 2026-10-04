//! Stable-object admission failures retain the row and custody that failed.

use adventuresim_core::{
    identity::{CharacterId, InventoryItemId},
    physical_object::{CarriedInventoryScope, InventoryLocation},
};

#[derive(Debug)]
pub(crate) enum InventoryObjectError {
    Custody(crate::object_custody::ObjectCustodyError),
    MissingActor(CharacterId),
    MissingStableObject {
        scope: CarriedInventoryScope,
        row_id: InventoryItemId,
    },
    Duplicate {
        scope: CarriedInventoryScope,
        row_id: InventoryItemId,
    },
    RequiresCarriedCustody {
        location: InventoryLocation,
    },
    MissingBackingRow {
        location: InventoryLocation,
    },
    AlreadyPresent {
        scope: CarriedInventoryScope,
        row_id: InventoryItemId,
    },
    PersonalRowMustBeIndividual {
        row_id: InventoryItemId,
    },
    PartyRowMustBeIndividual {
        row_id: InventoryItemId,
    },
}

impl std::fmt::Display for InventoryObjectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custody(source) => source.fmt(formatter),
            Self::MissingActor(actor) => write!(formatter, "Character not found ({actor})"),
            Self::MissingStableObject { scope, row_id } => write!(
                formatter,
                "Inventory row has no stable physical object identity ({scope:?} row {row_id})"
            ),
            Self::Duplicate { scope, row_id } => write!(
                formatter,
                "Inventory row has duplicate physical object identities ({scope:?} row {row_id})"
            ),
            Self::RequiresCarriedCustody { location } => write!(
                formatter,
                "Stable physical objects require carried custody at insertion ({location:?})"
            ),
            Self::MissingBackingRow { location } => write!(
                formatter,
                "Stable physical object has no backing inventory row ({location:?})"
            ),
            Self::AlreadyPresent { scope, row_id } => write!(
                formatter,
                "Inventory row already has a stable physical object identity ({scope:?} row {row_id})"
            ),
            Self::PersonalRowMustBeIndividual { row_id } => write!(
                formatter,
                "Stable personal inventory objects require quantity-one rows (row {row_id})"
            ),
            Self::PartyRowMustBeIndividual { row_id } => write!(
                formatter,
                "Stable party inventory objects require quantity-one rows (row {row_id})"
            ),
        }
    }
}

impl From<crate::object_custody::ObjectCustodyError> for InventoryObjectError {
    fn from(source: crate::object_custody::ObjectCustodyError) -> Self {
        Self::Custody(source)
    }
}

impl std::error::Error for InventoryObjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Custody(source) => Some(source),
            _ => None,
        }
    }
}
