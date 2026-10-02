use super::*;
use bevy::math::Vec3;

fn merchant() -> (BuildingProgram, BuildingPlan) {
    let mut program = BuildingProgram::fixture(BuildingArchetype::FachwerkMerchantHouse, 0);
    program.domestic_heating = Some(DomesticHeatingProgramme::HearthAndRearFedStove);
    let plan = generate(&program).unwrap();
    (program, plan)
}

fn part(plan: &BuildingPlan, kind: HeatingPartKind) -> ResolvedItemId {
    plan.domestic_heating
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .find(|p| p.kind == kind)
        .unwrap()
        .solid
}

fn fails(plan: &BuildingPlan, code: &str) {
    let issues = audit(plan);
    assert!(
        issues.iter().any(|i| i.code == code),
        "expected {code}: {issues:?}"
    );
}

#[test]
fn upper_appliance_preserves_roof_structure_and_furnished_access_on_every_floor() {
    let (program, plan) = merchant();
    let heating = plan.domestic_heating.as_ref().unwrap();
    assert_eq!(heating.kitchen.storey_level, 1);
    assert_eq!(heating.floor_height_metres, plan.storey_height_metres);
    assert_eq!(heating.floors.len(), 2);
    assert!(audit_plan(&plan).is_empty());
    crate::interior::validate_circulation(&plan).unwrap();
    let layout = crate::interior::furnish(&plan, &program).unwrap();
    for storey in &plan.storeys {
        assert!(layout.placements.iter().any(|p| p.storey == storey.level));
    }
    let mut unheated = program;
    unheated.domestic_heating = None;
    let baseline = generate(&unheated).unwrap();
    for member in plan
        .timber_frame
        .as_ref()
        .unwrap()
        .members
        .iter()
        .filter(|m| {
            matches!(
                m.role,
                TimberMemberRole::Rafter
                    | TimberMemberRole::Collar
                    | TimberMemberRole::Purlin
                    | TimberMemberRole::DormerTrimmer
            )
        })
    {
        let actual = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == member.solid)
            .unwrap();
        let before = baseline
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == member.solid)
            .unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
    let collision = compile_building_collision(&plan);
    for part in &heating.parts {
        assert!(collision.cuboids.iter().any(|s| s.source == part.solid));
    }
}

#[test]
fn upper_weight_requires_continuous_downward_ground_bearing() {
    let (_, original) = merchant();
    let pier = part(&original, HeatingPartKind::SupportPier);
    let mut missing = original.clone();
    missing.resolved_geometry.solids.retain(|s| s.id != pier);
    fails(&missing, "unsupported_upper_heating");
    let mut shifted = original.clone();
    shifted
        .resolved_geometry
        .solids
        .iter_mut()
        .find(|s| s.id == pier)
        .unwrap()
        .centre
        .x += 0.2;
    fails(&shifted, "unsupported_upper_heating");
    let mut false_foundation = original;
    let footing = part(&false_foundation, HeatingPartKind::Footing);
    let node = false_foundation
        .resolved_geometry
        .solids
        .iter()
        .find(|s| s.id == footing)
        .unwrap()
        .supported_by[0];
    false_foundation
        .resolved_geometry
        .structural_nodes
        .iter_mut()
        .find(|n| n.id == node)
        .unwrap()
        .grounded = true;
    fails(&false_foundation, "unsupported_upper_heating");
}

#[test]
fn decks_and_joists_cannot_cross_either_occupied_floor_penetration() {
    let (_, original) = merchant();
    for opening in &original.domestic_heating.as_ref().unwrap().floors {
        let mut blocked = original.clone();
        let mut plug = blocked
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.role == SolidRole::FrameFloor)
            .unwrap()
            .clone();
        plug.id = ResolvedItemId(u64::MAX);
        plug.centre = (opening.cut.min + opening.cut.max) * 0.5;
        plug.size = opening.cut.max - opening.cut.min;
        blocked.resolved_geometry.solids.push(plug);
        fails(&blocked, "blocked_heating_floor_penetration");
        let mut joist = original.clone();
        let member = joist
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.role == SolidRole::FrameJoist)
            .unwrap();
        member.centre = (opening.cut.min + opening.cut.max) * 0.5;
        fails(&joist, "blocked_heating_floor_penetration");
    }
}

