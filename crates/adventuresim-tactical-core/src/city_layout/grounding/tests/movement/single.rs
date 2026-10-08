//! Single-building entrances traverse their exact accepted full-width approaches.
use super::*;
use crate::scene_input::{GeneratedBuildingRecipe, GeneratedBuildingRecipes, TacticalSceneInput};
use adventuresim_building_generator::{BuildingEntranceSupport, compile_ground_entrances};

#[test]
#[ignore = "requires a frozen production export in FABELGEIST_TERRAIN_SCENE"]
fn production_single_entrances_allow_full_width_entry_and_return() {
    let input = TacticalSceneInput::load(std::path::Path::new(
        &std::env::var("FABELGEIST_TERRAIN_SCENE").unwrap(),
    ))
    .unwrap();
    let selected = std::env::var("FABELGEIST_SINGLE_PROPERTY")
        .ok()
        .map(|id| id.parse::<u64>().unwrap());
    let scene = input
        .generate_unfurnished(GeneratedBuildingRecipes::default())
        .unwrap();
    let outdoor = outdoor::OutdoorCollision::compile(
        &input,
        &scene.terrain,
        &scene.obstacles,
        &scene.furniture,
    );
    let terrain = scene.terrain.colliders().unwrap();
    let mut buildings = scene.buildings;
    for distant in &input.distant_buildings {
        let placement = crate::scene_input::TacticalBuildingPlacement::from(*distant);
        let recipe = GeneratedBuildingRecipe::generate(placement.program.clone()).unwrap();
        buildings.push(crate::scene_input::GeneratedBuilding {
            placement,
            plan: recipe.plan,
            collision: recipe.collision,
        });
    }
    let buildings: Vec<_> = buildings
        .into_iter()
        .map(gardens::OccupiedBuilding::from)
        .collect();
    let mut observations = Vec::new();
    for owner in input.grounding.as_ref().unwrap().surfaces() {
        if owner.member_building_ids().len() != 1
            || selected.is_some_and(|id| id != owner.property_id().0)
        {
            continue;
        }
        let building = &buildings
            .iter()
            .find(|b| b.building.placement.id == owner.member_building_ids()[0])
            .unwrap()
            .building;
        let transform = building.transform().unwrap();
        let origin = building.collision.bounds.centre().unwrap().metres().xz();
        let mut doors: Vec<_> = compile_ground_entrances(&building.plan)
            .unwrap()
            .into_iter()
            .filter(|door| door.support == BuildingEntranceSupport::ArchitecturalFloor)
            .collect();
        doors.sort_by_key(|door| door.id);
        assert_eq!(doors.len(), owner.support_regions().len() - 1);
        for (door, apron) in doors.into_iter().zip(&owner.support_regions()[1..]) {
            if let Ok(selected) = std::env::var("FABELGEIST_SINGLE_ENTRANCE") {
                let selected: adventuresim_building_generator::BuildingEntranceId =
                    serde_json::from_str(&selected).unwrap();
                if door.id != selected {
                    continue;
                }
            }
            let outward = building
                .placement
                .orientation
                .local_to_world(door.outward.vector());
            let tangent = Vec2::new(outward.y, -outward.x);
            let threshold = building.placement.centre_metres.metres()
                + building
                    .placement
                    .orientation
                    .local_to_world(door.threshold_metres.metres() - origin);
            let outer = apron.centre_metres() + outward * apron.dimensions_metres().y * 0.5;
            let skin = CharacterController::default().move_and_slide.skin_width;
            let tolerance = crate::city_layout::CompoundGradingPolicy::bounded_settlement()
                .limits
                .contact_tolerance_metres();
            let edge_clearance = apron.dimensions_metres().x * 0.5
                - HUMANOID_COLLISION_RADIUS_METRES
                - skin
                - tolerance;
            assert!(edge_clearance >= 0.0);
            let tracks = if std::env::var_os("FABELGEIST_CENTRELINE_ONLY").is_some() {
                vec![0.0]
            } else {
                vec![-edge_clearance, 0.0, edge_clearance]
            };
            let tracks = if let Ok(index) = std::env::var("FABELGEIST_SINGLE_TRACK") {
                vec![tracks[index.parse::<usize>().unwrap()]]
            } else {
                tracks
            };
            for lateral in tracks {
                let offset = tangent * lateral;
                let start = outer + outward * 0.35 + offset;
                let target = threshold - outward * 0.35 + offset;
                let mut walker = gardens::occupied_walker(
                    start,
                    threshold,
                    apron.dimensions_metres().length() + 6.0,
                    &scene.terrain,
                    &terrain,
                    &buildings,
                    &outdoor,
                    &input,
                    None,
                );
                let arrival = walker.attempt_walk_to(target);
                let arrival_distance = arrival.xz().distance(target);
                let return_position = if arrival_distance < 0.08 {
                    walker.attempt_walk_to(start)
                } else {
                    arrival
                };
                let return_distance = return_position.xz().distance(start);
                let passed = arrival_distance < 0.08 && return_distance < 0.08;
                observations.push(serde_json::json!({"property_id":owner.property_id(),
                    "building_id":building.placement.id,"entrance":door.id,
                    "lateral_metres":lateral,"start_metres":start,"target_metres":target,
                    "arrival_metres":arrival,"arrival_shortfall_metres":arrival_distance,
                    "return_metres":return_position,"return_shortfall_metres":return_distance,"permitted_metres":0.08,"pass":passed,
                    "floor_metres":building.placement.base_elevation_metres.metres(),
                    "transform_origin_metres":transform.translation}));
                if !passed {
                    walker.capture_step_casts(return_position);
                    break;
                }
            }
            if observations.last().is_some_and(|o| o["pass"] == false) {
                break;
            }
        }
        println!(
            "single {}: {}",
            owner.property_id().0,
            if observations.last().is_some_and(|o| o["pass"] == false) {
                "FAIL"
            } else {
                "pass"
            }
        );
        if observations.last().is_some_and(|o| o["pass"] == false) {
            break;
        }
    }
    if let Ok(path) = std::env::var("FABELGEIST_SINGLE_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "input_digest":input.digest().unwrap(),"observations":observations,
            "scope":"Production KCC with complete canonical terrain and exact nearby fixed geometry and production tactical obstacles and outdoor furniture. Unfurnished interiors; vista furniture has no physics. Open building entrances, closed compound gates; centre and both usable-width edges, entry and return."
        })).unwrap()).unwrap();
    }
    assert!(!observations.is_empty());
    assert!(
        observations.iter().all(|o| o["pass"] == true),
        "{:?}",
        observations.iter().find(|o| o["pass"] == false)
    );
}

