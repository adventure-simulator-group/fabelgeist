//! Capture promotion retains the producer's source, geometry and bindings.
use super::*;
use crate::tactical_scene_viewer::fixed_city_cameras::{CONTRACT_VERSION, Document, PROFILE};
use adventuresim_tactical_core::city_layout::CityGroundingProjectionError;
use adventuresim_tactical_core::prelude::FurnitureLocation;
use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes;

fn terrain_bytes(scene: &GeneratedTacticalScene) -> Vec<u8> {
    serde_json::to_vec(&scene.terrain).unwrap()
}

#[test]
fn capture_near_edge_promotion_preserves_accepted_surface_and_complete_bindings() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/massive-city.json");
    let input = TacticalSceneInput::load(&path).unwrap();
    let input_bytes = serde_json::to_vec(&input).unwrap();
    let before = input.generate().unwrap();
    let member = *input.distant_buildings.iter().find(|b| b.id == 15).unwrap();
    let far_member = *input.distant_buildings.last().unwrap();
    let contract = Contract(Some(Document {
        version: CONTRACT_VERSION,
        exterior: Vec::new(),
        benchmark: None,
        playable_building_ids: vec![input.buildings[0].id, member.id, far_member.id],
    }));
    let after = contract.generate(&input).unwrap();
    assert!(
        terrain_bytes(&before) == terrain_bytes(&after),
        "capture must retain exact accepted terrain bytes"
    );
    assert_eq!(before.digest, after.digest);
    assert_eq!(
        serde_json::to_vec(&before.ground).unwrap(),
        serde_json::to_vec(&after.ground).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&before.obstacles).unwrap(),
        serde_json::to_vec(&after.obstacles).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&before.repairs).unwrap(),
        serde_json::to_vec(&after.repairs).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&before.boundaries).unwrap(),
        serde_json::to_vec(&after.boundaries[..before.boundaries.len()]).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&before.gardens).unwrap(),
        serde_json::to_vec(&after.gardens[..before.gardens.len()]).unwrap()
    );
    assert_eq!(input_bytes, serde_json::to_vec(&input).unwrap());
    assert_eq!(after.buildings.len(), before.buildings.len() + 2);
    for member in [member, far_member] {
        let generated = after
            .buildings
            .iter()
            .find(|b| b.placement.id == member.id)
            .unwrap();
        assert_eq!(generated.placement, member.into());
        let expected = GeneratedBuildingRecipe::generate(member.occupied_program()).unwrap();
        assert_eq!(
            serde_json::to_vec(&expected.plan).unwrap(),
            serde_json::to_vec(&generated.plan).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&expected.collision).unwrap(),
            serde_json::to_vec(&generated.collision).unwrap()
        );
    }
    let distant =
        crate::tactical_scene_viewer::buildings::distant_placements(&input, &after.buildings);
    assert!(
        !distant
            .iter()
            .any(|b| [member.id, far_member.id].contains(&b.id))
    );
    assert_eq!(
        distant.len() + after.buildings.len(),
        input.distant_buildings.len() + input.buildings.len()
    );
    let mut before_placements: Vec<_> = input
        .buildings
        .iter()
        .cloned()
        .chain(input.distant_buildings.iter().copied().map(Into::into))
        .collect();
    let mut after_placements: Vec<_> = after
        .buildings
        .iter()
        .map(|b| b.placement.clone())
        .chain(distant.into_iter().map(Into::into))
        .collect();
    before_placements.sort_by_key(|b| b.id);
    after_placements.sort_by_key(|b| b.id);
    assert_eq!(before_placements, after_placements);
    let outdoors = |scene: &GeneratedTacticalScene| {
        scene
            .furniture
            .instances
            .iter()
            .filter(|f| matches!(f.scene.location, FurnitureLocation::Outdoor { .. }))
            .copied()
            .collect::<Vec<_>>()
    };
    assert_eq!(outdoors(&before), outdoors(&after));
    assert!(
        after
            .furniture
            .interiors
            .iter()
            .any(|i| i.building_id == member.id)
    );
}

#[test]
fn promoted_garden_owner_retains_planting_and_accepted_root_elevations() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/garden-review.json");
    let input = TacticalSceneInput::load(&path).unwrap();
    let member = input.distant_buildings[0];
    let contract = Contract(Some(Document {
        version: CONTRACT_VERSION,
        exterior: Vec::new(),
        benchmark: None,
        playable_building_ids: vec![member.id],
    }));
    let generated = contract.generate(&input).unwrap();
    let garden = input
        .gardens
        .iter()
        .find(|g| g.front_building_id == member.id)
        .unwrap();
    let projected = generated
        .gardens
        .iter()
        .find(|g| g.garden.owner == garden.owner)
        .unwrap();
    assert_eq!(projected.garden, *garden);
    assert_eq!(
        *projected,
        SceneGarden::project(garden.clone(), &generated.terrain).unwrap()
    );
    assert_eq!(projected.plant_support.len(), garden.plants.len());
    assert_eq!(generated.gardens.len(), input.gardens.len());
    assert!(
        !crate::tactical_scene_viewer::buildings::distant_placements(&input, &generated.buildings)
            .iter()
            .any(|b| b.id == member.id)
    );
}

