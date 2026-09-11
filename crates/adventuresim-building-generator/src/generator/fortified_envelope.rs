//! Resolve the shared round-tower envelope and its bonded gatehouse junctions.
use super::*;

pub(super) fn resolve(
    program: &BuildingProgram,
    towers: &[RoundTower],
    crowns: &[CrownAssembly],
    defenses: &[ProjectedDefenseAssembly],
    walls: &mut Vec<crate::WallAssembly>,
    openings: &mut Vec<crate::OpeningAssembly>,
    geometry: &mut ResolvedGeometry,
) {
    if !matches!(
        program.archetype,
        BuildingArchetype::CastleGatehouse
            | BuildingArchetype::CourtyardCastle
            | BuildingArchetype::WalledKeep
            | BuildingArchetype::ArtilleryRondelCastle
    ) {
        return;
    }
    resolve_round_tower_wall_assemblies(towers, crowns, walls, geometry);
    if matches!(
        program.archetype,
        BuildingArchetype::CastleGatehouse | BuildingArchetype::CourtyardCastle
    ) {
        replace_storey_wall_sources_inside_round_towers(towers, walls, openings, geometry);
    }
    if program.archetype == BuildingArchetype::CastleGatehouse {
        resolve_gatehouse_tower_chord_bonds(towers, defenses, walls, geometry);
    }
}
