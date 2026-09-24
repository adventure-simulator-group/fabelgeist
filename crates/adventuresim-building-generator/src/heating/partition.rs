//! Replace only the fire-wall patch; preserve the room boundary and other timber.
use super::placement::Placement;
use crate::*;
use bevy::math::Vec3;

pub(super) fn cut(plan: &mut BuildingPlan, placement: Placement) {
    let wall = plan
        .wall_assemblies
        .iter_mut()
        .find(|w| w.id == placement.wall)
        .unwrap();
    let cut = placement.bounds(
        Vec3::new(-0.48, 0.0, -0.16),
        Vec3::new(0.48, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.16),
    );
    let old = wall.host_solids.clone();
    let solids = &mut plan.resolved_geometry.solids;
    let mut retained = Vec::new();
    for id in old {
        let index = solids.iter().position(|s| s.id == id).unwrap();
        let source = solids.remove(index);
        for (index, bounds) in subtract(source.cuboid_bounds(), cut)
            .into_iter()
            .enumerate()
        {
            let mut piece = source.clone();
            piece.id = ResolvedItemId(source.id.0 | ((index as u64) << 20));
            piece.centre = (bounds.min + bounds.max) * 0.5;
            piece.size = bounds.max - bounds.min;
            retained.push(piece.id);
            solids.push(piece);
        }
    }
    wall.host_solids = retained;
}
fn subtract(source: ResolvedBounds, cut: ResolvedBounds) -> Vec<ResolvedBounds> {
    let min = source.min.max(cut.min);
    let max = source.max.min(cut.max);
    if (max - min).min_element() <= 0.001 {
        return vec![source];
    }
    let mut pieces = Vec::new();
    let mut core = source;
    for axis in 0..3 {
        if core.min[axis] < min[axis] - 0.001 {
            let mut piece = core;
            piece.max[axis] = min[axis];
            pieces.push(piece);
        }
        if core.max[axis] > max[axis] + 0.001 {
            let mut piece = core;
            piece.min[axis] = max[axis];
            pieces.push(piece);
        }
        core.min[axis] = min[axis];
        core.max[axis] = max[axis];
    }
    pieces
}
