use crate::BuildingArchetype;

pub(super) fn wall_material_and_thickness(
    archetype: BuildingArchetype,
    exterior: bool,
    level: u16,
) -> (crate::WallMaterialClass, crate::WallStructuralRole, f32) {
    if !exterior {
        return if matches!(
            archetype,
            BuildingArchetype::TownHouse
                | BuildingArchetype::HallHouse
                | BuildingArchetype::FachwerkCottage
                | BuildingArchetype::FachwerkMerchantHouse
                | BuildingArchetype::RenaissanceTownHall
        ) {
            (
                crate::WallMaterialClass::InternalTimber,
                crate::WallStructuralRole::LoadBearing,
                0.16,
            )
        } else {
            (
                crate::WallMaterialClass::InternalMasonry,
                crate::WallStructuralRole::LoadBearing,
                0.30,
            )
        };
    }
    match archetype {
        BuildingArchetype::TownHouse
        | BuildingArchetype::HallHouse
        | BuildingArchetype::FachwerkCottage
        | BuildingArchetype::FachwerkMerchantHouse => (
            crate::WallMaterialClass::TimberInfill,
            crate::WallStructuralRole::Infill,
            if level == 0 { 0.24 } else { 0.22 },
        ),
        BuildingArchetype::RenaissanceTownHall if level == 0 => (
            crate::WallMaterialClass::CivilianMasonry,
            crate::WallStructuralRole::LoadBearing,
            0.50,
        ),
        BuildingArchetype::RenaissanceTownHall => (
            crate::WallMaterialClass::TimberInfill,
            crate::WallStructuralRole::Infill,
            0.22,
        ),
        BuildingArchetype::ParishChurch => (
            crate::WallMaterialClass::RubbleMasonry,
            crate::WallStructuralRole::LoadBearing,
            0.50,
        ),
        BuildingArchetype::Workplace => (
            crate::WallMaterialClass::CivilianMasonry,
            crate::WallStructuralRole::LoadBearing,
            0.50,
        ),
        BuildingArchetype::Cathedral => (
            crate::WallMaterialClass::CathedralMasonry,
            crate::WallStructuralRole::Buttressed,
            0.90,
        ),
        BuildingArchetype::CastleGatehouse
        | BuildingArchetype::CourtyardCastle
        | BuildingArchetype::WalledKeep
        | BuildingArchetype::ArtilleryRondelCastle => (
            crate::WallMaterialClass::FortifiedMasonry,
            crate::WallStructuralRole::LoadBearing,
            1.20,
        ),
    }
}
