use super::*;
use crate::{BuildingArchetype, BuildingProgram, compile_building_lod, generate};

fn triangles(meshes: &[LodMesh]) -> usize {
    meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
}

#[test]
fn component_instances_preserve_detailed_and_facade_surfaces() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkCottage,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        let plan = generate(&BuildingProgram::fixture(
            archetype,
            fabelgeist_determinism::Seed::from_u64(42),
        ))
        .unwrap();
        let kit = BuildingKit::new(&plan).unwrap();
        assert!(!kit.instances.is_empty());
        for instance in &kit.instances {
            let solid = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == instance.source)
                .unwrap();
            let expected = compile_solid_detail(&plan, solid).unwrap();
            let actual = instance.component.meshes();
            assert_eq!(actual.len(), expected.meshes.len());
            for (a, b) in actual.iter().zip(&expected.meshes) {
                assert_eq!(a.material, b.material);
                assert_eq!(a.indices, b.indices);
                assert_eq!(a.vertices.len(), b.vertices.len());
                for (a, b) in a.vertices.iter().zip(&b.vertices) {
                    assert!(
                        instance
                            .transform
                            .transform_point3(a.position)
                            .abs_diff_eq(b.position, 0.00001)
                    );
                    assert!(
                        instance
                            .transform
                            .transform_vector3(a.normal)
                            .abs_diff_eq(b.normal, 0.00001)
                    );
                    assert!((a.uv + instance.uv_offset).abs_diff_eq(b.uv, 0.00001));
                }
            }
        }
        for facade in [false, true] {
            let (expected, residual) = if facade {
                (
                    compile_building_lod(&plan, crate::BuildingLodLevel::Facade)
                        .unwrap()
                        .meshes,
                    kit.facade().unwrap().meshes,
                )
            } else {
                (
                    compile_building_detail(&plan).unwrap().meshes,
                    kit.detail().unwrap().meshes,
                )
            };
            let instanced: usize = kit
                .instances
                .iter()
                .filter(|instance| !facade || instance.facade)
                .map(|instance| triangles(&instance.component.meshes()))
                .sum();
            assert_eq!(
                triangles(&expected),
                triangles(&residual) + instanced,
                "{archetype:?}, facade={facade}"
            );
        }
    }
}
