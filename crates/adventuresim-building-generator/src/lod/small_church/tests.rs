use super::*;
use crate::{
    BuildingProgram, ServiceBuildingSize, compile_building_detail, generate, settlement_archetype,
};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

fn plan(usage: BuildingUse, size: ServiceBuildingSize) -> BuildingPlan {
    generate(
        &BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
            .with_service_size(size),
    )
    .unwrap()
}

fn triangles(meshes: &[LodMesh]) -> usize {
    meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
}

#[test]
fn church_lods_reduce_triangles_without_losing_canonical_roof_silhouettes() {
    for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
        for size in [
            ServiceBuildingSize::Small,
            ServiceBuildingSize::Medium,
            ServiceBuildingSize::Large,
        ] {
            let plan = plan(usage, size);
            let detail = compile_building_detail(&plan);
            let facade = compile(&plan, BuildingLodLevel::Facade);
            let shell = compile(&plan, BuildingLodLevel::Shell);
            let (detail_count, facade_count, shell_count) = (
                triangles(&detail.meshes),
                triangles(&facade.meshes),
                triangles(&shell.meshes),
            );
            eprintln!(
                "{usage:?} {size:?}: detail={detail_count}, facade={facade_count}, shell={shell_count} triangles"
            );
            assert!(
                facade_count * 2 < detail_count,
                "{usage:?} {size:?}: detail={detail_count}, facade={facade_count}"
            );
            assert!(
                shell_count * 2 < facade_count,
                "{usage:?} {size:?}: facade={facade_count}, shell={shell_count}"
            );
            for lod in [&facade, &shell] {
                assert!(
                    lod.meshes
                        .iter()
                        .all(|mesh| mesh.material != BuildingLodMaterial::Floor)
                );
                for point in plan
                    .roof_assemblies
                    .iter()
                    .flat_map(|roof| &roof.faces)
                    .flat_map(|face| &face.polygon)
                {
                    assert!(
                        lod.meshes
                            .iter()
                            .flat_map(|mesh| &mesh.vertices)
                            .any(|vertex| vertex.position.distance(*point) < 0.001),
                        "{usage:?} {size:?} {:?} lost canonical roof silhouette at {point:?}",
                        lod.level
                    );
                }
                let stage = plan.small_church.as_ref().unwrap().belfry_stage;
                assert!(
                    lod.meshes
                        .iter()
                        .filter(|mesh| matches!(
                            mesh.material,
                            BuildingLodMaterial::Timber | BuildingLodMaterial::InteriorTimber
                        ))
                        .flat_map(|mesh| &mesh.vertices)
                        .all(|vertex| vertex.position.y >= stage.min.y - 0.001
                            || mesh_is_door_vertex(&plan, vertex.position)),
                    "hidden floor-to-belfry supports remain in {:?}",
                    lod.level
                );
            }
        }
    }
}

fn mesh_is_door_vertex(plan: &BuildingPlan, point: Vec3) -> bool {
    plan.opening_assemblies
        .iter()
        .filter(|opening| opening.use_kind == crate::OpeningUse::Door)
        .any(|opening| {
            let offset = Vec2::new(point.x, point.z) - opening.frame.origin;
            offset.dot(opening.frame.tangent).abs()
                <= opening.profile.exterior_width_metres() * 0.5 + 0.01
                && offset.dot(opening.frame.outward).abs() < 1.0
        })
}

#[test]
fn facade_windows_remain_actual_openings_in_the_wall_surface() {
    let plan = plan(BuildingUse::ParishChurch, ServiceBuildingSize::Large);
    let facade = compile(&plan, BuildingLodLevel::Facade);
    let windows = plan
        .opening_assemblies
        .iter()
        .filter(|opening| opening.use_kind == crate::OpeningUse::Window)
        .collect::<Vec<_>>();
    assert!(!windows.is_empty());
    assert!(
        facade
            .meshes
            .iter()
            .any(|mesh| mesh.material == BuildingLodMaterial::Glass)
    );
    for opening in windows {
        let sample = Vec2::new(
            0.0,
            opening.sill_elevation_metres + opening.profile.clear_height_metres() * 0.5,
        );
        let outward = Vec3::new(opening.frame.outward.x, 0.0, opening.frame.outward.y);
        let wall = plan
            .wall_assemblies
            .iter()
            .find(|wall| wall.id == opening.host_wall)
            .unwrap();
        let exterior_plane =
            opening.frame.origin.dot(opening.frame.outward) + wall.thickness_metres * 0.5;
        for mesh in facade.meshes.iter().filter(|mesh| {
            matches!(
                mesh.material,
                BuildingLodMaterial::Wall(_) | BuildingLodMaterial::DressedStone
            )
        }) {
            for indices in mesh.indices.as_chunks::<3>().0 {
                let vertices = indices.map(|index| mesh.vertices[index as usize]);
                if vertices[0].normal.dot(outward) < 0.9
                    || (vertices[0].position.dot(outward) - exterior_plane).abs() > 0.05
                {
                    continue;
                }
                let projected = vertices.map(|vertex| {
                    Vec2::new(
                        (Vec2::new(vertex.position.x, vertex.position.z) - opening.frame.origin)
                            .dot(opening.frame.tangent),
                        vertex.position.y,
                    )
                });
                let sides = (0..3)
                    .map(|edge| {
                        (projected[(edge + 1) % 3] - projected[edge])
                            .perp_dot(sample - projected[edge])
                    })
                    .collect::<Vec<_>>();
                assert!(
                    !(sides.iter().all(|side| *side > 0.001)
                        || sides.iter().all(|side| *side < -0.001)),
                    "window {:?} is covered by facade masonry",
                    opening.id
                );
            }
        }
    }
}

#[test]
fn boarded_belfry_skirts_keep_their_exact_material_at_both_lod_levels() {
    for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
        let plan = plan(usage, ServiceBuildingSize::Small);
        let detail = compile_building_detail(&plan);
        let facade = compile(&plan, BuildingLodLevel::Facade);
        let shell = compile(&plan, BuildingLodLevel::Shell);
        let skirts = plan
            .roof_assemblies
            .iter()
            .flat_map(|roof| &roof.enclosure_faces)
            .filter(|face| face.material == RoofMaterial::TimberInfill)
            .collect::<Vec<_>>();
        assert!(
            !skirts.is_empty(),
            "{usage:?} has no canonical boarded belfry skirt"
        );
        for face in skirts {
            for triangle in tessellate_roof_enclosure(face) {
                for meshes in [&detail.meshes, &facade.meshes, &shell.meshes] {
                    assert!(meshes.iter().filter(|mesh| mesh.material == BuildingLodMaterial::Roof(face.material))
                        .any(|mesh| mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
                            let positions = indices.map(|index| mesh.vertices[index as usize].position);
                            triangle.positions.iter().all(|expected| positions.iter().any(|position| position.distance(*expected) < 0.001))
                        })), "{usage:?} changed the skirt's authored material on a canonical enclosure triangle");
                }
            }
        }
    }
}
