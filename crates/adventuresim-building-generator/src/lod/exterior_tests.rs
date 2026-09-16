use super::*;
use crate::{BuildingArchetype, BuildingProgram, OpeningUse, generate};
use std::collections::BTreeSet;

fn triangles(meshes: &[LodMesh]) -> usize {
    meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
}

fn hit(meshes: &[LodMesh], origin: Vec3, direction: Vec3, reach: f32) -> bool {
    meshes.iter().any(|mesh| {
        mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
            let [a, b, c] = indices.map(|i| mesh.vertices[i as usize].position);
            let edge = b - a;
            let side = c - a;
            let p = direction.cross(side);
            let determinant = edge.dot(p);
            if determinant < 0.000001 {
                return false;
            }
            let t = origin - a;
            let u = t.dot(p) / determinant;
            let q = t.cross(edge);
            let v = direction.dot(q) / determinant;
            let distance = side.dot(q) / determinant;
            u >= -0.00001
                && v >= -0.00001
                && u + v <= 1.00001
                && distance >= 0.0
                && distance <= reach
        })
    })
}

#[test]
fn civilian_facades_keep_real_apertures_reveals_and_materials_with_bounded_geometry() {
    for archetype in [
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::HallHouse,
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        for seed in [42, 47, 101] {
            let mut plan = generate(&BuildingProgram::fixture(archetype, seed)).unwrap();
            let facade = compile_building_lod(&plan, BuildingLodLevel::Facade);
            let detail = crate::compile_building_detail(&plan);
            let shell = compile_building_lod(&plan, BuildingLodLevel::Shell);
            eprintln!(
                "{archetype:?}/{seed}: detail {}, facade {}, shell {}",
                triangles(&detail.meshes),
                triangles(&facade.meshes),
                triangles(&shell.meshes)
            );
            assert!(triangles(&facade.meshes) < triangles(&detail.meshes));
            assert!(triangles(&shell.meshes) < triangles(&facade.meshes));
            assert!(triangles(&facade.meshes) < 38_000);
            assert!(facade.meshes.iter().all(|mesh| !matches!(
                mesh.material,
                BuildingLodMaterial::FacadeDetails | BuildingLodMaterial::Floor
            )));
            let inner = facade
                .meshes
                .iter()
                .filter(|m| m.material == BuildingLodMaterial::InteriorPlaster)
                .collect::<Vec<_>>();
            assert!(
                !inner.is_empty(),
                "open apertures require actual inward wall skins"
            );
            for mesh in inner {
                for indices in mesh.indices.as_chunks::<3>().0.iter().step_by(7) {
                    let vertices = indices.map(|i| mesh.vertices[i as usize]);
                    let point = vertices.map(|v| v.position).into_iter().sum::<Vec3>() / 3.0;
                    let normal = vertices[0].normal;
                    assert!(
                        hit(&facade.meshes, point + normal * 0.05, -normal, 0.051),
                        "inward wall face is missing or backface-culled"
                    );
                }
            }
            let excluded = plan
                .opening_assemblies
                .iter()
                .flat_map(|opening| opening.closure_solids.iter().copied())
                .collect();
            for opening in &mut plan.opening_assemblies {
                opening
                    .closure
                    .layers
                    .retain(|layer| *layer != crate::ClosureKind::IronBars);
            }
            let mut exterior = BuildingLod {
                level: BuildingLodLevel::Facade,
                facade_runs: extract_facade_runs(&plan),
                meshes: Vec::new(),
            };
            exterior::append_facades(&mut exterior, &plan, &excluded);
            let mut checked = 0;
            for opening in plan.opening_assemblies.iter().filter(|opening| {
                opening.use_kind == OpeningUse::Window
                    && exterior
                        .facade_runs
                        .iter()
                        .any(|run| run.source_walls.contains(&opening.host_wall))
            }) {
                let wall = plan
                    .wall_assemblies
                    .iter()
                    .find(|w| w.id == opening.host_wall)
                    .unwrap();
                let outward = Vec3::new(wall.frame.outward.x, 0.0, wall.frame.outward.y);
                let tangent = Vec3::new(wall.frame.tangent.x, 0.0, wall.frame.tangent.y);
                let centre = Vec3::new(
                    opening.frame.origin.x,
                    opening.sill_elevation_metres + opening.profile.clear_height_metres() * 0.4,
                    opening.frame.origin.y,
                );
                let width = opening.profile.interior_width_metres();
                for offset in [-0.3, 0.0, 0.3] {
                    assert!(
                        !hit(
                            &exterior.meshes,
                            centre + tangent * width * offset + outward * 0.7,
                            -outward,
                            0.7 + wall.thickness_metres * 0.45
                        ),
                        "{archetype:?}/{seed} {:?} has an opaque aperture",
                        opening.id
                    );
                }
                // From inside the empty aperture, look sideways into its real jamb return.
                assert!(
                    hit(&exterior.meshes, centre, tangent, width * 0.5 + 0.2),
                    "{archetype:?}/{seed} {:?} loses the reveal",
                    opening.id
                );
                checked += 1;
            }
            assert!(checked > 0);
        }
    }
}

