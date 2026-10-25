//! Generated catalog classification conversion at the database transport boundary.

use adventuresim_core::item_classification::CatalogItemKind;
use adventuresim_stdb_client as sats;

pub(super) fn catalog_item_kind(value: sats::CatalogItemKind) -> CatalogItemKind {
    match value {
        sats::CatalogItemKind::Simple => CatalogItemKind::Simple,
        sats::CatalogItemKind::Weapon => CatalogItemKind::Weapon,
        sats::CatalogItemKind::Armor => CatalogItemKind::Armor,
        sats::CatalogItemKind::Shield => CatalogItemKind::Shield,
        sats::CatalogItemKind::Clothing => CatalogItemKind::Clothing,
        sats::CatalogItemKind::Container => CatalogItemKind::Container,
        sats::CatalogItemKind::Currency => CatalogItemKind::Currency,
        sats::CatalogItemKind::Ingredient => CatalogItemKind::Ingredient,
        sats::CatalogItemKind::Medication => CatalogItemKind::Medication,
        sats::CatalogItemKind::Food => CatalogItemKind::Food,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generated_kind_preserves_its_materialized_classification() {
        for (wire, domain) in [
            (sats::CatalogItemKind::Simple, CatalogItemKind::Simple),
            (sats::CatalogItemKind::Weapon, CatalogItemKind::Weapon),
            (sats::CatalogItemKind::Armor, CatalogItemKind::Armor),
            (sats::CatalogItemKind::Shield, CatalogItemKind::Shield),
            (sats::CatalogItemKind::Clothing, CatalogItemKind::Clothing),
            (sats::CatalogItemKind::Container, CatalogItemKind::Container),
            (sats::CatalogItemKind::Currency, CatalogItemKind::Currency),
            (
                sats::CatalogItemKind::Ingredient,
                CatalogItemKind::Ingredient,
            ),
            (
                sats::CatalogItemKind::Medication,
                CatalogItemKind::Medication,
            ),
            (sats::CatalogItemKind::Food, CatalogItemKind::Food),
        ] {
            assert_eq!(catalog_item_kind(wire), domain);
        }
    }
}