#[test]
fn floor_clearance_needs_complete_covers_and_bearing_laps() {
    let (_, original) = merchant();
    for opening in &original.domestic_heating.as_ref().unwrap().floors {
        let mut missing = original.clone();
        missing
            .resolved_geometry
            .solids
            .retain(|s| s.id != opening.closures[0]);
        fails(&missing, "open_heating_floor_clearance");
        let mut excess = original.clone();
        excess
            .resolved_geometry
            .solids
            .iter_mut()
            .find(|s| s.id == opening.closures[0])
            .unwrap()
            .size
            .x += 0.2;
        fails(&excess, "open_heating_floor_clearance");
        let mut no_deck = original.clone();
        no_deck.resolved_geometry.solids.retain(|s| {
            s.role != SolidRole::FrameFloor
                || (s.cuboid_bounds().max.y - opening.core.max.y).abs() > 0.001
        });
        fails(&no_deck, "open_heating_floor_clearance");
    }
    let mut no_inner_bearing = original;
    let shoulder = part(&no_inner_bearing, HeatingPartKind::FlueShoulder);
    no_inner_bearing
        .resolved_geometry
        .solids
        .retain(|s| s.id != shoulder);
    fails(&no_inner_bearing, "open_heating_floor_clearance");
}

#[test]
fn floor_bearing_cannot_refer_to_an_unrelated_ground_node() {
    let (_, mut plan) = merchant();
    let floor = &plan.timber_frame.as_ref().unwrap().floors[1];
    let id = *floor.floor_joist_interfaces.last().unwrap();
    let ground = plan.domestic_heating.as_ref().unwrap().ground_support;
    plan.resolved_geometry
        .support_interfaces
        .iter_mut()
        .find(|i| i.id == id)
        .unwrap()
        .node = ground;
    fails(&plan, "detached_heating_floor_bearing");
}

#[test]
fn occupied_upper_flue_bore_and_lower_entrance_remain_clear() {
    let (_, original) = merchant();
    let h = original.domestic_heating.as_ref().unwrap();
    let bore = h
        .passages
        .iter()
        .find(|p| p.kind == HeatingPassageKind::FlueBore)
        .unwrap()
        .void;
    let bounds = original
        .resolved_geometry
        .voids
        .iter()
        .find(|v| v.id == bore)
        .unwrap()
        .bounds;
    let mut plugged = original.clone();
    let mut plug = original.resolved_geometry.solids[0].clone();
    plug.id = ResolvedItemId(u64::MAX);
    plug.shape = ResolvedSolidShape::Cuboid;
    plug.yaw_radians = 0.0;
    plug.crossfall_radians = 0.0;
    plug.longfall_radians = 0.0;
    plug.centre = Vec3::new(
        (bounds.min.x + bounds.max.x) * 0.5,
        6.5,
        (bounds.min.z + bounds.max.z) * 0.5,
    );
    plug.size = Vec3::splat(0.1);
    plugged.resolved_geometry.solids.push(plug.clone());
    fails(&plugged, "blocked_domestic_smoke_route");
    let mut blocked = original.clone();
    let entry = blocked
        .timber_frame
        .as_ref()
        .unwrap()
        .circulation
        .entry_opening
        .unwrap();
    let door = blocked
        .opening_assemblies
        .iter()
        .find(|o| o.id == entry)
        .unwrap();
    plug.centre = Vec3::new(door.frame.origin.x, 1.0, door.frame.origin.y);
    plug.size = Vec3::new(2.0, 2.0, 2.0);
    blocked
        .domestic_heating
        .as_mut()
        .unwrap()
        .parts
        .push(HeatingPart {
            solid: plug.id,
            kind: HeatingPartKind::SupportPier,
            material: BuildingLodMaterial::Wall(WallMaterialClass::CivilianMasonry),
        });
    blocked.resolved_geometry.solids.push(plug);
    assert!(crate::interior::validate_circulation(&blocked).is_err());
}

#[test]
fn town_kitchen_and_support_share_an_accessible_vertical_bay() {
    let mut program = BuildingProgram::fixture(BuildingArchetype::TownHouse, 11);
    program.domestic_heating = Some(DomesticHeatingProgramme::HearthAndRearFedStove);
    let plan = generate(&program).unwrap();
    assert_eq!(plan.domestic_heating.as_ref().unwrap().floors.len(), 1);
    assert!(audit_plan(&plan).is_empty());
    crate::interior::validate_circulation(&plan).unwrap();
    let furniture = crate::interior::furnish(&plan, &program).unwrap();
    assert!(furniture.placements.iter().any(|p| p.storey == 0));
    assert!(furniture.placements.iter().any(|p| p.storey == 1));
}

#[test]
fn occupied_recipe_catalogue_selects_buildable_upper_heating() {
    for archetype in [
        BuildingArchetype::TownHouse,
        BuildingArchetype::FachwerkMerchantHouse,
    ] {
        for seed in [42, 47, 101] {
            let program = BuildingProgram::validated_settlement(
                archetype,
                adventuresim_world_schema::settlement_buildings::BuildingUse::Dwelling,
                seed,
                None,
            )
            .unwrap();
            let plan = generate(&program).unwrap();
            assert_eq!(
                plan.domestic_heating.as_ref().unwrap().kitchen.storey_level,
                1
            );
            crate::interior::furnish(&plan, &program).unwrap();
        }
    }
}
