use super::*;
use crate::{
    BuildingLodLevel, audit_plan, compile_building_collision, compile_building_detail,
    compile_building_lod, generate, settlement_archetype,
};

fn recipe(kind: WorkplaceKind, size: ServiceBuildingSize, seed: u64) -> BuildingProgram {
    BuildingProgram::settlement(settlement_archetype(kind.usage()), Some(kind.usage()), seed)
        .with_service_size(size)
}

#[test]
fn workplace_matrix_has_clear_passages_and_shared_geometry() {
    for kind in WorkplaceKind::ALL {
        for size in [
            ServiceBuildingSize::Small,
            ServiceBuildingSize::Medium,
            ServiceBuildingSize::Large,
        ] {
            for seed in [0, 42, 101] {
                let program = recipe(kind, size, seed);
                let plan = generate(&program)
                    .unwrap_or_else(|error| panic!("{kind:?} {size:?} {seed}: {error:?}"));
                let work = plan.workplace.as_ref().unwrap();
                assert!(!work.passages.is_empty());
                let collision = compile_building_collision(&plan);
                for part in work.parts.iter().filter(|part| {
                    !matches!(
                        part.feature,
                        WorkplaceFeature::Boarding | WorkplaceFeature::ProcessLiquid
                    )
                }) {
                    assert!(
                        collision
                            .cuboids
                            .iter()
                            .any(|cuboid| cuboid.source == part.solid),
                        "{kind:?} {:?} lacks collision",
                        part.feature
                    );
                }
                let detail = compile_building_detail(&plan);
                assert!(!detail.meshes.is_empty());
                assert_gable_uvs(&detail.meshes, work, program.storey_height_metres);
                for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
                    let lod = compile_building_lod(&plan, level);
                    assert_gable_uvs(&lod.meshes, work, program.storey_height_metres);
                    assert!(
                        lod.meshes.iter().any(|mesh| matches!(
                            mesh.material,
                            crate::BuildingLodMaterial::Roof(_)
                        )),
                        "{kind:?} roof finish disappeared at {level:?}"
                    );
                    for part in work.parts.iter().filter(|part| part.silhouette) {
                        let solid = plan
                            .resolved_geometry
                            .solids
                            .iter()
                            .find(|solid| solid.id == part.solid)
                            .unwrap();
                        let corner = solid.centre
                            + super::assembly::contact::rotation(solid) * (solid.size * 0.5);
                        assert!(
                            lod.meshes
                                .iter()
                                .flat_map(|mesh| &mesh.vertices)
                                .any(|vertex| vertex.position.distance(corner) < 0.001),
                            "{kind:?} {:?} vanished at {level:?}",
                            part.feature
                        );
                    }
                    assert!(lod.meshes.iter().any(|mesh| {
                        mesh.vertices
                            .iter()
                            .any(|v| v.position.y > program.storey_height_metres)
                    }));
                }
                assert_eq!(
                    program.plot_dimensions_metres(),
                    work.plot_dimensions_metres
                );
            }
        }
    }
}

fn assert_gable_uvs(meshes: &[crate::LodMesh], work: &WorkplacePlan, eaves: f32) {
    let mut checked = 0;
    for mesh in meshes
        .iter()
        .filter(|mesh| mesh.material == work.gable_material().render_material())
    {
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let vertices = triangle.map(|index| &mesh.vertices[index as usize]);
            if vertices
                .iter()
                .any(|vertex| vertex.position.y > eaves + 0.1)
                && vertices[0].normal.y.abs() < 0.9
            {
                let area = (vertices[1].uv - vertices[0].uv)
                    .perp_dot(vertices[2].uv - vertices[0].uv)
                    .abs();
                assert!(
                    area > 1.0e-7,
                    "{:?} has a collapsed gable texture",
                    work.kind
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "{:?} lacks textured gables", work.kind);
}

#[test]
fn capacity_changes_working_space_and_roundtrips_recipe() {
    for kind in WorkplaceKind::ALL {
        let range = kind.usage().definition().capacity;
        let small = ServiceBuildingSize::for_capacity(kind.usage(), range.minimum).unwrap();
        let large = ServiceBuildingSize::for_capacity(kind.usage(), range.maximum).unwrap();
        let small = recipe(kind, small, 42);
        let large = recipe(kind, large, 42);
        assert!(small.plot_dimensions_metres().y < large.plot_dimensions_metres().y);
        let encoded = serde_json::to_string(&large).unwrap();
        let restored: BuildingProgram = serde_json::from_str(&encoded).unwrap();
        assert_eq!(large, restored);
    }
}

#[test]
fn workplace_audit_rejects_blocked_passage_and_missing_equipment() {
    let mut plan = generate(&recipe(
        WorkplaceKind::Stable,
        ServiceBuildingSize::Small,
        42,
    ))
    .unwrap();
    let workplace = plan.workplace.as_ref().unwrap();
    let stall = workplace
        .parts
        .iter()
        .find(|part| part.feature == WorkplaceFeature::Stall)
        .unwrap()
        .solid;
    let passage = &workplace.passages[0];
    let centre = (passage.min + passage.max) * 0.5;
    plan.resolved_geometry
        .solids
        .iter_mut()
        .find(|solid| solid.id == stall)
        .unwrap()
        .centre = centre;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "workplace_blocked_passage")
    );
    plan.workplace
        .as_mut()
        .unwrap()
        .parts
        .retain(|part| part.feature != WorkplaceFeature::Stall);
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "workplace_missing_function")
    );
}
