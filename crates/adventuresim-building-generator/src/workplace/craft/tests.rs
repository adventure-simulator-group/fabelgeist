use super::*;
use crate::{BuildingArchetype, audit_plan, generate, settlement_archetype};

#[test]
fn timber_trades_keep_visible_supported_stock_outside_the_handling_lane() {
    for usage in [BuildingUse::TimberYard, BuildingUse::Carpenter] {
        for size in [
            WorkplaceSize::Small,
            WorkplaceSize::Medium,
            WorkplaceSize::Large,
        ] {
            for seed in [0, 42, 101] {
                let program =
                    BuildingProgram::settlement(settlement_archetype(usage), Some(usage), seed)
                        .with_workplace_size(size);
                let plan = generate(&program)
                    .unwrap_or_else(|error| panic!("{usage:?} {size:?} {seed}: {error:?}"));
                let work = plan.workplace.as_ref().unwrap();
                assert!(work.passages[0].max.x - work.passages[0].min.x >= 3.0);
                assert!(
                    work.parts
                        .iter()
                        .any(|part| part.feature == WorkplaceFeature::TimberStack)
                );
                assert!(
                    work.parts
                        .iter()
                        .any(|part| part.feature == WorkplaceFeature::SawBench)
                );
                let full_height_walls = work
                    .walls
                    .iter()
                    .filter(|id| {
                        plan.wall_assemblies.iter().any(|wall| {
                            wall.id == **id
                                && wall.base_elevation_metres == 0.0
                                && wall.height_metres >= program.storey_height_metres
                        })
                    })
                    .count();
                if usage == BuildingUse::TimberYard {
                    assert_eq!(full_height_walls, 0, "drying shelter must remain open");
                } else {
                    assert!(
                        full_height_walls >= 3,
                        "joinery requires an enclosed rear shop"
                    );
                }
                let mut blocked = plan.clone();
                let stock = work
                    .parts
                    .iter()
                    .find(|part| part.feature == WorkplaceFeature::TimberStack)
                    .unwrap();
                let passage = &work.passages[0];
                blocked
                    .resolved_geometry
                    .solids
                    .iter_mut()
                    .find(|solid| solid.id == stock.solid)
                    .unwrap()
                    .centre = (passage.min + passage.max) * 0.5;
                assert!(
                    audit_plan(&blocked)
                        .iter()
                        .any(|issue| issue.code == "workplace_blocked_passage")
                );
            }
        }
    }
}

#[test]
fn armorers_reserve_the_shared_forge_without_losing_their_shop_identity() {
    let usage = BuildingUse::Armorer;
    assert_eq!(settlement_archetype(usage), BuildingArchetype::Workplace);
    let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42);
    assert_eq!(program.usage, Some(usage));
    let plan = generate(&program).unwrap();
    let work = plan.workplace.unwrap();
    assert_eq!(work.kind, WorkplaceKind::Smithy);
    assert!(
        work.parts
            .iter()
            .any(|part| part.feature == WorkplaceFeature::Forge)
    );
    assert!(
        work.plot_dimensions_metres.x
            > f32::from(program.footprint.dimensions().0) * crate::CELL_SIZE_METRES
    );
}
