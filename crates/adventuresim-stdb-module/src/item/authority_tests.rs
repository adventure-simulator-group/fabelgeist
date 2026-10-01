//! Guarded authored-to-materialized classification and database agreement.

use super::*;

#[reducer]
pub fn authority_test_catalog_item_classification(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    use adventuresim_core::item_catalog::ItemKind as Authored;
    let mut kinds = std::collections::BTreeSet::new();
    for definition in adventuresim_core::item_catalog::catalog() {
        let expected = match &definition.kind {
            Authored::Container { .. } => CatalogItemKind::Container,
            Authored::Simple => CatalogItemKind::Simple,
            Authored::Weapon { .. } => CatalogItemKind::Weapon,
            Authored::Armor { .. } => CatalogItemKind::Armor,
            Authored::Shield { .. } => CatalogItemKind::Shield,
            Authored::Clothing => CatalogItemKind::Clothing,
            Authored::Currency => CatalogItemKind::Currency,
            Authored::Ingredient => CatalogItemKind::Ingredient,
            Authored::Medication => CatalogItemKind::Medication,
            Authored::Food => CatalogItemKind::Food,
        };
        let row = ctx
            .db
            .item()
            .id()
            .find(definition.id.clone())
            .ok_or("Catalog row missing")?;
        if row.kind != expected || project_definition(definition).kind != expected {
            return Err(format!(
                "Item {} lost authored classification or capability",
                definition.id
            ));
        }
        kinds.insert(expected.stable_id());
    }
    if kinds.len() != 10 {
        return Err("Catalog classification fixture does not cover all kinds".into());
    }
    Ok(())
}
