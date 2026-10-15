//! Accessible destination links and their collision priority, alongside Bevy pins.
use crate::{
    routes::travel::CaseSiteKnowledgePresentation,
    spacetimedb::{BackendCaseSitePin, SettlementCategory, SettlementView},
    templates::prosperity_tier_label,
};
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_tactical_core::regional_map::{MapMarker, MapMarkerEmphasis, MapMarkerRank};

pub(super) struct MapLink {
    pub marker: MapMarker,
    pub name: String,
    pub label: String,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MapLinkPriority {
    Hamlet,
    Village,
    Town,
    City,
    Capital,
    CaseSite,
    Connected,
    Selected,
    Current,
}

impl MapLink {
    pub fn settlement(marker: MapMarker, settlement: &SettlementView) -> Self {
        let prosperity = prosperity_tier_label(settlement.economy.prosperity_tier);
        let state = match marker.emphasis {
            MapMarkerEmphasis::Current | MapMarkerEmphasis::CurrentSelected => "current settlement",
            MapMarkerEmphasis::Connected => "direct route available",
            MapMarkerEmphasis::Selected => "selected destination",
            MapMarkerEmphasis::Ordinary => "no direct route",
        };
        Self {
            marker,
            name: settlement.name.clone(),
            label: format!("{}, {prosperity} prosperity, {state}", settlement.name),
        }
    }

    pub fn case_site(
        marker: MapMarker,
        site: &BackendCaseSitePin,
        knowledge: CaseSiteKnowledgePresentation,
    ) -> Self {
        Self {
            marker,
            name: site.display_title.clone(),
            label: format!(
                "Known case site: {}, {}",
                site.display_title,
                knowledge.label()
            ),
        }
    }

    pub fn settlement_rank(category: &SettlementCategory) -> MapMarkerRank {
        match category {
            SettlementCategory::Unknown | SettlementCategory::Village => MapMarkerRank::Village,
            SettlementCategory::Hamlet => MapMarkerRank::Hamlet,
            SettlementCategory::Town => MapMarkerRank::Town,
            SettlementCategory::City => MapMarkerRank::City,
            SettlementCategory::Capital => MapMarkerRank::Capital,
        }
    }

    pub fn priority(&self) -> MapLinkPriority {
        match self.marker.emphasis {
            MapMarkerEmphasis::Current | MapMarkerEmphasis::CurrentSelected => {
                MapLinkPriority::Current
            }
            MapMarkerEmphasis::Selected => MapLinkPriority::Selected,
            MapMarkerEmphasis::Connected => MapLinkPriority::Connected,
            MapMarkerEmphasis::Ordinary => match self.marker.rank {
                MapMarkerRank::Hamlet => MapLinkPriority::Hamlet,
                MapMarkerRank::Village => MapLinkPriority::Village,
                MapMarkerRank::Town => MapLinkPriority::Town,
                MapMarkerRank::City => MapLinkPriority::City,
                MapMarkerRank::Capital => MapLinkPriority::Capital,
                MapMarkerRank::CaseSite => MapLinkPriority::CaseSite,
            },
        }
    }

    /// Canonical place IDs become HTTP query components only at this adapter.
    pub fn href(&self, path: &str) -> Option<String> {
        let destination = match &self.marker.place {
            StrategicPlaceId::Settlement { settlement_id } => settlement_id.as_str(),
            StrategicPlaceId::CaseSite { site_id } => site_id.as_str(),
            _ => return None,
        };
        Some(crate::location_urls::with_query(
            path,
            "destination",
            destination,
        ))
    }

    pub fn selected(&self) -> bool {
        matches!(
            self.marker.emphasis,
            MapMarkerEmphasis::Selected | MapMarkerEmphasis::CurrentSelected
        )
    }

    pub fn state_label(&self) -> Option<&'static str> {
        match self.marker.emphasis {
            MapMarkerEmphasis::Current => Some("Origin"),
            MapMarkerEmphasis::CurrentSelected => Some("Origin · selected"),
            MapMarkerEmphasis::Selected => Some("Selected"),
            MapMarkerEmphasis::Connected | MapMarkerEmphasis::Ordinary => None,
        }
    }

    pub fn state_class(&self) -> &'static str {
        match self.marker.emphasis {
            MapMarkerEmphasis::Ordinary => "ordinary",
            MapMarkerEmphasis::Connected => "connected",
            MapMarkerEmphasis::Current => "current",
            MapMarkerEmphasis::Selected => "selected",
            MapMarkerEmphasis::CurrentSelected => "current selected",
        }
    }
}
