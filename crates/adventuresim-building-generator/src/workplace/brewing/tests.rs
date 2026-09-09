use super::*;
use crate::{audit_plan, compile_building_collision, generate, settlement_archetype};

#[test]
fn brewing_vessels_have_usable_open_volume_and_solid_timber_bottoms() {
    let usage = BuildingUse::Brewery;
    let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
        .with_workplace_size(WorkplaceSize::Small);
    let plan = generate(&program).unwrap();
    let workplace = plan.workplace.as_ref().unwrap();
    let centre = Vec3::new(2.0, 0.7, 2.24);
    let collision = compile_building_collision(&plan);
    let contains = |point: Vec3| {
        collision
            .cuboids
            .iter()
            .filter(|cuboid| {
                workplace.parts.iter().any(|part| {
                    part.solid == cuboid.source && part.feature == WorkplaceFeature::Vat
                })
            })
            .any(|cuboid| {
                let solid = plan
                    .resolved_geometry
                    .solids
                    .iter()
                    .find(|solid| solid.id == cuboid.source)
                    .unwrap();
                let local = super::super::assembly::contact::rotation(solid).inverse()
                    * (point - solid.centre);
                (solid.size * 0.5 - local.abs()).min_element() > 0.0
            })
    };
    assert!(
        !contains(centre),
        "a solid prop must not fill the vessel's working volume"
    );
    assert!(
        contains(Vec3::new(centre.x, 0.08, centre.z)),
        "vessels need physical bottoms"
    );
    assert!(
        workplace
            .parts
            .iter()
            .any(|part| part.feature == WorkplaceFeature::Vat
                && part.material == WorkplaceMaterial::Iron)
    );
}

#[test]
fn detached_vessel_hoops_are_rejected_by_the_architecture_audit() {
    let usage = BuildingUse::Brewery;
    let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
        .with_workplace_size(WorkplaceSize::Small);
    let mut plan = generate(&program).unwrap();
    let hoop = plan
        .workplace
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .find(|part| {
            part.feature == WorkplaceFeature::Vat && part.material == WorkplaceMaterial::Iron
        })
        .unwrap()
        .solid;
    plan.resolved_geometry
        .solids
        .iter_mut()
        .find(|solid| solid.id == hoop)
        .unwrap()
        .centre
        .y += 20.0;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "workplace_floating_part")
    );
}

#[test]
fn malt_kiln_rejects_obstruction_of_its_continuous_exhaust_channel() {
    let usage = BuildingUse::Malthouse;
    let program = BuildingProgram::settlement(settlement_archetype(usage), Some(usage), 42)
        .with_workplace_size(WorkplaceSize::Medium);
    let mut plan = generate(&program).unwrap();
    let work = plan.workplace.as_ref().unwrap();
    let vent = work
        .passages
        .iter()
        .find(|passage| passage.min.y > 0.5 && passage.max.y > 3.0)
        .unwrap();
    let centre = (vent.min + vent.max) * 0.5;
    let collision = compile_building_collision(&plan);
    for cuboid in &collision.cuboids {
        let solid = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|solid| solid.id == cuboid.source)
            .unwrap();
        let local =
            super::super::assembly::contact::rotation(solid).inverse() * (centre - solid.centre);
        assert!(
            (solid.size * 0.5 - local.abs()).min_element() <= 0.0,
            "the kiln must not hide a solid cap under its visible outlet"
        );
    }
    let blocked = work
        .parts
        .iter()
        .find(|part| part.feature == WorkplaceFeature::Kiln)
        .unwrap()
        .solid;
    plan.resolved_geometry
        .solids
        .iter_mut()
        .find(|solid| solid.id == blocked)
        .unwrap()
        .centre = centre;
    assert!(
        audit_plan(&plan)
            .iter()
            .any(|issue| issue.code == "workplace_blocked_passage")
    );
}
