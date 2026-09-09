use super::*;

pub(super) fn material_for_solid(
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
) -> BuildingLodMaterial {
    if let Some(part) = plan
        .workplace
        .as_ref()
        .and_then(|workplace| workplace.parts.iter().find(|part| part.solid == solid.id))
    {
        return part.material.render_material();
    }
    let wall_material = wall_for_solid(plan, solid).map(|wall| wall.material);
    material_for_solid_body(plan, solid, wall_material)
}

pub(super) fn wall_for_solid<'plan>(
    plan: &'plan BuildingPlan,
    solid: &ResolvedSolid,
) -> Option<&'plan crate::WallAssembly> {
    if solid.role == SolidRole::WorkplacePart {
        return plan
            .wall_assemblies
            .iter()
            .find(|wall| wall.host_solids.contains(&solid.id));
    }
    if let Some(wall) = plan.wall_assemblies.iter().find(|wall| {
        wall.host_solids.contains(&solid.id) || wall.replaced_by_owner == Some(solid.owner)
    }) {
        return Some(wall);
    }
    if let Some(wall_id) = plan.timber_frame.as_ref().and_then(|frame| {
        let member = frame
            .members
            .iter()
            .find(|member| member.solid == solid.id)?;
        frame
            .bays
            .iter()
            .find(|bay| bay.member_ids.contains(&member.id))?
            .wall
    }) {
        return plan.wall_assemblies.iter().find(|wall| wall.id == wall_id);
    }
    plan.wall_assemblies
        .iter()
        .find(|wall| wall.owner == solid.owner || wall.replaced_by_owner == Some(solid.owner))
}

pub(super) fn material_for_solid_body(
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    wall_material: Option<WallMaterialClass>,
) -> BuildingLodMaterial {
    let infill_material = wall_material.unwrap_or(match plan.wall_style {
        WallStyle::Brick => WallMaterialClass::CivilianMasonry,
        WallStyle::Stone => WallMaterialClass::FortifiedMasonry,
        WallStyle::TimberFrame | WallStyle::Plaster => WallMaterialClass::TimberInfill,
    });
    match solid.role {
        SolidRole::EdgeGuard
        | SolidRole::FrameMember
        | SolidRole::FrameSill
        | SolidRole::FramePost
        | SolidRole::FramePlate
        | SolidRole::FrameRail
        | SolidRole::FrameTie
        | SolidRole::FrameBrace
        | SolidRole::FrameJettyBeam
        | SolidRole::FrameKnagge
        | SolidRole::FrameGableMember
        | SolidRole::FrameDormerTrimmer
        | SolidRole::FrameOrnament
        | SolidRole::OpeningClosure
        | SolidRole::ChurchStairNewel
        | SolidRole::ChurchServiceLadder
        | SolidRole::ArtilleryBridgeBeam
        | SolidRole::ArtilleryBridgeDeck
        | SolidRole::ArtilleryGateMechanism => BuildingLodMaterial::Timber,
        SolidRole::FrameJoist
        | SolidRole::FrameGirder
        | SolidRole::BeamJoist
        | SolidRole::RoofFraming
        | SolidRole::RoofPlate => BuildingLodMaterial::InteriorTimber,
        SolidRole::LeadedGlazing => BuildingLodMaterial::Glass,
        SolidRole::FrameFloor
        | SolidRole::WalkSurface
        | SolidRole::DrainageChannel
        | SolidRole::DrainageFloor
        | SolidRole::GalleryFloor
        | SolidRole::Landing
        | SolidRole::CircuitWalk
        | SolidRole::ChurchFloor
        | SolidRole::ChurchBellFloor
        | SolidRole::ChurchVaultShell => BuildingLodMaterial::Floor,
        SolidRole::RoofFlashing
        | SolidRole::DefenseRoof
        | SolidRole::RoofEdgeTreatment
        | SolidRole::RoofGutter => BuildingLodMaterial::Roof(RoofMaterial::ClayTile),
        SolidRole::FrameInfill => BuildingLodMaterial::Wall(infill_material),
        SolidRole::OpeningHead if wall_material == Some(WallMaterialClass::RubbleMasonry) => {
            BuildingLodMaterial::DressedStone
        }
        SolidRole::OpeningJamb | SolidRole::OpeningHead
            if wall_material == Some(WallMaterialClass::InternalTimber) =>
        {
            BuildingLodMaterial::Timber
        }
        SolidRole::WallHost
        | SolidRole::OpeningJamb
        | SolidRole::OpeningSill
        | SolidRole::OpeningHead
        | SolidRole::OpeningSpandrel
        | SolidRole::OpeningReveal => {
            BuildingLodMaterial::Wall(wall_material.unwrap_or(WallMaterialClass::FortifiedMasonry))
        }
        _ => BuildingLodMaterial::Wall(WallMaterialClass::FortifiedMasonry),
    }
}
