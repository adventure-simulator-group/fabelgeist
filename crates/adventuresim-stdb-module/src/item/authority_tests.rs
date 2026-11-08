//! Guarded authored-to-materialized classification and database agreement.

use super::*;

#[reducer]
pub fn authority_test_catalog_item_classification(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
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
    use crate::inventory_item;
    use adventuresim_core::prelude::PlayerEquipment;
    let inventory = ctx
        .db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .find(|row| row.item_id == "longsword")
        .ok_or("Diagnostic character has no longsword")?;
    let geometry = crate::weapon_instance::combat_geometry(ctx, inventory.id, &inventory.item_id)
        .map_err(|error| error.to_string())?
        .ok_or("Diagnostic longsword has no persisted geometry")?;
    let original = ctx
        .db
        .item()
        .id()
        .find(&inventory.item_id)
        .ok_or("Longsword row missing")?;
    let mut selected = original.clone();
    selected.reach = geometry.melee_reach_m().metres() + 1.0;
    ctx.db.item().id().update(selected.clone());
    let equipment = crate::capability::StrategicEquipment::load(ctx, character_id)
        .map_err(|error| error.to_string())?;
    let combat = equipment
        .combat_equipment()
        .melee_weapon
        .ok_or("Melee weapon missing")?;
    if equipment.weapon_reach().metres() != selected.reach
        || combat.melee_reach != geometry.melee_reach_m()
        || combat.melee_reach == equipment.weapon_reach()
    {
        return Err("Selected and per-instance reach projections disagree".into());
    }
    ctx.db.item().id().update(original);
    Ok(())
}
