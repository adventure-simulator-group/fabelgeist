//! Exterior triangle selection from authoritative wall and opening assemblies.
use super::*;
use crate::{ResolvedSolid, WallAssembly};
// Keep perpendicular jamb reveals and wall-end returns that close corners.
const EXTERIOR_FACE_DOT_TOLERANCE: f32 = 0.001;
const OUTWARD_FACE_DOT_MINIMUM: f32 = 0.5;

pub(super) fn append_facades(
    lod: &mut BuildingLod,
    plan: &BuildingPlan,
    excluded: &std::collections::BTreeSet<crate::ResolvedItemId>,
) {
    let (contexts, openings) = facade_contexts(&lod.facade_runs, plan, excluded);
    let compiler = crate::detail::SolidDetailCompiler::new(plan);
    // Real inner skins enclose views through open civilian windows.
    let retain_inner_skin = plan.church.is_none() && plan.small_church.is_none();
    let minimum_dot = if plan.small_church.is_some() {
        OUTWARD_FACE_DOT_MINIMUM
    } else {
        -EXTERIOR_FACE_DOT_TOLERANCE
    };
    let mut detail = crate::BuildingDetail { meshes: Vec::new() };
    for solid in &plan.resolved_geometry.solids {
        let Some(outwards) = contexts.get(&solid.id) else {
            continue;
        };
        for mut mesh in compiler.compile(solid).meshes {
            mesh.indices = mesh
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|indices| {
                    let normal = mesh.vertices[indices[0] as usize].normal;
                    retain_inner_skin
                        || outwards
                            .iter()
                            .any(|outward| normal.dot(*outward) >= minimum_dot)
                })
                .flatten()
                .copied()
                .collect();
            detail.meshes.push(mesh);
        }
    }
    for bar in crate::compile_window_bars(plan)
        .iter()
        .filter(|bar| openings.contains(&bar.opening))
    {
        detail
            .meshes
            .extend(crate::detail::compile_bar_detail(bar).meshes);
    }
    crate::detail::resolve_masonry_surfaces(&mut detail);
    for mesh in detail.meshes {
        let target = lod.mesh_mut(mesh.material);
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let vertices = triangle.map(|index| mesh.vertices[index as usize]);
            target.push_triangle(
                vertices.map(|v| v.position),
                vertices[0].normal,
                vertices.map(|v| v.uv),
            );
        }
    }
}

type FacadeContexts = std::collections::BTreeMap<crate::ResolvedItemId, Vec<Vec3>>;
fn facade_contexts(
    runs: &[FacadeRun],
    plan: &BuildingPlan,
    excluded: &std::collections::BTreeSet<crate::ResolvedItemId>,
) -> (
    FacadeContexts,
    std::collections::BTreeSet<crate::OpeningAssemblyId>,
) {
    let ids = runs
        .iter()
        .flat_map(|run| run.source_walls.iter())
        .copied()
        .collect::<Vec<_>>();
    let mut contexts = std::collections::BTreeMap::<_, Vec<Vec3>>::new();
    let mut openings = std::collections::BTreeSet::new();
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
            openings.insert(opening.id);
        }
        if let Some(frame) = &plan.timber_frame {
            for bay in frame.bays.iter().filter(|bay| bay.wall == Some(wall.id)) {
                solid_ids.extend(
                    frame
                        .members
                        .iter()
                        .filter(|member| bay.member_ids.contains(&member.id))
                        .map(|member| member.solid),
                );
            }
        }
        for id in solid_ids.difference(excluded) {
            contexts.entry(*id).or_default().push(Vec3::new(
                wall.frame.outward.x,
                0.0,
                wall.frame.outward.y,
            ));
        }
    }
    (contexts, openings)
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
