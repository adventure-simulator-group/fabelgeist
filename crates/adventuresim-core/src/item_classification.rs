//! Materialized item classification shared by persistence and native views.

use crate::settlement_economy::CatalogKind;

/// Classification projected from authored, stat-bearing item definitions.
/// Catalog loading owns this projection; generated SDK rows adapt it at the
/// transport boundary. Capabilities and equipment topology remain separate facts.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum CatalogItemKind {
    #[default]
    Simple,
    Weapon,
    Armor,
    Shield,
    Clothing,
    Container,
    Currency,
    Ingredient,
    Medication,
    Food,
}

impl CatalogItemKind {
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Weapon => "weapon",
            Self::Armor => "armor",
            Self::Shield => "shield",
            Self::Clothing => "clothing",
            Self::Container => "container",
            Self::Currency => "currency",
            Self::Ingredient => "ingredient",
            Self::Medication => "medication",
            Self::Food => "food",
        }
    }

    /// Economic stocking groups containers with general goods. This is a lossy
    /// policy projection, not a second materialized item classification.
    pub const fn economy_kind(self) -> CatalogKind {
        match self {
            Self::Simple | Self::Container => CatalogKind::Simple,
            Self::Weapon => CatalogKind::Weapon,
            Self::Armor => CatalogKind::Armor,
            Self::Shield => CatalogKind::Shield,
            Self::Clothing => CatalogKind::Clothing,
            Self::Currency => CatalogKind::Currency,
            Self::Ingredient => CatalogKind::Ingredient,
            Self::Medication => CatalogKind::Medication,
            Self::Food => CatalogKind::Food,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_classification_preserves_transport_and_economic_contracts() {
        for (kind, wire, stable, economic) in [
            (
                CatalogItemKind::Simple,
                "Simple",
                "simple",
                CatalogKind::Simple,
            ),
            (
                CatalogItemKind::Weapon,
                "Weapon",
                "weapon",
                CatalogKind::Weapon,
            ),
            (CatalogItemKind::Armor, "Armor", "armor", CatalogKind::Armor),
            (
                CatalogItemKind::Shield,
                "Shield",
                "shield",
                CatalogKind::Shield,
            ),
            (
                CatalogItemKind::Clothing,
                "Clothing",
                "clothing",
                CatalogKind::Clothing,
            ),
            (
                CatalogItemKind::Container,
                "Container",
                "container",
                CatalogKind::Simple,
            ),
            (
                CatalogItemKind::Currency,
                "Currency",
                "currency",
                CatalogKind::Currency,
            ),
            (
                CatalogItemKind::Ingredient,
                "Ingredient",
                "ingredient",
                CatalogKind::Ingredient,
            ),
            (
                CatalogItemKind::Medication,
                "Medication",
                "medication",
                CatalogKind::Medication,
            ),
            (CatalogItemKind::Food, "Food", "food", CatalogKind::Food),
        ] {
            assert_eq!(kind.stable_id(), stable);
            assert_eq!(kind.economy_kind(), economic);
            let encoded = serde_json::to_value(kind).unwrap();
            assert_eq!(encoded, wire);
            assert_eq!(
                serde_json::from_value::<CatalogItemKind>(encoded).unwrap(),
                kind
            );
        }
    }
}
