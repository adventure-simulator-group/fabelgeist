//! Admit catalog food metadata before inventory insertion can produce rows.

use adventuresim_core::{item_catalog::ItemDefinitionId, item_classification::CatalogItemKind};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MissingInventoryFoodDefinition {
    item_id: ItemDefinitionId,
}

impl MissingInventoryFoodDefinition {
    pub(crate) fn for_item(item_id: ItemDefinitionId) -> Self {
        Self { item_id }
    }
}

impl std::fmt::Display for MissingInventoryFoodDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Food definition not found for {}", self.item_id)
    }
}
impl std::error::Error for MissingInventoryFoodDefinition {}

pub(crate) fn inventory_food_definition(
    kind: Option<CatalogItemKind>,
    item_id: &ItemDefinitionId,
) -> Result<Option<&'static adventuresim_core::food::FoodDefinition>, MissingInventoryFoodDefinition>
{
    let definition = adventuresim_core::food::definition(item_id);
    if kind == Some(CatalogItemKind::Food) || definition.is_some() {
        definition
            .map(Some)
            .ok_or_else(|| -> MissingInventoryFoodDefinition {
                MissingInventoryFoodDefinition::for_item(item_id.clone())
            })
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_inventory_is_prevalidated_before_rows_can_be_inserted() {
        let cooked = inventory_food_definition(Some(CatalogItemKind::Food), &"cooked_meal".into())
            .unwrap()
            .expect("cooked meal definition");
        assert!(cooked.kcal_per_unit > 0.0);
        assert!(
            inventory_food_definition(Some(CatalogItemKind::Food), &"missing_food".into()).is_err()
        );
        assert_eq!(
            inventory_food_definition(Some(CatalogItemKind::Simple), &"torch".into()).unwrap(),
            None
        );
        let source = crate::production_source(include_str!("../item.rs"));
        assert_eq!(
            source.matches("id: \"cooked_meal\".into()").count(),
            0,
            "the standard food catalog must be the sole cooked-meal item seed"
        );
        let issuance = crate::production_source(include_str!("issuance.rs"));
        let checked = issuance
            .split("pub(crate) fn add_inventory_item_checked")
            .nth(1)
            .and_then(|tail| tail.split("pub fn add_inventory_item").next())
            .expect("checked inventory insertion");
        assert!(
            checked.find("inventory_food_definition").unwrap()
                < checked.find("inventory_item().insert").unwrap()
        );
        assert!(checked.contains("for row_quantity in quantity.rows(allocation)"));
        assert!(
            checked
                .find("quantity == InventoryGrantQuantity::Empty")
                .unwrap()
                < checked.find("ctx.db.item()").unwrap()
        );
        assert!(checked.contains("create_personal_food_lot("));
    }
}
