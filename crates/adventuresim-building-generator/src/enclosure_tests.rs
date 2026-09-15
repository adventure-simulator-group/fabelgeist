//! Regressions against compiled triangles, independent of assembly declarations.
use bevy::math::Vec3;

use crate::*;

fn hits(meshes: &[LodMesh], start: Vec3, end: Vec3) -> bool {
    let direction = end - start;
    meshes.iter().any(|mesh| {
        mesh.indices.as_chunks::<3>().0.iter().any(|indices| {
            let [a, b, c] = [0, 1, 2].map(|i| mesh.vertices[indices[i] as usize].position);
            let edge = b - a;
            let cross = direction.cross(c - a);
            let determinant = edge.dot(cross);
            if determinant.abs() < 1.0e-7 {
                return false;
            }
            let offset = start - a;
            let u = offset.dot(cross) / determinant;
            let q = offset.cross(edge);
            let v = direction.dot(q) / determinant;
            let t = (c - a).dot(q) / determinant;
            u >= -1.0e-5 && v >= -1.0e-5 && u + v <= 1.00001 && (0.0..=1.0).contains(&t)
        })
    })
}

#[test]
fn enclosure_mesh_closes_projecting_storey_corners() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        let program = BuildingProgram::fixture(archetype, 47);
        let plan = generate(&program).unwrap();
        let representations = [
            compile_building_detail(&plan).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Facade).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Shell).meshes,
        ];
        let (width, depth) = plan.footprint.dimensions();
        let size = Vec3::new(f32::from(width), 0.0, f32::from(depth)) * CELL_SIZE_METRES;
        for (x, z) in [(0.0, 0.0), (size.x, 0.0), (size.x, size.z), (0.0, size.z)] {
            let outward = Vec3::new(
                if x == 0.0 { -1.0 } else { 1.0 },
                0.0,
                if z == 0.0 { -1.0 } else { 1.0 },
            );
            for fraction in [0.2, 0.4, 0.6, 0.8] {
                let corner = Vec3::new(x, plan.storey_height_metres * (1.0 + fraction), z);
                assert!(
                    representations.iter().all(|meshes| hits(
                        meshes,
                        corner - outward * 0.15,
                        corner + outward * 0.6
                    )),
                    "{archetype:?} corner {corner:?} is open"
                );
            }
        }
    }
}

#[test]
fn enclosure_mesh_closes_gable_rakes() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        let plan = generate(&BuildingProgram::fixture(archetype, 47)).unwrap();
        let representations = [
            compile_building_detail(&plan).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Facade).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Shell).meshes,
        ];
        let roof = &plan.roof_assemblies[0];
        let recipe = plan.roofs[0];
        for sign in [-1.0, 1.0] {
            let x = recipe.centre.x + recipe.size.x * 0.25;
            let z = recipe.centre.y + recipe.size.y * 0.5 * sign;
            let face = roof.faces.iter().find(|f| f.plane.normal.x > 0.1).unwrap();
            let n = face.plane.normal;
            let y = -(n.x * x + n.z * z + face.plane.constant + face.thickness_metres) / n.y - 0.03;
            let point = Vec3::new(x, y, z);
            assert!(
                representations.iter().all(|meshes| hits(
                    meshes,
                    point - Vec3::Z * 0.6,
                    point + Vec3::Z * 0.6
                )),
                "{archetype:?} gable rake {point:?} is open"
            );
        }
    }
}

#[test]
fn enclosure_audit_rejects_lowered_and_narrowed_gables() {
    let fixture = generate(&BuildingProgram::fixture(
        BuildingArchetype::FachwerkMerchantHouse,
        47,
    ))
    .unwrap();
    for narrow in [false, true] {
        let mut plan = fixture.clone();
        let centre = plan.roofs[0].centre.x;
        for face in &mut plan.roof_assemblies[0].enclosure_faces {
            for point in &mut face.polygon {
                if narrow {
                    point.x = centre + (point.x - centre) * 0.8;
                } else {
                    point.y -= 1.0;
                }
            }
        }
        assert!(
            audit_plan(&plan)
                .iter()
                .any(|issue| issue.code == crate::audit::enclosure::GABLE_GAP)
        );
    }
}

#[test]
fn enclosure_audit_rejects_missing_corner_material_without_a_declared_bond() {
    let mut plan = generate(&BuildingProgram::fixture(BuildingArchetype::TownHouse, 47)).unwrap();
    let projection = plan.upper_storey_projection_metres;
    let corner = Vec3::new(-projection, plan.storey_height_metres * 1.5, -projection);
    // Remove the post and backing infill to create an actual enclosure defect.
    // No declared bond is needed to identify this required corner.
    let post = plan
        .resolved_geometry
        .solids
        .iter()
        .filter(|solid| solid.role == SolidRole::FramePost)
        .min_by(|a, b| {
            a.centre
                .distance_squared(corner)
                .total_cmp(&b.centre.distance_squared(corner))
        })
        .unwrap()
        .id;
    let corner_plan = bevy::math::Vec2::new(corner.x, corner.z);
    let removed = plan
        .wall_assemblies
        .iter()
        .filter(|wall| wall.storey_level == 1)
        .filter(|wall| {
            [-1.0, 1.0].into_iter().any(|side| {
                (wall.frame.origin + wall.frame.tangent * side * wall.length_metres * 0.5)
                    .distance(corner_plan)
                    < 0.001
            })
        })
        .flat_map(|wall| wall.host_solids.iter().copied())
        .chain([post])
        .collect::<std::collections::BTreeSet<_>>();
    plan.resolved_geometry
        .solids
        .retain(|solid| !removed.contains(&solid.id));
    plan.resolved_geometry.junction_bonds.clear();
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == crate::audit::enclosure::WALL_GAP)
    );
}

#[test]
fn enclosure_actual_fachwerk_scene_passes() {
    let scene: serde_json::Value = serde_json::from_str(include_str!(
        "../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let program = scene["buildings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|building| {
            serde_json::from_value::<BuildingProgram>(building["program"].clone()).unwrap()
        })
        .find(|program| program.archetype == BuildingArchetype::FachwerkMerchantHouse)
        .expect("city fixture retains a playable Fachwerk merchant house");
    assert!(generate(&program).is_ok());
}