#[test]
fn goslar_2_pedestrian_starts_on_the_source_clipped_rotated_stair() {
    let mesh: PropertyFoundationMesh = serde_json::from_str(include_str!(
        "../../../../../../../assets/tactical-grounding/goslar-2-stair-contact.json"
    ))
    .unwrap();
    let start = Vec2::new(26.832619, -31.933275);
    let mut walker = Walker::on_collider(
        mesh.collider().unwrap().into_solid().unwrap(),
        start,
        0.89393,
    );
    walker.walk_to(Vec2::new(26.88, -32.45));
    walker.walk_to(start);
}

#[test]
fn a_steep_physical_bearing_face_remains_unwalkable() {
    let surface = Collider::compound(vec![
        (
            Vec3::new(0.0, -0.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(4.0, 1.0, 4.0),
        ),
        (
            Vec3::new(0.0, 0.9, -1.05),
            Quat::from_rotation_x(60.0_f32.to_radians()),
            Collider::cuboid(2.0, 0.2, 2.0),
        ),
    ]);
    let mut walker = Walker::on_collider(surface, Vec2::new(0.0, 0.0), 0.0);
    let position = walker.attempt_walk_to(Vec2::new(0.0, -1.1));
    assert!(position.z > -0.5, "steep face was traversed: {position:?}");
}
