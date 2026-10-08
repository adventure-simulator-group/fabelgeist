//! Complete imported compounds retain surrounding fixed collision geometry.
use super::*;
use crate::scene_input::{GeneratedBuildingRecipes, TacticalSceneInput};

#[test]
#[ignore = "requires a frozen production export in FABELGEIST_TERRAIN_SCENE"]
fn production_compound_routes_allow_entry_thresholds_and_return() {
    let input = TacticalSceneInput::load(std::path::Path::new(
        &std::env::var("FABELGEIST_TERRAIN_SCENE").unwrap(),
    ))
    .unwrap();
    let selected = std::env::var("FABELGEIST_COMPOUND_PROPERTY")
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
    let mut properties: Vec<_> = input.compounds.iter().collect();
    properties.sort_by_key(|property| property.id);
    let mut observations = Vec::new();
    for property in properties {
        if selected.is_some_and(|id| property.id.0 != id) {
            continue;
        }
        let gate = property.boundary.gate.centre_metres;
        let passage = property
            .access
            .iter()
            .find(|route| route.contains_centreline(gate))
            .unwrap();
        let mut walker = gardens::occupied_walker(
            passage.start_metres(),
            property.plot.centre_metres(),
            property.plot.dimensions_metres().length() + 6.0,
            &scene.terrain,
            &terrain,
            &buildings,
            &outdoor,
            &input,
            Some(property.id),
        );
        let targets = [
            property.boundary.gate.centre_metres.metres(),
            passage.end_metres(),
            property.access[1].end_metres(),
            property.access[2].end_metres(),
            property.access[2].start_metres(),
            passage.end_metres(),
            property.access[3].end_metres(),
            property.access[4].end_metres(),
            property.access[4].start_metres(),
            passage.end_metres(),
            property.boundary.gate.centre_metres.metres(),
            passage.start_metres(),
        ];
        let mut visits = Vec::new();
        for (index, target) in targets.into_iter().enumerate() {
            let arrived = walker.attempt_walk_to(target);
            let distance = arrived.xz().distance(target);
            let passed = distance < 0.08;
            visits.push(
                serde_json::json!({"target_index":index,"target_metres":target,
                "arrived_metres":arrived,"horizontal_shortfall_metres":distance,
                "permitted_metres":0.08,"pass":passed}),
            );
            if !passed {
                walker.capture_step_casts(arrived);
                break;
            }
        }
        let passed = visits.len() == targets.len() && visits.iter().all(|v| v["pass"] == true);
        observations.push(serde_json::json!({"property_id":property.id,
            "member_building_ids":[property.front_building_id,property.rear_building_id],
            "pass":passed,"visits":visits}));
        println!(
            "compound {}: {}",
            property.id.0,
            if passed { "pass" } else { "FAIL" }
        );
        if !passed {
            break;
        }
    }
    assert!(
        !observations.is_empty(),
        "no exact requested compound present"
    );
    if let Ok(path) = std::env::var("FABELGEIST_COMPOUND_REPORT") {
        std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({
            "input_digest":input.digest().unwrap(),"observations":observations,
            "scope":"Production KCC, complete canonical terrain, exact nearby fixed buildings, enclosures and production tactical obstacles and outdoor furniture. Unfurnished interiors; vista furniture has no physics. Selected gate open; other gates closed. Street, passage, both court thresholds and return; gate sweep checked separately."
        })).unwrap()).unwrap();
    }
    assert!(
        observations.iter().all(|row| row["pass"] == true),
        "{:?}",
        observations.iter().find(|row| row["pass"] == false)
    );
}
