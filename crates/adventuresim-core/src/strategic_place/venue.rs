//! Physical settlement venues and their existing service capabilities.

use adventuresim_world_schema::SettlementActionService;
use serde::{Deserialize, Serialize};

use crate::settlement_economy::Storefront;

/// Closed physical vocabulary for existing settlement venues.
///
/// This is deliberately not another settlement-service taxonomy. Authority
/// adapters can use `storefront` and `action_service` to reach the existing
/// core service types where a venue actually represents one.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
#[serde(rename_all = "snake_case")]
pub enum SettlementVenueKind {
    /// Canonical physical identity behind the `overview` presentation alias.
    PublicSquare,
    Residences,
    Keep,
    Market,
    Forge,
    Armoury,
    Tailor,
    Herbalist,
    Inn,
    Church,
    Bookstore,
}

impl SettlementVenueKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::PublicSquare => "public-square",
            Self::Residences => "residences",
            Self::Keep => "keep",
            Self::Market => "market",
            Self::Forge => "forge",
            Self::Armoury => "armoury",
            Self::Tailor => "tailor",
            Self::Herbalist => "herbalist",
            Self::Inn => "inn",
            Self::Church => "church",
            Self::Bookstore => "bookstore",
        }
    }

    pub const fn storefront(self) -> Option<Storefront> {
        match self {
            Self::Market => Some(Storefront::General),
            Self::Forge => Some(Storefront::Weapons),
            Self::Armoury => Some(Storefront::Armor),
            Self::Tailor => Some(Storefront::Clothing),
            Self::Herbalist => Some(Storefront::Herbalist),
            Self::Inn => Some(Storefront::Inn),
            Self::Bookstore => Some(Storefront::Books),
            Self::PublicSquare | Self::Residences | Self::Keep | Self::Church => None,
        }
    }

    pub const fn action_service(self) -> Option<SettlementActionService> {
        match self {
            Self::Inn => Some(SettlementActionService::Inn),
            Self::Church => Some(SettlementActionService::Temple),
            _ => None,
        }
    }

    pub const fn is_service(self) -> bool {
        self.storefront().is_some() || self.action_service().is_some()
    }

    pub const fn supports_fireplace(self) -> bool {
        !matches!(self, Self::PublicSquare)
    }

    pub fn from_id(value: &str) -> Option<Self> {
        match value {
            "public-square" => Some(Self::PublicSquare),
            "residences" => Some(Self::Residences),
            "keep" => Some(Self::Keep),
            "market" => Some(Self::Market),
            "forge" => Some(Self::Forge),
            "armoury" => Some(Self::Armoury),
            "tailor" => Some(Self::Tailor),
            "herbalist" => Some(Self::Herbalist),
            "inn" => Some(Self::Inn),
            "church" => Some(Self::Church),
            "bookstore" => Some(Self::Bookstore),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venue_kinds_map_to_existing_service_authority_without_a_second_taxonomy() {
        for (venue, storefront) in [
            (SettlementVenueKind::Market, Storefront::General),
            (SettlementVenueKind::Forge, Storefront::Weapons),
            (SettlementVenueKind::Armoury, Storefront::Armor),
            (SettlementVenueKind::Tailor, Storefront::Clothing),
            (SettlementVenueKind::Herbalist, Storefront::Herbalist),
            (SettlementVenueKind::Inn, Storefront::Inn),
            (SettlementVenueKind::Bookstore, Storefront::Books),
        ] {
            assert_eq!(venue.storefront(), Some(storefront));
            assert!(venue.is_service());
        }
        assert_eq!(
            SettlementVenueKind::Church.action_service(),
            Some(SettlementActionService::Temple)
        );
        assert_eq!(
            SettlementVenueKind::Inn.action_service(),
            Some(SettlementActionService::Inn)
        );
        assert!(SettlementVenueKind::Church.is_service());
        assert_eq!(SettlementVenueKind::Keep.storefront(), None);
        assert!(!SettlementVenueKind::Residences.is_service());
        assert!(!SettlementVenueKind::PublicSquare.is_service());
        assert_eq!(
            SettlementVenueKind::from_id("armoury"),
            Some(SettlementVenueKind::Armoury)
        );
        assert_eq!(SettlementVenueKind::from_id("armor"), None);
    }
}
