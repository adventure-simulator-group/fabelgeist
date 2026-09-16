//! Keep the load-bearing external masses in the urban church silhouette.
use super::*;
use crate::SolidRole;

pub(super) fn append_buttresses(lod: &mut BuildingLod, plan: &BuildingPlan) {
    if plan.church.is_none() {
        return;
    }
    for solid in plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| solid.role == SolidRole::WallButtress)
    {
        exterior::append_outward_solid(lod, plan, solid, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildingArchetype, BuildingProgram, ServiceBuildingSize, WallSourceId, generate};

    #[test]
    fn urban_church_lods_keep_exterior_clerestory_panels_and_buttress_masses() {
        let program = BuildingProgram::fixture(BuildingArchetype::ParishChurch, 42)
            .with_service_size(ServiceBuildingSize::Large);
        let plan = generate(&program).unwrap();
        let detail = crate::compile_building_detail(&plan);
        let count = |meshes: &[LodMesh]| meshes.iter().map(|m| m.indices.len() / 3).sum::<usize>();
        for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
            let lod = compile_building_lod(&plan, level);
            assert!(
                count(&lod.meshes) < count(&detail.meshes) / 2,
                "{level:?}: {} triangles versus {} detail triangles",
                count(&lod.meshes),
                count(&detail.meshes)
            );
            for wall in plan.wall_assemblies.iter().filter(|w| {
                matches!(
                    w.source,
                    WallSourceId::ChurchArcade { .. }
                        | WallSourceId::ChurchExterior { .. }
                        | WallSourceId::ChurchTowerFace { .. }
                        | WallSourceId::SquareTowerFace { .. }
                )
            }) {
                assert!(
                    lod.facade_runs
                        .iter()
                        .any(|r| r.source_walls.contains(&wall.id)),
                    "church weather face omitted from the exterior"
                );
                let point = wall.frame.origin + wall.frame.outward * wall.thickness_metres * 0.5;
                let point = Vec3::new(
                    point.x,
                    wall.base_elevation_metres + wall.height_metres - 0.1,
                    point.y,
                );
                let normal = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
                assert!(
                    covered(&lod, point, normal),
                    "church masonry missing at {point:?}"
                );
            }
            if level == BuildingLodLevel::Facade {
                for wall in plan.wall_assemblies.iter().filter(|w| {
                    lod.facade_runs
                        .iter()
                        .any(|r| r.source_walls.contains(&w.id))
                }) {
                    let outward = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
                    for solid in plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .filter(|s| wall.host_solids.contains(&s.id))
                    {
                        for mesh in crate::compile_solid_detail(&plan, solid).meshes {
                            if !matches!(mesh.material, BuildingLodMaterial::Wall(_)) {
                                continue;
                            }
                            for triangle in mesh.indices.as_chunks::<3>().0 {
                                let vertices = triangle.map(|i| mesh.vertices[i as usize]);
                                let normal = vertices[0].normal;
                                if normal.dot(outward) < -0.001 {
                                    continue;
                                }
                                let centre =
                                    vertices.iter().map(|v| v.position).sum::<Vec3>() / 3.0;
                                assert!(
                                    covered(&lod, centre, normal),
                                    "missing facade face or reveal at {centre:?}"
                                );
                            }
                        }
                    }
                }
            }
            for solid in plan
                .resolved_geometry
                .solids
                .iter()
                .filter(|s| s.role == SolidRole::WallButtress)
            {
                let detail = crate::compile_solid_detail(&plan, solid);
                for point in detail
                    .meshes
                    .iter()
                    .flat_map(|m| &m.vertices)
                    .map(|v| v.position)
                {
                    assert!(
                        lod.meshes
                            .iter()
                            .flat_map(|m| &m.vertices)
                            .any(|v| v.position.distance(point) < 0.001)
                    );
                }
            }
        }
    }

    fn covered(lod: &BuildingLod, point: Vec3, normal: Vec3) -> bool {
        lod.meshes
            .iter()
            .filter(|m| matches!(m.material, BuildingLodMaterial::Wall(_)))
            .any(|mesh| {
                mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
                    let vertices = indices.map(|i| mesh.vertices[i as usize]);
                    if vertices[0].normal.dot(normal) < 0.9
                        || (point - vertices[0].position).dot(normal).abs() > 0.02
                    {
                        return false;
                    }
                    let points = vertices.map(|v| v.position);
                    let signs = (0..3)
                        .map(|i| {
                            (points[(i + 1) % 3] - points[i])
                                .cross(point - points[i])
                                .dot(normal)
                        })
                        .collect::<Vec<_>>();
                    signs.iter().all(|s| *s >= -0.001) || signs.iter().all(|s| *s <= 0.001)
                })
            })
    }
}
