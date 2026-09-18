/// Equal-value historical denominations used by the northern-German 1544
/// setting. Exchange-rate gameplay intentionally remains deferred.
pub use crate::item_references::CURRENCY_IDS;

pub fn is_currency_id(item_id: &str) -> bool {
    crate::item_catalog::definition(item_id)
        .is_some_and(|item| matches!(&item.kind, crate::item_catalog::ItemKind::Currency))
}

pub fn currency_name(item_id: &str) -> Option<&'static str> {
    crate::item_catalog::definition(item_id)
        .filter(|item| matches!(&item.kind, crate::item_catalog::ItemKind::Currency))
        .map(|item| item.display_name.as_str())
}

/// Deterministic assignment from the authored currency catalog.
pub fn assigned_currency_id(settlement_id: &str) -> &'static str {
    let mut random = fabelgeist_determinism::Seed::derive(
        settlement_id.as_bytes(),
        fabelgeist_determinism::StreamId::new("settlement.currency"),
        &[],
    )
    .rng();
    CURRENCY_IDS[random.index(CURRENCY_IDS.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_names_and_assignment_are_total_and_stable() {
        assert!(CURRENCY_IDS.iter().all(|id| currency_name(id).is_some()));
        assert_eq!(
            assigned_currency_id("viabundus-123"),
            assigned_currency_id("viabundus-123")
        );
        assert!(is_currency_id(assigned_currency_id("viabundus-123")));
    }
}
