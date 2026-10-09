//! Building scale and deterministic wilderness scenery.

use crate::spacetimedb::SettlementCategory;

pub(super) fn building_tier(category: &SettlementCategory) -> &'static str {
    match category {
        SettlementCategory::Unknown | SettlementCategory::Hamlet | SettlementCategory::Village => {
            "village"
        }
        SettlementCategory::Town => "town",
        SettlementCategory::City | SettlementCategory::Capital => "city",
    }
}

/// Temporary stable terrain selection. World terrain data can replace this
/// selector without changing the shared camp and quest-location header.
pub(super) fn wilderness_variant(location_id: &str) -> WildernessVariant {
    let index = fabelgeist_determinism::Seed::derive(
        location_id.as_bytes(),
        fabelgeist_determinism::StreamId::new("web.wilderness-variant"),
        &[],
    )
    .rng()
    .index(3);
    match index {
        0 => WildernessVariant::Forest,
        1 => WildernessVariant::Grassland,
        _ => WildernessVariant::Hills,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum WildernessVariant {
    Forest,
    Grassland,
    Hills,
}

impl WildernessVariant {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Forest => "forest",
            Self::Grassland => "grassland",
            Self::Hills => "hills",
        }
    }
}
