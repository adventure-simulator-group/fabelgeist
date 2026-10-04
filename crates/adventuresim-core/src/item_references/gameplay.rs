//! Runtime cross-catalog reference validation.

use super::REQUIRED_GAMEPLAY_ITEM_IDS;
use crate::item_catalog::{ItemDefinitionId, MissingItemDefinitions};

/// Check mandatory gameplay, treatment, currency, and catalog-wide loot references.
/// The supplied quest catalog owns monster membership; no second roster is stored.
pub fn validate_gameplay_references(
    quests: &crate::quest_catalog::Catalog,
) -> Result<(), MissingItemDefinitions> {
    let mut references = Vec::new();
    for id in REQUIRED_GAMEPLAY_ITEM_IDS {
        references.push(ItemDefinitionId::from(id));
    }
    for id in crate::strategic_currency::CURRENCY_IDS {
        references.push(ItemDefinitionId::from(id));
    }
    for profile in crate::physiology::INTERVENTION_PROFILES {
        references.push(ItemDefinitionId::from(profile.preparation_id));
    }
    for monster in quests.monsters() {
        if let Some(id) = monster.combat.loot_item_id.as_deref() {
            references.push(ItemDefinitionId::from(id));
        }
    }
    crate::item_catalog::validate_references(&references)
}
