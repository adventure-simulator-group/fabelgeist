//! Stable facade colors and scenery selection.
use super::*;

pub(super) fn building_tint(settlement: &str, service: &str, material: &str) -> String {
    let draw = |purpose: fabelgeist_determinism::StreamId, bound| {
        fabelgeist_determinism::Seed::derive(settlement.as_bytes(), purpose, &[service.as_bytes()])
            .rng()
            .index(bound)
    };
    let service_slot = match service {
        "public-square" => 0,
        "residences" => 1,
        "keep" => 2,
        "map" => 3,
        "merchants" => 4,
        "weapons" => 5,
        "armor" => 6,
        "clothing" => 7,
        "herbalist" => 8,
        "books" => 9,
        "inn" => 10,
        "religion" => 11,
        _ => draw(fabelgeist_determinism::StreamId::new("web.service-hue"), 12),
    };
    let settlement_shift = draw(
        fabelgeist_determinism::StreamId::new("web.settlement-hue"),
        9,
    );
    let hue = if material == "stone" {
        [46, 198, 218, 205, 224, 252, 282, 164, 128, 68, 36, 214][service_slot] + settlement_shift
    } else {
        [35, 58, 16, 8, 20, 31, 43, 56, 104, 48, 72, 350][service_slot] + settlement_shift
    };
    let saturation = if material == "stone" {
        12 + draw(
            fabelgeist_determinism::StreamId::new("web.stone-saturation"),
            13,
        )
    } else {
        30 + draw(
            fabelgeist_determinism::StreamId::new("web.timber-saturation"),
            25,
        )
    };
    let lightness = 19
        + draw(
            fabelgeist_determinism::StreamId::new("web.service-lightness"),
            8,
        );
    format!("hsl({hue} {saturation}% {lightness}%)")
}

pub(super) fn building_tier(category: &SettlementCategory) -> &'static str {
    match category {
        SettlementCategory::Unknown | SettlementCategory::Hamlet | SettlementCategory::Village => {
            "village"
        }
        SettlementCategory::Town => "town",
        SettlementCategory::City | SettlementCategory::Capital => "city",
    }
}

/// Temporary stable scenery selection. Imported hydrology will replace only
/// this selector; settlement markup and CSS remain variant-driven.
pub(super) fn horizon_variant(settlement_id: &str) -> HorizonVariant {
    let index = fabelgeist_determinism::Seed::derive(
        settlement_id.as_bytes(),
        fabelgeist_determinism::StreamId::new("web.horizon-variant"),
        &[],
    )
    .rng()
    .index(3);
    match index {
        0 => HorizonVariant::Inland,
        1 => HorizonVariant::Coastal,
        _ => HorizonVariant::River,
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
pub(super) enum HorizonVariant {
    Inland,
    Coastal,
    River,
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

impl HorizonVariant {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Inland => "inland",
            Self::Coastal => "coastal",
            Self::River => "river",
        }
    }
}
