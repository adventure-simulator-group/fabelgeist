use crate::*;

#[test]
fn merchant_upper_frame_bears_on_actual_masonry_lower_walls() {
    for seed in [42, 47, 101] {
        let plan = generate(&BuildingProgram::fixture(
            BuildingArchetype::FachwerkMerchantHouse,
            seed,
        ))
        .unwrap();
        for wall in plan.wall_assemblies.iter().filter(|wall| {
            wall.frame.outside_room.is_none()
                && matches!(wall.source, WallSourceId::StoreyWall { .. })
        }) {
            if wall.storey_level == 0 {
                assert_eq!(wall.material, WallMaterialClass::CivilianMasonry);
                assert_eq!(wall.structural_role, WallStructuralRole::LoadBearing);
            } else {
                assert_eq!(wall.material, WallMaterialClass::TimberInfill);
            }
        }
        let frame = plan.timber_frame.as_ref().unwrap();
        assert!(frame.masonry_bearing_interfaces.len() >= 4);
        for id in &frame.masonry_bearing_interfaces {
            let interface = plan
                .resolved_geometry
                .support_interfaces
                .iter()
                .find(|item| item.id == *id)
                .unwrap();
            let node = plan
                .resolved_geometry
                .structural_nodes
                .iter()
                .find(|item| item.id == interface.node)
                .unwrap();
            assert!(
                node.supported_by
                    .iter()
                    .any(|support| plan
                        .wall_assemblies
                        .iter()
                        .any(|wall| wall.storey_level == 0
                            && wall.material == WallMaterialClass::CivilianMasonry
                            && wall.support_node == *support))
            );
        }
    }
    let cottage = generate(&BuildingProgram::fixture(
        BuildingArchetype::FachwerkCottage,
        42,
    ))
    .unwrap();
    assert!(
        cottage
            .wall_assemblies
            .iter()
            .filter(|wall| wall.frame.outside_room.is_none()
                && matches!(wall.source, WallSourceId::StoreyWall { .. }))
            .all(|wall| wall.material == WallMaterialClass::TimberInfill)
    );
}