#[test]
fn ordinary_and_empty_capture_selection_preserve_complete_renderer_inputs() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/garden-review.json");
    let input = TacticalSceneInput::load(&path).unwrap();
    let before = input.generate().unwrap();
    let contracts = [
        Contract(None),
        Contract(Some(Document {
            version: CONTRACT_VERSION,
            exterior: Vec::new(),
            benchmark: None,
            playable_building_ids: Vec::new(),
        })),
    ];
    for contract in contracts {
        let after = contract.generate(&input).unwrap();
        assert!(serde_json::to_vec(&before).unwrap() == serde_json::to_vec(&after).unwrap());
        assert_eq!(
            crate::tactical_scene_viewer::buildings::distant_placements(&input, &after.buildings),
            input.distant_buildings
        );
    }
}

#[test]
fn altering_the_source_partition_still_rejects_a_stale_projection() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/tactical-scenes/massive-city.json");
    let mut input = TacticalSceneInput::load(&path).unwrap();
    let index = input
        .distant_buildings
        .iter()
        .position(|b| b.id == 15)
        .unwrap();
    let member = input.distant_buildings.remove(index);
    input.buildings.push(member.into());
    assert!(matches!(
        input.generate_unfurnished(GeneratedBuildingRecipes::default()),
        Err(SceneInputError::GroundingProjection(error))
            if matches!(*error, CityGroundingProjectionError::SourceMismatch)
    ));
}

#[test]
#[ignore = "requires the frozen production fixture/camera manifest"]
fn required_capture_members_preserve_all_accepted_production_terrain() {
    let manifest =
        std::path::PathBuf::from(std::env::var("FABELGEIST_CAPTURE_FIXTURE_MANIFEST").unwrap());
    let root = std::path::PathBuf::from(std::env::var("FABELGEIST_CAPTURE_FIXTURE_ROOT").unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    for fixture in manifest["fixtures"].as_array().unwrap() {
        let input =
            TacticalSceneInput::load(&root.join(fixture["input"].as_str().unwrap())).unwrap();
        let before = input.generate().unwrap();
        let contract = Contract::load(
            Some(root.join(fixture["camera"].as_str().unwrap())),
            PROFILE,
        );
        let after = contract.generate(&input).unwrap();
        assert_eq!(
            before.digest, after.digest,
            "fixture {}",
            fixture["fixture"]
        );
        assert!(
            terrain_bytes(&before) == terrain_bytes(&after),
            "fixture {}",
            fixture["fixture"]
        );
        let distant =
            crate::tactical_scene_viewer::buildings::distant_placements(&input, &after.buildings);
        assert_eq!(
            distant.len() + after.buildings.len(),
            input.distant_buildings.len() + input.buildings.len()
        );
        let visible_owners: BTreeSet<_> = after.buildings.iter().map(|b| b.placement.id).collect();
        let expected_boundaries: Vec<_> = input
            .compounds
            .iter()
            .filter(|p| visible_owners.contains(&p.front_building_id))
            .collect();
        assert_eq!(after.boundaries.len(), expected_boundaries.len());
        for compound in expected_boundaries {
            let boundary = after
                .boundaries
                .iter()
                .find(|b| b.scene.property_id == compound.id)
                .unwrap();
            assert_eq!(boundary.scene.front_building_id, compound.front_building_id);
            assert_eq!(boundary.scene.boundary, compound.boundary);
            assert_eq!(
                serde_json::to_vec(boundary).unwrap(),
                serde_json::to_vec(&GeneratedBoundary::project(compound, &after.terrain).unwrap())
                    .unwrap()
            );
        }
        let expected_gardens: Vec<_> = input
            .gardens
            .iter()
            .filter(|g| visible_owners.contains(&g.front_building_id))
            .collect();
        assert_eq!(after.gardens.len(), expected_gardens.len());
        for garden in expected_gardens {
            let planting = after
                .gardens
                .iter()
                .find(|g| g.garden.owner == garden.owner)
                .unwrap();
            assert_eq!(planting.garden, *garden);
            assert_eq!(
                *planting,
                SceneGarden::project(garden.clone(), &after.terrain).unwrap()
            );
        }
        input.validate().unwrap();
    }
}
