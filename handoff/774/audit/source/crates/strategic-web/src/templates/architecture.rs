//! Local construction choices, independent of services, state and lighting.

use adventuresim_core::{organization::ChapterBuildingKind, strategic_place::SettlementVenueKind};

use crate::location_urls::SettlementPlace;

/// Limit the ornamental rollout to the settlement used for visual acceptance.
const GOSLAR_SETTLEMENT: &str = "viabundus-2337";

/// The MVP's South Lower Saxon and Harz building tradition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArchitecturalFamily {
    Harz,
}

impl ArchitecturalFamily {
    pub(super) const fn tag(self) -> &'static str {
        match self {
            Self::Harz => "harz",
        }
    }
}

/// A furnishing or structural assembly, not a service or interaction state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConstructionSkin {
    MapBoard,
    CivicCourt,
    MerchantHall,
    HearthRoom,
    Sanctuary,
    IronboundStore,
    DomesticCabinet,
    Workshop,
}

impl ConstructionSkin {
    pub(super) const fn tag(self) -> &'static str {
        match self {
            Self::MapBoard => "map-board",
            Self::CivicCourt => "civic-court",
            Self::MerchantHall => "merchant-hall",
            Self::HearthRoom => "hearth-room",
            Self::Sanctuary => "sanctuary",
            Self::IronboundStore => "ironbound-store",
            Self::DomesticCabinet => "domestic-cabinet",
            Self::Workshop => "workshop",
        }
    }

    const fn for_venue(venue: SettlementVenueKind) -> Self {
        match venue {
            SettlementVenueKind::PublicSquare => Self::CivicCourt,
            SettlementVenueKind::Market => Self::MerchantHall,
            SettlementVenueKind::Inn => Self::HearthRoom,
            SettlementVenueKind::Church => Self::Sanctuary,
            SettlementVenueKind::Armoury | SettlementVenueKind::Keep => Self::IronboundStore,
            SettlementVenueKind::Residences
            | SettlementVenueKind::Bookstore
            | SettlementVenueKind::Herbalist => Self::DomesticCabinet,
            SettlementVenueKind::Forge | SettlementVenueKind::Tailor => Self::Workshop,
        }
    }

    const fn for_chapter(kind: ChapterBuildingKind) -> Self {
        match kind {
            ChapterBuildingKind::Guildhall => Self::MerchantHall,
            ChapterBuildingKind::Workshop => Self::Workshop,
            ChapterBuildingKind::College | ChapterBuildingKind::Lodge => Self::DomesticCabinet,
            ChapterBuildingKind::Confraternity => Self::Sanctuary,
            ChapterBuildingKind::Commandery => Self::IronboundStore,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StructuralMaterial {
    Timber,
    Sandstone,
    IronboundTimber,
}

impl StructuralMaterial {
    pub(super) const fn tag(self) -> &'static str {
        match self {
            Self::Timber => "timber",
            Self::Sandstone => "sandstone",
            Self::IronboundTimber => "ironbound-timber",
        }
    }

    /// Natural facade color; painted furnishings have their own CSS palette.
    pub(super) const fn tint(self) -> &'static str {
        match self {
            Self::Timber => "#856044",
            Self::Sandstone => "#9b896c",
            Self::IronboundTimber => "#625849",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SettlementPresentation {
    pub(super) family: Option<ArchitecturalFamily>,
    pub(super) skin: ConstructionSkin,
    pub(super) material: StructuralMaterial,
}

impl SettlementPresentation {
    /// Translate the existing layout boundary into canonical physical places.
    pub(super) fn from_active_service(
        settlement: &str,
        active_service: &str,
    ) -> SettlementPresentation {
        let place = if active_service == "map" {
            None
        } else {
            let slug = if active_service.is_empty() {
                adventuresim_core::strategic_place::SettlementVenueKind::PublicSquare.id()
            } else {
                adventuresim_core::organization::service_npc_location_id(active_service)
                    .unwrap_or(active_service)
            };
            Some(
                crate::location_urls::SettlementPlace::parse(settlement, slug)
                    .expect("settlement layouts have a canonical physical place"),
            )
        };
        Self::for_place(settlement, place)
    }

    pub(super) fn for_place(settlement: &str, place: Option<SettlementPlace<'_>>) -> Self {
        let skin = match place {
            None => ConstructionSkin::MapBoard,
            Some(SettlementPlace::Venue(venue)) => ConstructionSkin::for_venue(venue),
            Some(SettlementPlace::Chapter(id)) => {
                let (_, chapter) =
                    adventuresim_core::organization::organization_chapter_at(settlement, id)
                        .expect("presentation receives a validated local chapter");
                ConstructionSkin::for_chapter(chapter.building_kind)
            }
        };
        let material = match skin {
            ConstructionSkin::CivicCourt | ConstructionSkin::Sanctuary => {
                StructuralMaterial::Sandstone
            }
            ConstructionSkin::IronboundStore | ConstructionSkin::Workshop => {
                StructuralMaterial::IronboundTimber
            }
            _ => StructuralMaterial::Timber,
        };
        Self {
            family: (settlement == GOSLAR_SETTLEMENT).then_some(ArchitecturalFamily::Harz),
            skin,
            material,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_place_determines_construction_without_city_stone_assumption() {
        let inn = SettlementPresentation::for_place(
            "viabundus-2337",
            Some(SettlementPlace::Venue(SettlementVenueKind::Inn)),
        );
        let church = SettlementPresentation::for_place(
            "viabundus-2337",
            Some(SettlementPlace::Venue(SettlementVenueKind::Church)),
        );
        assert_eq!(inn.family, church.family);
        assert_eq!(inn.material, StructuralMaterial::Timber);
        assert_eq!(church.material, StructuralMaterial::Sandstone);
        assert_ne!(inn.skin, church.skin);
        assert_eq!(
            SettlementPresentation::for_place("viabundus-2337", None).skin,
            ConstructionSkin::MapBoard,
        );
    }

    #[test]
    fn ornamental_rollout_stays_in_the_reviewed_settlement() {
        assert!(
            SettlementPresentation::for_place(GOSLAR_SETTLEMENT, None)
                .family
                .is_some()
        );
        assert!(
            SettlementPresentation::for_place("viabundus-0", None)
                .family
                .is_none()
        );
    }

    #[test]
    fn chapter_construction_uses_its_building_function() {
        assert_eq!(
            ConstructionSkin::for_chapter(ChapterBuildingKind::Guildhall),
            ConstructionSkin::MerchantHall
        );
        assert_eq!(
            ConstructionSkin::for_chapter(ChapterBuildingKind::Workshop),
            ConstructionSkin::Workshop
        );
        assert_ne!(
            ConstructionSkin::for_chapter(ChapterBuildingKind::Guildhall),
            ConstructionSkin::Sanctuary
        );
    }
}
