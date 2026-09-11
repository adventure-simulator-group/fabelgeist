use super::*;

#[test]
fn modest_church_matrix_has_shared_openings_routes_and_capacity_geometry() {
    for usage in [BuildingUse::Chapel, BuildingUse::ParishChurch] {
        for size in [
            ServiceBuildingSize::Small,
            ServiceBuildingSize::Medium,
            ServiceBuildingSize::Large,
        ] {
            for seed in [0, 42, 101] {
                let program =
                    BuildingProgram::settlement(BuildingArchetype::ParishChurch, Some(usage), seed)
                        .with_service_size(size);
                let plan = crate::generate(&program)
                    .unwrap_or_else(|error| panic!("{usage:?}/{size:?}/{seed}: {error:?}"));
                let church = plan.small_church.as_ref().unwrap();
                assert!(plan.church.is_none());
                assert_eq!(
                    church.chancel_eave_metres.is_some(),
                    usage == BuildingUse::ParishChurch
                );
                let detail = crate::compile_building_detail(&plan);
                let collision = crate::compile_building_collision(&plan);
                for opening in plan
                    .opening_assemblies
                    .iter()
                    .filter(|opening| opening.use_kind == crate::OpeningUse::Window)
                {
                    assert!(matches!(
                        opening.profile,
                        crate::OpeningProfile::PointedTwoCentred { .. }
                    ));
                    for id in opening.jamb_solids {
                        let solid = plan
                            .resolved_geometry
                            .solids
                            .iter()
                            .find(|solid| solid.id == id)
                            .unwrap();
                        assert!(collision.cuboids.iter().any(|cuboid| cuboid.source == id));
                        let corner = solid.centre + solid.size * 0.5;
                        assert!(
                            detail
                                .meshes
                                .iter()
                                .flat_map(|mesh| &mesh.vertices)
                                .any(|vertex| vertex.position.distance(corner) < 0.001),
                            "real masonry jamb missing from detail: {id:?}"
                        );
                    }
                }
                for level in [
                    crate::BuildingLodLevel::Facade,
                    crate::BuildingLodLevel::Shell,
                ] {
                    let lod = crate::compile_building_lod(&plan, level);
                    assert!(
                        lod.meshes
                            .iter()
                            .map(|mesh| mesh.indices.len())
                            .sum::<usize>()
                            < detail
                                .meshes
                                .iter()
                                .map(|mesh| mesh.indices.len())
                                .sum::<usize>()
                    );
                }
            }
        }
    }
}

#[test]
fn sacred_aisle_rejects_an_added_obstruction() {
    let mut plan = crate::generate(&BuildingProgram::fixture(
        BuildingArchetype::ParishChurch,
        42,
    ))
    .unwrap();
    let aisle = plan.small_church.as_ref().unwrap().public_route;
    let mut obstruction = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|solid| solid.role == SolidRole::ChurchBell)
        .unwrap()
        .clone();
    obstruction.centre = (aisle.min + aisle.max) * 0.5;
    obstruction.size = Vec3::splat(0.3);
    plan.resolved_geometry.solids.push(obstruction);
    assert!(
        crate::audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "small_church_blocked_aisle")
    );
}

#[test]
fn turret_cut_preserves_bearing_rim_but_removes_the_spanning_ridge_cap() {
    let plan = crate::generate(&BuildingProgram::fixture(
        BuildingArchetype::ParishChurch,
        42,
    ))
    .unwrap();
    let roof = &plan.roof_assemblies[0];
    let child = roof
        .children
        .first()
        .expect("turret has an authoritative roof attachment");
    let cut = plan
        .resolved_geometry
        .voids
        .iter()
        .find(|void| void.id == child.parent_cut)
        .unwrap();
    assert!(!child.trimmer_nodes.is_empty());
    assert!(!child.flashing_ids.is_empty());
    let ridge = roof
        .edges
        .iter()
        .filter(|edge| edge.kind == RoofEdgeKind::Ridge)
        .collect::<Vec<_>>();
    assert_eq!(
        ridge.len(),
        2,
        "parent ridge must be split on both sides of the turret"
    );
    for edge in ridge {
        assert!(
            edge.start.z.max(edge.end.z) <= cut.bounds.min.z + 0.001
                || edge.start.z.min(edge.end.z) >= cut.bounds.max.z - 0.001
        );
    }
    let collision = crate::compile_building_collision(&plan);
    for cuboid in collision.cuboids.iter().filter(|cuboid| {
        plan.resolved_geometry.solids.iter().any(|solid| {
            solid.id == cuboid.source
                && solid.owner == roof.owner
                && solid.role == SolidRole::RoofEdgeTreatment
        })
    }) {
        let orientation = bevy::math::Quat::from_rotation_y(cuboid.yaw_radians)
            * bevy::math::Quat::from_rotation_x(cuboid.crossfall_radians)
            * bevy::math::Quat::from_rotation_z(cuboid.longfall_radians);
        let half = cuboid.size * 0.5;
        let extent = (orientation * Vec3::X).abs() * half.x
            + (orientation * Vec3::Y).abs() * half.y
            + (orientation * Vec3::Z).abs() * half.z;
        let overlap = (cuboid.centre + extent).min(cut.bounds.max)
            - (cuboid.centre - extent).max(cut.bounds.min);
        assert!(
            overlap.min_element() <= 0.025,
            "solid ridge cap still blocks the roof opening"
        );
    }
}
