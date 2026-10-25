//! Resolve a selected large church programme before shared roof construction.
use super::*;

pub(super) fn resolve(
    program: &BuildingProgram,
    towers: &[SquareTower],
    walls: &mut Vec<crate::WallAssembly>,
    openings: &mut Vec<crate::OpeningAssembly>,
    stairs: &mut Vec<Stair>,
    geometry: &mut ResolvedGeometry,
) -> Result<Option<crate::ChurchAssembly>, crate::GenerationError> {
    if program.church_program.is_none() {
        return Ok(None);
    }
    suppress_cathedral_legacy_storey_walls(walls, openings, geometry);
    resolve_cathedral_bell_stage(towers, walls, openings, geometry)?;
    Ok(Some(resolve_church_assembly(
        program, walls, openings, stairs, geometry,
    )?))
}
