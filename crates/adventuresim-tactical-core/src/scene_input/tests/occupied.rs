//! Occupied test drafts declare exact producer reservations before generation.
use super::*;
use crate::city_layout::{
    CityPlotBounds, CityPropertyId, CitySceneLayout, CitySingleProperty, CompoundGradingPolicy,
};
use adventuresim_building_generator::{BuildingArchetype, BuildingProgram};
use bevy::math::Vec2;

#[test]
fn bound_building_has_static_collision_without_rewriting_surrounding_source() {
    let mut input = fixture();
    let width = 41usize;
    let depth = 41usize;
    input.playable = TerrainSampleGrid {
        width: width as u16,
        depth: depth as u16,
        spacing_metres: 1.0,
        heights_metres: (0..width * depth)
            .map(|index| (index % width) as f32 * 0.04)
            .collect(),
        environment: vec![
            EnvironmentalSample {
                canopy_bps: 10_000,
                ..Default::default()
            };
            width * depth
        ],
    };
    input.buildings.push(TacticalBuildingPlacement {
        base_elevation_metres: crate::city_layout::grounding::SupportElevation::from_metres(2.0)
            .unwrap(),
        id: (7).into(),
        program: BuildingProgram::fixture(BuildingArchetype::FachwerkCottage, 42),
        centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(bevy::math::Vec2::ZERO)
            .unwrap(),
        orientation: BuildingOrientation::from_radians(core::f32::consts::FRAC_PI_2).unwrap(),
    });

    let source = input
        .prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        single_properties: vec![CitySingleProperty {
            id: CityPropertyId(7),
            building_id: (7).into(),
            plot: CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(Vec2::ZERO).unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    Vec2::splat(20.0),
                )
                .unwrap(),
                input.buildings[0].orientation,
            )
            .unwrap(),
        }],
        ..Default::default()
    };
    let before = input.buildings[0].clone();
    let input = input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap();
    let generated = input.generate().unwrap();
    assert_eq!(before.id, input.buildings[0].id);
    assert_eq!(before.program, input.buildings[0].program);
    assert_eq!(before.centre_metres, input.buildings[0].centre_metres);
    assert_eq!(before.orientation, input.buildings[0].orientation);
    for point in [Vec2::splat(-19.0), Vec2::splat(19.0)] {
        assert_eq!(
            generated.terrain.height_at(point),
            source.terrain.height_at(point)
        );
    }
    let building = &generated.buildings[0];
    assert_eq!(building.placement, input.buildings[0]);
    assert!(!building.collision.cuboids.is_empty());
    assert_eq!(generated.repairs.levelled_building_samples, 0);
    let centre_height = generated.terrain.height_at(bevy::math::Vec2::ZERO).unwrap();
    assert!((centre_height - building.placement.base_elevation_metres.metres()).abs() < 0.0001);
    assert_eq!(
        generated.ground.ground_at(bevy::math::Vec2::ZERO),
        Some(GroundSurface {
            substrate: GroundSubstrate::Stone,
            cover: GroundCover::Bare,
            cover_density_bps: 0,
            cover_height_cm: 0,
        })
    );
    let exclusion = building
        .collision
        .bounds
        .plan_half_extents()
        .unwrap()
        .metres()
        + bevy::math::Vec2::splat(5.5);
    assert!(generated.obstacles.iter().all(|obstacle| {
        let (x, z) = match *obstacle {
            GeneratedObstacle::Tree { x, z } | GeneratedObstacle::Rock { x, z, .. } => (x, z),
        };
        let point = bevy::math::Vec2::new(f32::from(x), f32::from(z))
            * input.playable.spacing_metres
            - bevy::math::Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
        let building_local = bevy::math::Vec2::new(point.y, -point.x).abs();
        !building_local.cmple(exclusion).all()
    }));
}

#[test]
fn distant_buildings_affect_scene_identity_without_entering_tactical_generation() {
    let mut input = fixture();
    input.vista.lods.push(VistaLod {
        level: crate::scene_input::VistaLevelIndex::new(0),
        width: 5,
        depth: 5,
        spacing_metres: 100.0,
        origin_east_metres: 0.0,
        origin_north_metres: 0.0,
        heights_metres: vec![0.0; 25],
        environment: vec![EnvironmentalSample::default(); 25],
    });
    let empty_digest = input.digest().unwrap();
    input.distant_buildings.push(DistantBuildingPlacement {
        prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
        usage: None,
        service_size: None,
        id: (1).into(),
        archetype: BuildingArchetype::TownHouse,
        seed: 42.into(),
        centre_metres: crate::scene_coordinates::ScenePlanPoint::try_from(bevy::math::Vec2::new(
            120.0, -90.0,
        ))
        .unwrap(),
        base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
        orientation: BuildingOrientation::from_radians(core::f32::consts::PI).unwrap(),
    });

    let original = input.distant_buildings[0];
    let layout = CitySceneLayout {
        distant: input.distant_buildings.clone(),
        single_properties: vec![CitySingleProperty {
            id: CityPropertyId(1),
            building_id: (1).into(),
            plot: CityPlotBounds::new(
                crate::scene_coordinates::ScenePlanPoint::try_from(original.centre_metres.metres())
                    .unwrap(),
                adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(
                    Vec2::splat(30.0),
                )
                .unwrap(),
                original.orientation,
            )
            .unwrap(),
        }],
        ..Default::default()
    };
    let input = input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap();
    let mut expected = original;
    expected.base_elevation_metres = input.distant_buildings[0].base_elevation_metres;
    assert_eq!(expected, input.distant_buildings[0]);
    assert_ne!(input.digest().unwrap(), empty_digest);
    assert_eq!(
        input
            .generate()
            .unwrap()
            .terrain
            .height_at(original.centre_metres.metres()),
        Some(expected.base_elevation_metres.metres())
    );
    assert!(input.generate().unwrap().buildings.is_empty());
}
