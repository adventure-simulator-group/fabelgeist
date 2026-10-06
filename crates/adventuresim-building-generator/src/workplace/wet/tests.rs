use super::*;
use crate::spatial_geometry::Position;
use crate::{audit_plan, compile_building_collision, generate, settlement_archetype};

#[test]
fn wet_vessels_have_physical_basins_without_solid_liquid_barriers() {
    for kind in [WorkplaceKind::Dyer, WorkplaceKind::Tannery] {
        let usage = kind.usage();
        let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
            .with_service_size(ServiceBuildingSize::Small);
        let plan = generate(&program).unwrap();
        let work = plan.workplace.as_ref().unwrap();
        let collision = compile_building_collision(&plan).unwrap();
        let liquids = work
            .parts
            .iter()
            .filter(|part| part.feature == WorkplaceFeature::ProcessLiquid)
            .collect::<Vec<_>>();
        assert!(!liquids.is_empty());
        for liquid in liquids {
            assert!(
                !collision
                    .cuboids
                    .iter()
                    .any(|cuboid| cuboid.source == liquid.solid),
                "rendered process liquid must not become a solid platform"
            );
            let fill = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == liquid.solid)
                .unwrap();
            let bottom = fill.centre.metres().y - fill.size.metres().y * 0.5;
            assert!(
                collision.cuboids.iter().any(|cuboid| {
                    let point = Vec3::new(
                        fill.centre.metres().x,
                        bottom - 0.04,
                        fill.centre.metres().z,
                    );
                    (cuboid.size.metres() * 0.5 - (point - cuboid.centre.metres()).abs())
                        .min_element()
                        > 0.0
                }),
                "liquid must be contained above an actual basin bottom"
            );
        }
    }
}

#[test]
fn detached_textiles_and_blocked_wet_trade_lanes_fail_the_audit() {
    for kind in [WorkplaceKind::Dyer, WorkplaceKind::Tannery] {
        let usage = kind.usage();
        let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
            .with_service_size(ServiceBuildingSize::Medium);
        let mut plan = generate(&program).unwrap();
        let work = plan.workplace.as_ref().unwrap();
        let textile = work
            .parts
            .iter()
            .find(|part| {
                matches!(
                    part.feature,
                    WorkplaceFeature::Cloth | WorkplaceFeature::Hide
                )
            })
            .unwrap()
            .solid;
        {
            let mut native_geometry = plan
                .resolved_geometry
                .solids
                .iter_mut()
                .find(|solid| solid.id == textile)
                .unwrap()
                .centre
                .metres();
            native_geometry.y += 20.0;
            plan.resolved_geometry
                .solids
                .iter_mut()
                .find(|solid| solid.id == textile)
                .unwrap()
                .centre = Position::from_metres(native_geometry).unwrap();
        };
        assert!(
            audit_plan(&plan)
                .unwrap()
                .iter()
                .any(|issue| issue.code == "workplace_floating_part")
        );
        let work = plan.workplace.as_ref().unwrap();
        let lane = &work.passages[1];
        let centre = (lane.bounds.min().metres() + lane.bounds.max().metres()) * 0.5;
        let frame = work
            .parts
            .iter()
            .find(|part| part.feature == WorkplaceFeature::DryingFrame)
            .unwrap()
            .solid;
        plan.resolved_geometry
            .solids
            .iter_mut()
            .find(|solid| solid.id == frame)
            .unwrap()
            .centre = Position::<crate::Architectural>::from_metres(centre).unwrap();
        assert!(
            audit_plan(&plan)
                .unwrap()
                .iter()
                .any(|issue| issue.code == "workplace_blocked_passage")
        );
    }
}

#[test]
fn sloping_fleshing_beams_update_their_actual_bearing_positions() {
    let usage = BuildingUse::Tannery;
    let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
        .with_service_size(ServiceBuildingSize::Small);
    let plan = generate(&program).unwrap();
    let work = plan.workplace.as_ref().unwrap();
    let mut sloped = 0;
    for part in work
        .parts
        .iter()
        .filter(|part| part.feature == WorkplaceFeature::FleshingBeam)
    {
        let solid = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.id == part.solid)
            .unwrap();
        if solid.crossfall_radians.radians().abs() < 0.1 {
            continue;
        }
        sloped += 1;
        let bounds = super::super::assembly::contact::bounds(solid).unwrap();
        let node = plan
            .resolved_geometry
            .structural_nodes
            .iter()
            .find(|node| node.id == solid.supported_by[0])
            .unwrap();
        assert!((node.position.metres().y - bounds.min().metres().y).abs() < 0.001);
        assert!(!node.grounded && !node.supported_by.is_empty());
    }
    assert_eq!(sloped, 2);
}
