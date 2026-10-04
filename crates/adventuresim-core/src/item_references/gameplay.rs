//! Runtime cross-catalog reference validation.

use super::REQUIRED_GAMEPLAY_ITEM_IDS;

/// Missing catalog definitions for references from gameplay or authored content.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingGameplayItemReferences {
    pub ids: Vec<String>,
}

/// Check mandatory gameplay, treatment, currency, and catalog-wide loot references.
/// The supplied quest catalog owns monster membership; no second roster is stored.
pub fn validate_gameplay_references(
    quests: &crate::quest_catalog::Catalog,
) -> Result<(), MissingGameplayItemReferences> {
    let mut references = REQUIRED_GAMEPLAY_ITEM_IDS.to_vec();
    references.extend(crate::strategic_currency::CURRENCY_IDS);
    references.extend(
        crate::physiology::INTERVENTION_PROFILES
            .iter()
            .map(|profile| profile.preparation_id),
    );
    references.extend(
        quests
            .monsters()
            .filter_map(|monster| monster.combat.loot_item_id.as_deref()),
    );
    crate::item_catalog::validate_references(references)
        .map_err(|ids| MissingGameplayItemReferences { ids })
}
