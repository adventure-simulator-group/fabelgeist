//! Door throat width and bearing material at the opening-programme boundary.
use super::*;

const INHERITED_MASONRY_SERVICE_DOOR_METRES: f32 = 0.78;

pub(super) fn resolve(
    archetype: BuildingArchetype,
    opening: Opening,
) -> (
    crate::OpeningUse,
    crate::OpeningProfile,
    crate::OpeningHeadKind,
) {
    let width_metres = if matches!(
        archetype,
        BuildingArchetype::CastleGatehouse
            | BuildingArchetype::CourtyardCastle
            | BuildingArchetype::WalledKeep
            | BuildingArchetype::ArtilleryRondelCastle
    ) {
        INHERITED_MASONRY_SERVICE_DOOR_METRES
    } else {
        opening.width_metres
    };
    let head = if matches!(
        archetype,
        BuildingArchetype::TownHouse
            | BuildingArchetype::HallHouse
            | BuildingArchetype::FachwerkCottage
            | BuildingArchetype::FachwerkMerchantHouse
    ) {
        crate::OpeningHeadKind::TimberLintel
    } else {
        crate::OpeningHeadKind::StoneLintel
    };
    (
        crate::OpeningUse::Door,
        crate::OpeningProfile::Rectangular {
            width_metres,
            height_metres: opening.height_metres,
        },
        head,
    )
}
