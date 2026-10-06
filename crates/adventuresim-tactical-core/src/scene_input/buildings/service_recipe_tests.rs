use super::*;
use adventuresim_building_generator::{ServiceBuildingSize, settlement_archetype};
use adventuresim_world_schema::settlement_buildings::BuildingUse;

#[test]
fn sized_church_and_workplace_recipes_survive_distant_transport_with_playable_geometry() {
    for usage in [
        BuildingUse::Chapel,
        BuildingUse::ParishChurch,
        BuildingUse::Stable,
    ] {
        for size in [
            ServiceBuildingSize::Small,
            ServiceBuildingSize::Medium,
            ServiceBuildingSize::Large,
        ] {
            let archetype = settlement_archetype(usage);
            let program =
                BuildingProgram::validated_settlement(archetype, usage, 42, Some(size)).unwrap();
            let orientation = BuildingOrientation::from_radians(0.37).unwrap();
            let centre_metres = Vec2::new(80.0, 35.0);
            let playable = prepare_buildings(
                &[TacticalBuildingPlacement {
                    base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
                    id: (1).into(),
                    program: program.clone(),
                    centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(
                        centre_metres,
                    )
                    .unwrap(),
                    orientation,
                }],
                &mut crate::scene_input::GeneratedBuildingRecipes::default(),
            )
            .unwrap()
            .pop()
            .unwrap();
            let distant = DistantBuildingPlacement {
                prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
                id: (1).into(),
                archetype,
                usage: Some(usage),
                service_size: program.service_size,
                seed: program.seed.into(),
                centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(centre_metres)
                    .unwrap(),
                base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
                orientation,
            };
            let encoded = serde_json::to_string(&distant).unwrap();
            let restored: DistantBuildingPlacement = serde_json::from_str(&encoded).unwrap();
            assert_eq!(restored.service_size, Some(size));
            let reconstructed = restored.occupied_program();
            assert_eq!(reconstructed, playable.placement.program);
            assert_eq!(
                reconstructed.plot_dimensions_metres(),
                program.plot_dimensions_metres()
            );
            let distant_plan = generate(&reconstructed).unwrap();
            assert!(!playable.collision.cuboids.is_empty());
            assert_eq!(
                compile_building_collision(&distant_plan).unwrap(),
                playable.collision,
                "{usage:?} {size:?} changed physical geometry after distant reconstruction"
            );
        }
    }
}
