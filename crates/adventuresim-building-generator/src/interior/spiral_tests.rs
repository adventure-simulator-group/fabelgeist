//! Fortified occupied floors use physical shared spiral landing portals.
use super::architecture::Floor;
use super::geometry::Rect;
use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::PlanExtents;
use crate::{BuildingProgram, ServiceBuildingSize};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

#[test]
fn fortified_spiral_landings_connect_all_occupied_rooms() {
    for usage in [BuildingUse::Castle, BuildingUse::Arsenal] {
        let program = BuildingProgram::validated_settlement(
            crate::settlement_archetype(usage),
            usage,
            fabelgeist_determinism::Seed::from_u64(42),
            Some(ServiceBuildingSize::Medium),
        )
        .unwrap();
        let plan = crate::generate(&program).unwrap();
        for index in 0..plan.stairs.len() {
            for landing in crate::spiral_stairs::landings(&plan, index) {
                let floor =
                    Floor::new(&plan, crate::StoreyIndex::from_serialized(landing.storey)).unwrap();
                let rect = Rect::new(
                    ArchitecturalPlanPoint::from_metres(landing.position_metres).unwrap(),
                    PlanExtents::from_metres(Vec2::splat(0.3)).unwrap(),
                )
                .unwrap();
                assert!(
                    floor
                        .supports(
                            rect,
                            crate::spatial_geometry::Elevation::from_metres(
                                landing.elevation_metres
                            )
                            .unwrap()
                        )
                        .unwrap(),
                    "{usage:?}: unsupported {landing:?}"
                );
                assert!(
                    floor.walkable(rect).unwrap(),
                    "{usage:?}: blocked {landing:?}"
                );
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
                .all(|path| path.points.first().unwrap().storey == crate::StoreyIndex::GROUND)
        );
    }
}
