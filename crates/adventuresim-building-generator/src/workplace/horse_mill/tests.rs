use super::*;
use crate::{BuildingPlan, audit_plan, generate, settlement_archetype};

fn mill(size: ServiceBuildingSize) -> BuildingPlan {
    let usage = BuildingUse::HorseMill;
    generate(
        &BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
            .with_service_size(size),
    )
    .unwrap()
}

#[test]
fn capacity_adds_grain_bays_without_expanding_the_drive_or_animal_track() {
    let small = mill(ServiceBuildingSize::Small);
    let large = mill(ServiceBuildingSize::Large);
    for feature in [
        WorkplaceFeature::MillDrive,
        WorkplaceFeature::MillSweep,
        WorkplaceFeature::Millstone,
    ] {
        let geometry = |plan: &BuildingPlan| {
            plan.workplace
                .as_ref()
                .unwrap()
                .parts
                .iter()
                .filter(|part| part.feature == feature)
                .map(|part| {
                    let solid = plan
                        .resolved_geometry
                        .solids
                        .iter()
                        .find(|s| s.id == part.solid)
                        .unwrap();
                    (
                        solid.centre,
                        solid.size,
                        solid.yaw_radians,
                        solid.crossfall_radians,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert!(!geometry(&small).is_empty());
        assert_eq!(geometry(&small), geometry(&large));
    }
    let grain_parts = |plan: &BuildingPlan| {
        plan.workplace
            .as_ref()
            .unwrap()
            .parts
            .iter()
            .filter(|part| part.feature == WorkplaceFeature::StorageBin)
            .count()
    };
    assert!(grain_parts(&large) > grain_parts(&small));
}

#[test]
fn full_rotation_rejects_obstacles_away_from_the_parked_sweep_including_its_high_brace() {
    for centre in [Vec3::new(8.0, 3.875, 6.0), Vec3::new(10.4, 1.9, 6.0)] {
        let mut plan = mill(ServiceBuildingSize::Small);
        let id = plan
            .workplace
            .as_ref()
            .unwrap()
            .parts
            .iter()
            .find(|part| part.feature == WorkplaceFeature::StorageBin)
            .unwrap()
            .solid;
        let obstruction = plan
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == id)
            .unwrap();
        obstruction.centre = centre;
        obstruction.size = Vec3::splat(0.02);
        assert!(
            audit_plan(&plan)
                .iter()
                .any(|issue| issue.code == "horse_mill_sweep_obstruction"),
            "full turning envelope missed obstacle at {centre:?}"
        );
    }
}

#[test]
fn animal_circuit_and_low_draw_link_are_enforced_independently_of_worker_passages() {
    let mut plan = mill(ServiceBuildingSize::Small);
    let work = plan.workplace.as_ref().unwrap();
    let stock = work
        .parts
        .iter()
        .find(|part| part.feature == WorkplaceFeature::StorageBin)
        .unwrap()
        .solid;
    let link = work
        .parts
        .iter()
        .filter(|part| part.feature == WorkplaceFeature::MillSweep)
        .find(|part| part.material == WorkplaceMaterial::HempRope)
        .unwrap()
        .solid;
    let obstruction = plan
        .resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == stock)
        .unwrap();
    obstruction.centre = Vec3::new(6.5, 0.9, 10.0);
    obstruction.size = Vec3::splat(0.3);
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "horse_mill_track_obstruction")
    );
    plan.resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == link)
        .unwrap()
        .centre
        .z = 3.5;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "horse_mill_draw_link_outside_track")
    );
}
