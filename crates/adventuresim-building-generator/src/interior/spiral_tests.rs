//! Fortified occupied floors use physical shared spiral landing portals.
use super::architecture::Floor;
use super::geometry::Rect;
use super::*;
use crate::{BuildingProgram, ServiceBuildingSize};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

#[test]
fn fortified_spiral_landings_connect_all_occupied_rooms() {
    for usage in [BuildingUse::Castle, BuildingUse::Arsenal] {
        let program = BuildingProgram::validated_settlement(
            crate::settlement_archetype(usage),
            usage,
            42,
            Some(ServiceBuildingSize::Medium),
        )
        .unwrap();
        let plan = crate::generate(&program).unwrap();
        for index in 0..plan.stairs.len() {
            for landing in crate::spiral_stairs::landings(&plan, index) {
                let floor = Floor::new(&plan, landing.storey);
                let rect = Rect::new(landing.position_metres, Vec2::splat(0.3));
                assert!(
                    floor.supports(rect, landing.elevation_metres),
                    "{usage:?}: unsupported {landing:?}"
                );
                assert!(floor.walkable(rect), "{usage:?}: blocked {landing:?}");
            }
        }
        super::navigation::Navigation::new(&plan)
            .unwrap_or_else(|error| panic!("{usage:?}: {error:?}"));
        let layout =
            furnish(&plan, &program).unwrap_or_else(|error| panic!("{usage:?}: {error:?}"));
        let fresh_paths = validate_layout(&plan, &layout).unwrap();
        assert!(!layout.placements.is_empty());
        assert_eq!(fresh_paths, layout.paths);
        assert!(
            fresh_paths
                .iter()
                .all(|path| path.points.first().unwrap().storey == 0)
        );
    }
}
