//! Exterior-only church representations, retaining accepted roofs and opening geometry.
use super::*;
use crate::{ResolvedSolid, RoofSurface, SmallChurchPlan, SolidRole, WallAssembly};

#[cfg(test)]
#[path = "small_church/tests.rs"]
mod tests;

const OUTWARD_FACE_DOT_MINIMUM: f32 = 0.5;

pub(super) fn compile(plan: &BuildingPlan, level: BuildingLodLevel) -> BuildingLod {
    let church = plan
        .small_church
        .as_ref()
        .expect("small church LOD requires its programme");
    let mut lod = BuildingLod {
        level,
        facade_runs: extract_facade_runs(plan),
        meshes: Vec::new(),
    };
    lod.facade_runs.retain(|run| {
        !run.source_walls
            .iter()
            .any(|id| church.bearing_walls.contains(id))
    });
    match level {
        BuildingLodLevel::Facade => append_facades(&mut lod, plan),
        BuildingLodLevel::Shell => append_wall_envelopes(&mut lod),
    }
    append_exterior_roofs(&mut lod, plan);
    append_belfry(&mut lod, plan, church);
    lod.meshes.retain(|mesh| !mesh.indices.is_empty());
    lod
}

fn append_facades(lod: &mut BuildingLod, plan: &BuildingPlan) {
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
        for solid in plan.resolved_geometry.solids.iter().filter(|solid| {
            wall.host_solids.contains(&solid.id)
                || plan
                    .opening_assemblies
                    .iter()
                    .any(|opening| opening.host_wall == wall.id && opening.owner == solid.owner)
        }) {
            append_outward_solid(lod, plan, solid, Some(wall));
        }
    }
}

fn append_outward_solid(
    lod: &mut BuildingLod,
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    wall: Option<&WallAssembly>,
) {
    let outward = wall.map(|wall| Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y));
    for mesh in crate::detail::compile_solid_detail(plan, solid).meshes {
        let target = lod.mesh_mut(mesh.material);
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let vertices = triangle.map(|index| mesh.vertices[index as usize]);
            if outward
                .is_some_and(|normal| vertices[0].normal.dot(normal) < OUTWARD_FACE_DOT_MINIMUM)
            {
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

fn append_exterior_roofs(lod: &mut BuildingLod, plan: &BuildingPlan) {
    for roof in &plan.roof_assemblies {
        for face in &roof.faces {
            for triangle in tessellate_roof_face(face)
                .into_iter()
                .filter(|triangle| triangle.surface != RoofSurface::Interior)
            {
                lod.mesh_mut(BuildingLodMaterial::Roof(face.material))
                    .push_triangle(
                        triangle.positions,
                        triangle.normal,
                        triangle
                            .positions
                            .map(|point| Vec2::new(point.x, point.z) / TEXTURE_REPEAT_METRES),
                    );
            }
        }
        for face in &roof.enclosure_faces {
            for triangle in tessellate_roof_enclosure(face) {
                // Church enclosures include a boarded belfry skirt. Keep its authored
                // finish: the generic shell's baked Fachwerk substitution adds infill.
                lod.mesh_mut(BuildingLodMaterial::Roof(face.material))
                    .push_triangle(
                        triangle.positions,
                        triangle.normal,
                        triangle.planar_uvs(TEXTURE_REPEAT_METRES),
                    );
            }
        }
    }
}

fn append_belfry(lod: &mut BuildingLod, plan: &BuildingPlan, church: &SmallChurchPlan) {
    let stage = church.belfry_stage;
    if lod.level == BuildingLodLevel::Shell {
        let min = Vec2::new(stage.min.x, stage.min.z);
        let max = Vec2::new(stage.max.x, stage.max.z);
        let corners = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];
        for edge in 0..4 {
            let start = corners[edge];
            let end = corners[(edge + 1) % 4];
            let tangent = (end - start).normalize();
            let outward = Vec3::new(tangent.y, 0.0, -tangent.x);
            let positions = [
                plan_vertex(start, stage.min.y),
                plan_vertex(end, stage.min.y),
                plan_vertex(end, stage.max.y),
                plan_vertex(start, stage.max.y),
            ];
            lod.mesh_mut(BuildingLodMaterial::InteriorTimber).push_quad(
                positions,
                outward,
                positions.map(|point| {
                    Vec2::new(Vec2::new(point.x, point.z).dot(tangent), point.y)
                        / TEXTURE_REPEAT_METRES
                }),
            );
        }
        return;
    }
    for solid in plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| church.fittings.contains(&solid.id))
    {
        if matches!(solid.role, SolidRole::ChurchFloor | SolidRole::ChurchPier) {
            continue;
        }
        let mut exposed = solid.clone();
        let bottom = (solid.centre.y - solid.size.y * 0.5).max(stage.min.y);
        let top = solid.centre.y + solid.size.y * 0.5;
        if top <= bottom {
            continue;
        }
        exposed.centre.y = (bottom + top) * 0.5;
        exposed.size.y = top - bottom;
        append_outward_solid(lod, plan, &exposed, None);
    }
}
