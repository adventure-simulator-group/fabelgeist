use super::*;

#[test]
fn recipe_shells_keep_roof_height_and_outward_winding_with_bounded_geometry() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::HallHouse,
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::FachwerkMerchantHouse,
        BuildingArchetype::StorageRange,
    ] {
        let program = BuildingProgram::fixture(archetype, 42);
        let shell = compile_program_shell(&program).unwrap();
        let plan = crate::generate(&program).unwrap();
        let roof_height = plan
            .roof_assemblies
            .iter()
            .flat_map(|roof| &roof.faces)
            .flat_map(|face| &face.polygon)
            .map(|p| p.y)
            .fold(0.0, f32::max);
        let actual_height = shell
            .meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .map(|vertex| vertex.position.y)
            .fold(0.0, f32::max);
        assert!(
            (roof_height - actual_height).abs() < 0.01,
            "{archetype:?}: {roof_height} vs {actual_height}"
        );
        assert!(
            shell
                .meshes
                .iter()
                .map(|mesh| mesh.indices.len() / 3)
                .sum::<usize>()
                < 500
        );
        for mesh in &shell.meshes {
            for triangle in mesh.indices.as_chunks::<3>().0 {
                let [a, b, c] = triangle.map(|i| mesh.vertices[i as usize]);
                assert!(a.position.is_finite() && a.uv.is_finite());
                assert!(
                    (b.position - a.position)
                        .cross(c.position - a.position)
                        .dot(a.normal)
                        > 0.0
                );
            }
        }
    }
}

#[test]
fn specialized_architecture_keeps_its_semantic_compiler() {
    for archetype in [
        BuildingArchetype::ParishChurch,
        BuildingArchetype::Cathedral,
        BuildingArchetype::Workplace,
        BuildingArchetype::CourtyardCastle,
    ] {
        assert!(compile_program_shell(&BuildingProgram::fixture(archetype, 42)).is_none());
    }
}

#[test]
#[ignore = "explicit generation-phase benchmark"]
fn benchmark_program_shell() {
    let program = BuildingProgram::fixture(BuildingArchetype::FachwerkMerchantHouse, 42);
    let plan = crate::generate(&program).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(compile_building_lod(&plan, BuildingLodLevel::Shell));
    }
    let semantic = start.elapsed();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(compile_program_shell(&program).unwrap());
    }
    eprintln!(
        "100 shells: semantic={semantic:?}, direct={:?}",
        start.elapsed()
    );
}
