use crate::*;
use bevy::math::Vec2;

#[test]
fn storage_range_has_one_usable_nonresidential_level_and_normal_door() {
    for seed in [42, 47, 101] {
        let program = BuildingProgram::fixture(BuildingArchetype::StorageRange, seed);
        let plan = generate(&program).unwrap();
        assert_eq!(program.usage, None);
        assert_eq!(program.plot_dimensions_metres(), Vec2::new(9.0, 6.0));
        assert_eq!(plan.storeys.len(), 1);
        assert!(program.vertical_connections.is_empty());
        assert!(
            program
                .storeys
                .iter()
                .flat_map(|storey| &storey.rooms)
                .all(|room| room.kind == RoomKind::Storage)
        );
        let doors = compile_operable_doors(&plan);
        assert_eq!(doors.len(), 1);
        assert_eq!(doors[0].outward, -Vec2::Y);
        assert!(doors[0].size_metres.x >= 0.9);
        let collision = compile_building_collision(&plan);
        assert!(!collision.cuboids.is_empty());
        let furnished = interior::furnish(&plan, &program)
            .expect("rear storage floor is accessible and furnished");
        assert!(!furnished.placements.is_empty());
        for meshes in [
            compile_building_detail(&plan).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Facade).meshes,
            compile_building_lod(&plan, BuildingLodLevel::Shell).meshes,
        ] {
            assert!(!meshes.is_empty());
            assert!(
                meshes
                    .iter()
                    .flat_map(|mesh| &mesh.vertices)
                    .all(|vertex| vertex.position.is_finite())
            );
        }
    }
}

#[test]
fn merchant_ground_floor_has_an_operable_court_exit() {
    for seed in [42, 47, 101] {
        let plan = generate(&BuildingProgram::fixture(
            BuildingArchetype::FachwerkMerchantHouse,
            seed,
        ))
        .unwrap();
        let doors = compile_operable_doors(&plan);
        assert!(doors.iter().any(|door| door.outward == -Vec2::Y));
        assert!(doors.iter().any(|door| door.outward == Vec2::Y));
        // A successful structural audit also verifies the connected room graph;
        // these are compiled operable leaves, rather than intended route points.
        assert!(
            doors
                .iter()
                .all(|door| door.closed_centre.y < plan.storey_height_metres)
        );
    }
}
