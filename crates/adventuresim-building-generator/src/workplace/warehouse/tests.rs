use super::*;
use crate::{
    BuildingArchetype, audit_plan, compile_building_collision, generate, settlement_archetype,
};

#[test]
fn warehouses_provide_loading_routes_to_two_real_storage_floors() {
    for size in [
        ServiceBuildingSize::Small,
        ServiceBuildingSize::Medium,
        ServiceBuildingSize::Large,
    ] {
        for seed in [0, 42, 101] {
            let usage = BuildingUse::Warehouse;
            let program =
                BuildingProgram::settlement(settlement_archetype(usage), Some(usage), seed)
                    .with_service_size(size);
            assert_eq!(program.archetype, BuildingArchetype::Workplace);
            let plan =
                generate(&program).unwrap_or_else(|error| panic!("{size:?} {seed}: {error:?}"));
            let (width, depth) = program.footprint.dimensions();
            let w = f32::from(width) * crate::CELL_SIZE_METRES;
            let d = f32::from(depth) * crate::CELL_SIZE_METRES;
            let collision = compile_building_collision(&plan);
            let occupied = |point: Vec3| {
                collision.cuboids.iter().any(|cuboid| {
                    let local = bevy::math::Quat::from_rotation_y(-cuboid.yaw_radians)
                        * (point - cuboid.centre);
                    (cuboid.size * 0.5 - local.abs()).min_element() > 0.0
                })
            };
            for z in [d * 0.25, d * 0.5] {
                for x in [w * 0.5, w, w + 2.8, w + 6.4] {
                    assert!(
                        !occupied(Vec3::new(x, 1.3, z)),
                        "loading route blocked at {x},{z}"
                    );
                }
            }
            assert!(
                occupied(Vec3::new(w * 0.5, 3.28, d * 0.5)),
                "upper vents require a usable floor"
            );
            assert!(
                !occupied(Vec3::new(w * 0.5, 4.3, d * 0.5)),
                "upper handling aisle blocked"
            );
            for step in 0..18 {
                let rise = 3.36 * (step + 1) as f32 / 18.0;
                let z = d - 7.1 + (step as f32 + 0.5) * (4.8 / 18.0);
                assert!(
                    occupied(Vec3::new(w - 1.0, rise - 0.02, z)),
                    "stair has a missing tread"
                );
                assert!(
                    !occupied(Vec3::new(w - 1.0, rise + 1.0, z)),
                    "stair headroom blocked"
                );
            }
            assert!(
                occupied(Vec3::new(w - 1.0, 3.28, d - 1.3)),
                "stair must reach an upper landing"
            );
            assert!(program.plot_dimensions_metres().x >= w + 7.8);
            assert!(
                plan.roofs.len() > 1,
                "the external loading station needs its hood"
            );
        }
    }
}

#[test]
fn warehouse_audit_rejects_a_detached_loading_hook() {
    let program = BuildingProgram::settlement(
        settlement_archetype(BuildingUse::Warehouse),
        Some(BuildingUse::Warehouse),
        42,
    );
    let mut plan = generate(&program).unwrap();
    let hook = plan
        .workplace
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .find(|part| {
            part.feature == WorkplaceFeature::LoadingHoist
                && part.material == WorkplaceMaterial::Iron
        })
        .unwrap()
        .solid;
    plan.resolved_geometry
        .solids
        .iter_mut()
        .find(|solid| solid.id == hook)
        .unwrap()
        .centre
        .y += 20.0;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "workplace_floating_part")
    );
}
