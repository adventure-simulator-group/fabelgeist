//! Replace only the fire-wall patch; preserve the room boundary and other timber.
use super::placement::Placement;
use crate::*;
use bevy::math::Vec3;

pub(super) fn cut(
    plan: &mut BuildingPlan,
    placement: Placement,
) -> Result<(), crate::GenerationError> {
    let wall = plan
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == placement.site.wall)
        .ok_or(HeatingConstructionError::MissingWall {
            wall: placement.site.wall,
        })?;
    let cut = placement.site.bounds(
        crate::spatial_geometry::Displacement::from_metres(Vec3::new(-0.48, 0.0, -0.16))?,
        crate::spatial_geometry::Displacement::from_metres(Vec3::new(
            0.48,
            super::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
            0.16,
        ))?,
    )?;
    let old = wall.host_solids.clone();
    let solids = &mut plan.resolved_geometry.solids;
    let mut retained = Vec::new();
    for id in old {
        let index = solids
            .iter()
            .position(|s| s.id == id)
            .ok_or(HeatingConstructionError::MissingSolid { solid: id })?;
        let source = solids.remove(index);
        for (index, bounds) in subtract(source.cuboid_bounds()?, cut)?
            .into_iter()
            .enumerate()
        {
            let mut piece = source.clone();
            piece.id = ResolvedItemId(source.id.0 | ((index as u64) << 20));
            piece.centre = crate::spatial_geometry::Position::<crate::Architectural>::from_metres(
                (bounds.min().metres() + bounds.max().metres()) * 0.5,
            )?;
            piece.size = crate::spatial_geometry::CuboidDimensions::from_metres(
                bounds.max().metres() - bounds.min().metres(),
            )?;
            retained.push(piece.id);
            solids.push(piece);
        }
    }
    wall.host_solids = retained;

    Ok(())
}
fn subtract(
    source: SpatialBounds<Architectural>,
    cut: SpatialBounds<Architectural>,
) -> Result<Vec<SpatialBounds<Architectural>>, crate::GenerationError> {
    let min = source.min().metres().max(cut.min().metres());
    let max = source.max().metres().min(cut.max().metres());
    if (max - min).min_element() <= 0.001 {
        return Ok(vec![source]);
    }
    let mut pieces = Vec::new();
    let mut core_min = source.min().metres();
    let mut core_max = source.max().metres();
    for axis in 0..3 {
        if core_min[axis] < min[axis] - 0.001 {
            let mut piece_max = core_max;
            piece_max[axis] = min[axis];
            pieces.push(SpatialBounds::from_metres(core_min, piece_max)?);
        }
        if core_max[axis] > max[axis] + 0.001 {
            let mut piece_min = core_min;
            piece_min[axis] = max[axis];
            pieces.push(SpatialBounds::from_metres(piece_min, core_max)?);
        }
        core_min[axis] = min[axis];
        core_max[axis] = max[axis];
    }
    Ok(pieces)
}
