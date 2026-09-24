use super::*;
use crate::*;

fn principal(seed: u64) -> BuildingProgram {
    BuildingProgram::settlement(
        BuildingArchetype::ParishChurch,
        Some(BuildingUse::ParishChurch),
        seed,
    )
    .with_service_size(ServiceBuildingSize::Large)
}

#[test]
fn principal_parish_has_a_distinct_supported_programme_in_every_representation() {
    for seed in [42, 47, 101] {
        let program = principal(seed);
        let validated = BuildingProgram::validated_settlement(
            program.archetype,
            program.usage.unwrap(),
            seed,
            program.service_size,
        )
        .unwrap();
        assert_eq!(program, validated);
        let plan = generate(&program).unwrap();
        assert_eq!(plan.archetype, BuildingArchetype::ParishChurch);
        assert!(plan.small_church.is_none());
        let church = plan.church.as_ref().unwrap();
        assert_eq!(church.program, ChurchProgram::URBAN_BRICK_BASILICA);
        assert_eq!(program.frontage_direction(), Direction::West);
        let collision = compile_building_collision(&plan);
        let origin = collision.bounds.centre();
        let half = program.plot_dimensions_metres() * 0.5;
        let detail = compile_building_detail(&plan);
        for mesh in &detail.meshes {
            for vertex in &mesh.vertices {
                let p = vertex.position - origin;
                // Rotate the west-fronted physical plan into the street frame.
                assert!(
                    p.z.abs() <= half.x && p.x.abs() <= half.y,
                    "detail vertex {p:?} exceeds the reserved frontage envelope {half:?}"
                );
            }
        }
        for level in [BuildingLodLevel::Facade, BuildingLodLevel::Shell] {
            let lod = compile_building_lod(&plan, level);
            let highest = lod
                .meshes
                .iter()
                .flat_map(|m| &m.vertices)
                .map(|v| v.position.y)
                .reduce(f32::max)
                .unwrap();
            assert!(highest > church.datum.bell_floor_metres);
            let tower = &plan.square_towers[0];
            let in_tower = |p: bevy::math::Vec3| {
                (Vec2::new(p.x, p.z) - tower.centre)
                    .abs()
                    .cmple(tower.size * 0.5 + Vec2::splat(tower.roof.eave_metres))
                    .all()
            };
            let tower_top = |meshes: &[LodMesh]| {
                meshes
                    .iter()
                    .flat_map(|m| &m.vertices)
                    .filter(|v| in_tower(v.position))
                    .map(|v| v.position.y)
                    .reduce(f32::max)
                    .unwrap()
            };
            assert!((tower_top(&detail.meshes) - tower_top(&lod.meshes)).abs() < 0.15);
        }
        // Returning to a subordinate scale removes the large programme cleanly.
        let smaller = program.with_service_size(ServiceBuildingSize::Medium);
        assert!(generate(&smaller).unwrap().small_church.is_some());
    }
}

#[test]
fn principal_programme_rejects_missing_or_corrupt_authority() {
    let mut programme = principal(42);
    programme.footprint = Footprint::Rectangle {
        width: 20,
        depth: 14,
    };
    assert_eq!(
        generate(&programme).unwrap_err(),
        GenerationError::InvalidChurchProgram
    );
    let mut programme = principal(42);
    programme.church_program = None;
    assert_eq!(
        generate(&programme).unwrap_err(),
        GenerationError::InvalidChurchProgram
    );
    let mut programme = principal(42);
    programme.church_program.as_mut().unwrap().nave_bays = 3;
    assert_eq!(
        generate(&programme).unwrap_err(),
        GenerationError::InvalidChurchProgram
    );
    let mut programme = principal(42);
    programme.usage = Some(BuildingUse::Chapel);
    assert_eq!(
        generate(&programme).unwrap_err(),
        GenerationError::InvalidChurchProgram
    );
    let mut plan = generate(&principal(42)).unwrap();
    plan.church = None;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|i| i.code == "missing_church_program")
    );
}

#[test]
fn principal_tower_rejects_missing_bearing_and_landing() {
    let plan = generate(&principal(42)).unwrap();
    let tower = &plan.church.as_ref().unwrap().tower;
    let mut no_bearing = plan.clone();
    no_bearing
        .resolved_geometry
        .structural_nodes
        .retain(|node| node.id != tower.stair_bearing_node);
    assert!(
        audit_plan(&no_bearing)
            .iter()
            .any(|i| i.code == "invalid_church_circulation")
    );
    let mut no_landing = plan.clone();
    no_landing
        .resolved_geometry
        .solids
        .retain(|solid| solid.id != tower.landing_solids[0]);
    assert!(
        audit_plan(&no_landing)
            .iter()
            .any(|i| i.code == "invalid_church_circulation")
    );
    let mut shifted_portal = plan.clone();
    let portal_void = shifted_portal
        .opening_assemblies
        .iter()
        .find(|o| o.id == tower.west_portal)
        .unwrap()
        .void_id;
    let aperture = shifted_portal
        .resolved_geometry
        .voids
        .iter_mut()
        .find(|v| v.id == portal_void)
        .unwrap();
    aperture.bounds.min.z += 1.0;
    aperture.bounds.max.z += 1.0;
    assert!(
        audit_plan(&shifted_portal)
            .iter()
            .any(|i| i.code == "invalid_church_circulation")
    );
}
