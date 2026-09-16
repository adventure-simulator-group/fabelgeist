//! Roof windows and timber framing resolve against the same roof assembly.
use super::*;
pub(super) fn resolve(
    program: &BuildingProgram,
    edits: &[BuildingEdit],
    roofs: &[RoofPiece],
    roof_dormers: &[RoofDormer],
    towers: &[RoundTower],
    square_towers: &[SquareTower],
    stairs: &mut [Stair],
    wall_assemblies: &mut Vec<crate::WallAssembly>,
    opening_assemblies: &mut Vec<crate::OpeningAssembly>,
    resolved_geometry: &mut ResolvedGeometry,
) -> Result<(Vec<RoofAssembly>, Option<crate::TimberFrameAssembly>), GenerationError> {
    let mut roof_assemblies = resolve_roof_assemblies(
        program,
        roofs,
        roof_dormers,
        towers,
        square_towers,
        stairs,
        wall_assemblies,
        opening_assemblies,
        resolved_geometry,
    )?;
    resolve_roof_child_front_openings(
        program,
        roof_dormers,
        &mut roof_assemblies,
        wall_assemblies,
        opening_assemblies,
        resolved_geometry,
    );
    let timber_frame = resolve_timber_frame_assembly(
        program,
        edits,
        wall_assemblies,
        opening_assemblies,
        roofs,
        roof_dormers,
        stairs,
        &mut roof_assemblies,
        resolved_geometry,
    );
    Ok((roof_assemblies, timber_frame))
}
