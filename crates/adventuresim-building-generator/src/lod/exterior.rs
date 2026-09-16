//! Exterior triangle selection from authoritative wall and opening assemblies.
use super::*;
use crate::{ResolvedSolid, WallAssembly};
// Keep perpendicular jamb reveals and wall-end returns that close corners.
const EXTERIOR_FACE_DOT_TOLERANCE: f32 = 0.001;
const OUTWARD_FACE_DOT_MINIMUM: f32 = 0.5;

pub(super) fn append_facades(lod: &mut BuildingLod, plan: &BuildingPlan) {
    let ids = lod
        .facade_runs
        .iter()
        .flat_map(|run| run.source_walls.iter())
        .copied()
        .collect::<Vec<_>>();
    for wall in plan
        .wall_assemblies
        .iter()
        .filter(|wall| ids.contains(&wall.id))
    {
        // One church geometry owner may span many walls. Follow explicit solid
        // references so a facade never repeats unrelated vaults or stairwork.
        let mut solid_ids = wall
            .host_solids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for opening in plan
            .opening_assemblies
            .iter()
            .filter(|o| o.host_wall == wall.id)
        {
            solid_ids.extend(&opening.closure_solids);
        }
        for solid in plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|solid| solid_ids.contains(&solid.id))
        {
            append_outward_solid(lod, plan, solid, Some(wall));
        }
    }
}

pub(super) fn append_outward_solid(
    lod: &mut BuildingLod,
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    wall: Option<&WallAssembly>,
) {
    let minimum_dot = if plan.church.is_some() {
        -EXTERIOR_FACE_DOT_TOLERANCE
    } else {
        OUTWARD_FACE_DOT_MINIMUM
    };
    let outward = wall.map(|wall| Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y));
    for mesh in crate::detail::compile_solid_detail(plan, solid).meshes {
        let target = lod.mesh_mut(mesh.material);
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let vertices = triangle.map(|index| mesh.vertices[index as usize]);
            if outward.is_some_and(|normal| vertices[0].normal.dot(normal) < minimum_dot) {
                continue;
            }
            target.push_triangle(
                vertices.map(|v| v.position),
                vertices[0].normal,
                vertices.map(|v| v.uv),
            );
        }
    }
}
