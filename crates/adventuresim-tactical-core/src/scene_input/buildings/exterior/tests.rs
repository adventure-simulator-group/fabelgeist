use super::*;
use crate::scene_input::{BuildingOrientation, TacticalSceneInput};
use adventuresim_building_generator::{ServiceBuildingSize, generate};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::math::Vec2;

fn placement(archetype: BuildingArchetype) -> DistantBuildingPlacement {
    DistantBuildingPlacement {
        id: 1,
        prosperity: ProsperityTier::Wealthy,
        archetype,
        usage: None,
        service_size: None,
        seed: 42,
        centre_metres: Vec2::ZERO,
        base_elevation_metres: 0.0,
        orientation: BuildingOrientation::IDENTITY,
    }
}

#[test]
fn thousands_of_different_lots_use_only_three_exteriors_per_family() {
    for archetype in BuildingArchetype::ALL {
        let mut programs = Vec::new();
        for id in 1..=1_000 {
            let building = DistantBuildingPlacement {
                id,
                seed: id * 17,
                ..placement(archetype)
            };
            let program = building.exterior_program();
            assert_eq!(program, building.exterior_program());
            if !programs.contains(&program) {
                programs.push(program);
            }
        }
        assert_eq!(programs.len(), 3, "{archetype:?}");
        for program in programs {
            generate(&program).unwrap_or_else(|error| panic!("{archetype:?}: {error}"));
        }
    }
}

#[test]
fn occupations_share_exteriors_without_changing_the_occupied_recipe() {
    let mut building = placement(BuildingArchetype::Workplace);
    let expected = building.exterior_program();
    for usage in [
        BuildingUse::Stable,
        BuildingUse::Barn,
        BuildingUse::Warehouse,
    ] {
        for size in [ServiceBuildingSize::Small, ServiceBuildingSize::Large] {
            building.usage = Some(usage);
            building.service_size = Some(size);
            assert_eq!(building.exterior_program(), expected);
            assert_eq!(building.occupied_program().usage, Some(usage));
            assert_eq!(building.occupied_program().service_size, Some(size));
            let scale = building.exterior_scale(&expected);
            assert!(scale > 0.0 && scale <= 1.0);
            assert!(
                (expected.plot_dimensions_metres() * scale)
                    .cmple(
                        building.occupied_program().plot_dimensions_metres() + Vec2::splat(0.001)
                    )
                    .all()
            );
        }
    }
}

#[test]
fn prosperity_limits_ornate_houses_but_preserves_landmark_frontages() {
    let mut building = placement(BuildingArchetype::FachwerkMerchantHouse);
    let occupied = building.occupied_program();
    for tier in [ProsperityTier::Subsistence, ProsperityTier::Modest] {
        building.prosperity = tier;
        assert_eq!(
            building.exterior_program().archetype,
            BuildingArchetype::TownHouse
        );
        assert_eq!(building.occupied_program(), occupied);
    }
    building.prosperity = ProsperityTier::Wealthy;
    assert_eq!(
        building.exterior_program().archetype,
        BuildingArchetype::FachwerkMerchantHouse
    );
    building.archetype = BuildingArchetype::ParishChurch;
    building.usage = Some(BuildingUse::ParishChurch);
    building.service_size = Some(ServiceBuildingSize::Large);
    assert_eq!(
        building.exterior_program().frontage_direction(),
        building.occupied_program().frontage_direction()
    );
    assert_eq!(
        building.exterior_program().plot_dimensions_metres(),
        building.occupied_program().plot_dimensions_metres()
    );
}

#[test]
fn massive_city_shares_prototypes_and_preserves_scene_round_trip() {
    let input: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let mut occupied = Vec::new();
    let mut exteriors = Vec::new();
    for building in &input.distant_buildings {
        let original = building.occupied_program();
        let exterior = building.exterior_program();
        if !occupied.contains(&original) {
            occupied.push(original);
        }
        if !exteriors.contains(&exterior) {
            exteriors.push(exterior);
        }
    }
    assert!(exteriors.len() * 3 < occupied.len());
    assert!(exteriors.len() <= BuildingArchetype::ALL.len() * 3);
    println!(
        "{} distant buildings: {} occupied recipes -> {} exterior prototypes",
        input.distant_buildings.len(),
        occupied.len(),
        exteriors.len()
    );
    let restored: TacticalSceneInput =
        serde_json::from_str(&serde_json::to_string(&input).unwrap()).unwrap();
    assert_eq!(restored, input);
    for program in exteriors {
        generate(&program).unwrap();
    }
}
