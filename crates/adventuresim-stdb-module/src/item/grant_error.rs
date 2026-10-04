//! Issuance failures retain the stage and its concrete diagnostic cause.

use super::MissingInventoryFoodDefinition;
use crate::{
    food::FoodLotCreationError, inventory_container::InventoryObjectError,
    weapon_instance::WeaponInstanceError,
};
use adventuresim_core::{identity::CharacterId, item_catalog::ItemDefinitionId};

#[derive(Debug)]
pub(crate) enum InventoryGrantError {
    FoodDefinition(MissingInventoryFoodDefinition),
    Object(InventoryObjectError),
    Weapon(WeaponInstanceError),
    FoodLot(FoodLotCreationError),
    MissingForageRow {
        character: CharacterId,
        item: ItemDefinitionId,
    },
}

impl From<MissingInventoryFoodDefinition> for InventoryGrantError {
    fn from(source: MissingInventoryFoodDefinition) -> Self {
        Self::FoodDefinition(source)
    }
}
impl From<InventoryObjectError> for InventoryGrantError {
    fn from(source: InventoryObjectError) -> Self {
        Self::Object(source)
    }
}
impl From<WeaponInstanceError> for InventoryGrantError {
    fn from(source: WeaponInstanceError) -> Self {
        Self::Weapon(source)
    }
}
impl From<FoodLotCreationError> for InventoryGrantError {
    fn from(source: FoodLotCreationError) -> Self {
        Self::FoodLot(source)
    }
}
impl std::fmt::Display for InventoryGrantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FoodDefinition(source) => source.fmt(f),
            Self::Object(source) => source.fmt(f),
            Self::Weapon(source) => source.fmt(f),
            Self::FoodLot(source) => write!(f, "Could not create food lot: {source}"),
            Self::MissingForageRow { character, item } => write!(
                f,
                "Foraged inventory insertion returned no row for character {character}, item {item}"
            ),
        }
    }
}
impl std::error::Error for InventoryGrantError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FoodDefinition(source) => Some(source),
            Self::Object(source) => Some(source),
            Self::Weapon(source) => Some(source),
            Self::FoodLot(source) => Some(source),
            Self::MissingForageRow { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{CatalogItemKind, inventory_food_definition};
    use adventuresim_core::{identity::InventoryItemId, physical_object::CarriedInventoryScope};
    use std::error::Error;

    #[test]
    fn missing_food_metadata_retains_the_exact_requested_identity() {
        let key = ItemDefinitionId::from("missing:food/metadata");
        let cause = inventory_food_definition(Some(CatalogItemKind::Food), &key).unwrap_err();
        let error = InventoryGrantError::from(cause);
        let source = error.source().unwrap();
        assert_eq!(
            source.downcast_ref::<MissingInventoryFoodDefinition>(),
            Some(&MissingInventoryFoodDefinition::for_item(key))
        );
        assert_eq!(
            error.to_string(),
            "Food definition not found for missing:food/metadata"
        );
    }

    #[test]
    fn food_lot_failure_keeps_the_underlying_inventory_binding() {
        let row = InventoryItemId::new(73);
        let cause = InventoryObjectError::Duplicate {
            scope: CarriedInventoryScope::Party,
            row_id: row,
        };
        let error = InventoryGrantError::from(FoodLotCreationError::from(cause));
        let food = error
            .source()
            .unwrap()
            .downcast_ref::<FoodLotCreationError>()
            .unwrap();
        let object = food
            .source()
            .unwrap()
            .downcast_ref::<InventoryObjectError>()
            .unwrap();
        assert!(matches!(object, InventoryObjectError::Duplicate {
            scope: CarriedInventoryScope::Party,
            row_id,
        } if *row_id == row));
        assert!(error.to_string().starts_with("Could not create food lot: "));
        assert!(error.to_string().contains("Party row 73"));
    }
}