#[test]
fn facade_reserves_only_operable_leaves_and_preserves_fixed_layers_and_bar_geometry() {
    let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
    let dynamic = crate::detail::dynamic_closure_solids(&plan);
    assert!(!dynamic.is_empty());
    let mut all = BuildingLod {
        level: BuildingLodLevel::Facade,
        facade_runs: extract_facade_runs(&plan),
        meshes: Vec::new(),
    };
    exterior::append_facades(&mut all, &plan, &BTreeSet::new());
    let mut reserved = BuildingLod {
        level: BuildingLodLevel::Facade,
        facade_runs: extract_facade_runs(&plan),
        meshes: Vec::new(),
    };
    exterior::append_facades(&mut reserved, &plan, &dynamic);
    assert!(triangles(&reserved.meshes) < triangles(&all.meshes));
    for mesh in reserved.meshes.iter().filter(|mesh| {
        matches!(
            mesh.material,
            BuildingLodMaterial::Glass | BuildingLodMaterial::Iron
        )
    }) {
        for vertex in &mesh.vertices {
            assert!(
                all.meshes
                    .iter()
                    .filter(|m| m.material == mesh.material)
                    .any(|m| m
                        .vertices
                        .iter()
                        .any(|v| v.position.abs_diff_eq(vertex.position, 0.00001)
                            && v.uv.abs_diff_eq(vertex.uv, 0.00001)))
            );
        }
    }
    let shell = compile_building_lod(&plan, BuildingLodLevel::Shell);
    let dynamic_shell = compile_static_building_lod(&plan, BuildingLodLevel::Shell);
    assert_eq!(
        serde_json::to_vec(&shell.meshes).unwrap(),
        serde_json::to_vec(&dynamic_shell.meshes).unwrap()
    );
}

#[test]
fn dynamic_facade_capability_excludes_coarse_hosts_and_replaced_workplace_walls() {
    for archetype in [
        BuildingArchetype::StorageRange,
        BuildingArchetype::CastleGatehouse,
    ] {
        let plan = generate(&BuildingProgram::fixture(archetype, 42)).unwrap();
        assert!(plan.facade_dynamic_openings().is_empty());
    }
    let plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
    let supported = plan.facade_dynamic_openings();
    assert!(!supported.is_empty());
    for id in supported {
        let opening = plan
            .opening_assemblies
            .iter()
            .find(|opening| opening.id == id)
            .unwrap();
        let host = plan
            .wall_assemblies
            .iter()
            .find(|wall| wall.id == opening.host_wall)
            .unwrap();
        assert!(host.replaced_by_owner.is_none());
        assert!(host.frame.outside_room.is_none());
    }
}

#[test]
fn a_workplace_owned_cell_excludes_the_entire_joined_run_from_dynamic_capability() {
    let mut plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 42)).unwrap();
    let run = extract_facade_runs(&plan)
        .into_iter()
        .find(|run| {
            run.source_walls.len() > 1
                && plan.opening_assemblies.iter().any(|opening| {
                    run.source_walls.contains(&opening.host_wall)
                        && plan.facade_dynamic_openings().contains(&opening.id)
                })
        })
        .unwrap();
    let usage = crate::WorkplaceKind::Smithy.usage();
    let mut work = generate(&BuildingProgram::settlement(
        crate::settlement_archetype(usage),
        Some(usage),
        42,
    ))
    .unwrap()
    .workplace
    .unwrap();
    work.walls = vec![run.source_walls[0]];
    plan.workplace = Some(work);
    let capability = plan.facade_dynamic_openings();
    let retained = compilation::retained_facade_runs(&plan);
    for wall in run.source_walls {
        assert!(retained.iter().all(|run| !run.source_walls.contains(&wall)));
        for opening in plan
            .opening_assemblies
            .iter()
            .filter(|opening| opening.host_wall == wall)
        {
            assert!(!capability.contains(&opening.id));
        }
    }
}
