//! Role policy for explicitly declared deep masonry and deck junctions.
use super::*;

pub(super) fn permits(a: &ResolvedSolid, b: &ResolvedSolid, bond: &crate::JunctionBond) -> bool {
    bond.maximum_penetration_metres <= 0.18
        || matches!(
            (a.role, b.role),
            (
                SolidRole::RoofFlashing,
                SolidRole::WallHost
                    | SolidRole::OpeningJamb
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
            ) | (
                SolidRole::WallHost
                    | SolidRole::OpeningJamb
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
                    | SolidRole::ArtilleryRevetment
                    | SolidRole::ArtilleryEarthCore
                    | SolidRole::ArtilleryRetainingWall,
                SolidRole::RoofFlashing
            )
        )
        || matches!(
            (a.role, b.role),
            (
                SolidRole::WallHost
                    | SolidRole::DefenseHostWall
                    | SolidRole::CircuitWalk
                    | SolidRole::LoadBearing
                    | SolidRole::Breastwork
                    | SolidRole::WalkSurface
                    | SolidRole::DrainageChannel
                    | SolidRole::Landing
                    | SolidRole::DefenseHostButtress
                    | SolidRole::ProjectionSupport
                    | SolidRole::GalleryFloor
                    | SolidRole::InteriorFloor
                    | SolidRole::StairTread
                    | SolidRole::OpeningJamb
                    | SolidRole::OpeningSill
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
                    | SolidRole::ArtilleryRevetment
                    | SolidRole::ArtilleryEarthCore
                    | SolidRole::ArtilleryRetainingWall,
                SolidRole::WallHost
                    | SolidRole::DefenseHostWall
                    | SolidRole::CircuitWalk
                    | SolidRole::LoadBearing
                    | SolidRole::Breastwork
                    | SolidRole::WalkSurface
                    | SolidRole::DrainageChannel
                    | SolidRole::Landing
                    | SolidRole::DefenseHostButtress
                    | SolidRole::ProjectionSupport
                    | SolidRole::GalleryFloor
                    | SolidRole::InteriorFloor
                    | SolidRole::StairTread
                    | SolidRole::OpeningJamb
                    | SolidRole::OpeningSill
                    | SolidRole::OpeningHead
                    | SolidRole::OpeningSpandrel
                    | SolidRole::ArtilleryRevetment
                    | SolidRole::ArtilleryEarthCore
                    | SolidRole::ArtilleryRetainingWall
            )
        )
}
