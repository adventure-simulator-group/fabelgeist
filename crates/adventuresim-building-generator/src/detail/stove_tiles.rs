//! Recessed ceramic joints inside the existing stove shell; no new collision.
use super::*;

const TILE_TARGET_METRES: f32 = 0.20;
const JOINT_WIDTH_METRES: f32 = 0.003;
const JOINT_DEPTH_METRES: f32 = 0.003;
const FACE_TOLERANCE_METRES: f32 = 0.0001;

pub(super) fn append(
    detail: &mut BuildingDetail,
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
) -> bool {
    let Some(heating) = &plan.domestic_heating else {
        return false;
    };
    if !heating
        .parts
        .iter()
        .any(|part| part.solid == solid.id && part.kind == crate::HeatingPartKind::TiledStove)
    {
        return false;
    }
    let bounds =
        plan.resolved_geometry
            .solids
            .iter()
            .filter(|s| {
                heating.parts.iter().any(|part| {
                    part.solid == s.id && part.kind == crate::HeatingPartKind::TiledStove
                })
            })
            .map(ResolvedSolid::cuboid_bounds)
            .fold(
                (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
                |(min, max), b| (min.min(b.min), max.max(b.max)),
            );
    let kitchen = Vec3::new(heating.kitchen_axis.x, 0.0, heating.kitchen_axis.y);
    let mut plain = BuildingDetail { meshes: Vec::new() };
    append_oriented_cuboid(&mut plain, BuildingLodMaterial::GlazedTile, solid, None);
    for mesh in plain.meshes {
        for face in mesh.vertices.as_chunks::<4>().0 {
            let normal = face[0].normal;
            let axis = if normal.x.abs() > 0.5 {
                0
            } else if normal.y.abs() > 0.5 {
                1
            } else {
                2
            };
            let outside_plane = if normal[axis] > 0.0 {
                bounds.1[axis]
            } else {
                bounds.0[axis]
            };
            let exposed = normal.dot(kitchen) < 0.5
                && normal.y >= 0.0
                && (face[0].position[axis] - outside_plane).abs() < FACE_TOLERANCE_METRES;
            if exposed {
                append_face(detail, face, bounds, axis);
            } else {
                detail.mesh_mut(mesh.material).push_quad(
                    face.map(|v| v.position),
                    normal,
                    face.map(|v| v.uv),
                );
            }
        }
    }
    true
}

fn append_face(
    detail: &mut BuildingDetail,
    face: &[crate::LodVertex; 4],
    bounds: (Vec3, Vec3),
    axis: usize,
) {
    let (u, v) = match axis {
        0 => (2, 1),
        1 => (0, 2),
        _ => (0, 1),
    };
    let project = |p: Vec3| Vec2::new(p[u], p[v]);
    let global_min = project(bounds.0);
    let global_max = project(bounds.1);
    let face_min = face
        .iter()
        .map(|p| project(p.position))
        .fold(Vec2::splat(f32::INFINITY), Vec2::min);
    let face_max = face
        .iter()
        .map(|p| project(p.position))
        .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
    let counts = ((global_max - global_min) / TILE_TARGET_METRES).ceil();
    let size = (global_max - global_min) / counts;
    let normal = face[0].normal;
    let point = |p: Vec2, depth: f32| {
        let mut result = face[0].position - normal * depth;
        result[u] = p.x;
        result[v] = p.y;
        result
    };
    // Joint backing is recessed into ceramic body, never a through-cut.
    detail.mesh_mut(BuildingLodMaterial::Earthenware).push_quad(
        face.map(|p| p.position - normal * JOINT_DEPTH_METRES),
        normal,
        face.map(|p| p.uv),
    );
    for row in 0..counts.y as usize {
        for column in 0..counts.x as usize {
            let tile_min = global_min
                + Vec2::new(column as f32, row as f32) * size
                + Vec2::splat(JOINT_WIDTH_METRES * 0.5);
            let tile_max = tile_min + size - Vec2::splat(JOINT_WIDTH_METRES);
            let min = tile_min.max(face_min);
            let max = tile_max.min(face_max);
            if min.x >= max.x || min.y >= max.y {
                continue;
            }
            let corners = [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)];
            let mesh = detail.mesh_mut(BuildingLodMaterial::GlazedTile);
            mesh.push_quad(
                corners.map(|p| point(p, 0.0)),
                normal,
                corners.map(|p| p / BUILDING_DETAIL_UV_METRES_PER_UNIT),
            );
            for edge in 0..4 {
                let edge_axis = if edge % 2 == 0 { 1 } else { 0 };
                let limit = if edge < 2 {
                    if edge == 0 { tile_min.y } else { tile_max.x }
                } else if edge == 2 {
                    tile_max.y
                } else {
                    tile_min.x
                };
                if (corners[edge][edge_axis] - limit).abs() > FACE_TOLERANCE_METRES {
                    continue;
                }
                let next = (edge + 1) % 4;
                let positions = [
                    point(corners[edge], 0.0),
                    point(corners[next], 0.0),
                    point(corners[next], JOINT_DEPTH_METRES),
                    point(corners[edge], JOINT_DEPTH_METRES),
                ];
                let mut side_normal = Vec3::ZERO;
                side_normal[if edge_axis == 0 { u } else { v }] =
                    if edge == 0 || edge == 3 { -1.0 } else { 1.0 };
                let length = positions[0].distance(positions[1]);
                mesh.push_quad(
                    positions,
                    side_normal,
                    [
                        Vec2::ZERO,
                        Vec2::new(length, 0.0),
                        Vec2::new(length, JOINT_DEPTH_METRES),
                        Vec2::new(0.0, JOINT_DEPTH_METRES),
                    ]
                    .map(|uv| uv / BUILDING_DETAIL_UV_METRES_PER_UNIT),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, HeatingPartKind, generate};

    #[test]
    fn tiled_shell_has_recessed_closed_joints_inside_canonical_solids() {
        for archetype in [
            BuildingArchetype::FachwerkCottage,
            BuildingArchetype::HallHouse,
        ] {
            for seed in [42, 47, 101] {
                let plan = generate(&BuildingProgram::fixture(archetype, seed)).unwrap();
                let heating = plan.domestic_heating.as_ref().unwrap();
                let solids = heating
                    .parts
                    .iter()
                    .filter(|p| p.kind == HeatingPartKind::TiledStove)
                    .map(|p| {
                        plan.resolved_geometry
                            .solids
                            .iter()
                            .find(|s| s.id == p.solid)
                            .unwrap()
                    })
                    .collect::<Vec<_>>();
                let mut triangles = 0;
                let mut recessed = 0;
                let mut joint_sides = 0;
                for solid in solids {
                    let bounds = solid.cuboid_bounds();
                    let detail = compile_solid_detail(&plan, solid);
                    for mesh in detail.meshes {
                        triangles += mesh.indices.len() / 3;
                        for vertex in &mesh.vertices {
                            assert!(
                                vertex
                                    .position
                                    .cmpge(bounds.min - Vec3::splat(0.0001))
                                    .all()
                            );
                            assert!(
                                vertex
                                    .position
                                    .cmple(bounds.max + Vec3::splat(0.0001))
                                    .all()
                            );
                        }
                        if mesh.material == BuildingLodMaterial::Earthenware {
                            recessed += mesh.indices.len() / 3;
                        }
                        for t in mesh.indices.as_chunks::<3>().0 {
                            let v = t.map(|i| mesh.vertices[i as usize]);
                            let geometric = (v[1].position - v[0].position)
                                .cross(v[2].position - v[0].position);
                            assert!(geometric.dot(v[0].normal) > 0.0);
                            let short_edge = [
                                v[0].position.distance(v[1].position),
                                v[1].position.distance(v[2].position),
                                v[2].position.distance(v[0].position),
                            ]
                            .into_iter()
                            .fold(f32::INFINITY, f32::min);
                            if (short_edge - JOINT_DEPTH_METRES).abs() < 0.0001 {
                                joint_sides += 1;
                            }
                        }
                    }
                }
                assert!(recessed >= 8 && joint_sides > 100);
                assert!((500..2000).contains(&triangles), "tile budget: {triangles}");
                assert!(
                    compile_heating_lod(&plan)
                        .meshes
                        .iter()
                        .all(|m| m.material != BuildingLodMaterial::GlazedTile
                            && m.material != BuildingLodMaterial::Earthenware)
                );
                assert!(crate::audit_plan(&plan).is_empty());
            }
        }
    }
}
