use super::*;
use crate::prelude::FurnitureLocation;
use bevy::math::Vec2;

fn fixture() -> TacticalSceneInput {
    serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/compound-review.json"
    ))
    .unwrap()
}

#[test]
fn compound_loaded_scene_rejects_broken_membership_and_authority() {
    let input = fixture();
    input.validate().unwrap();
    let mut missing = input.clone();
    missing
        .buildings
        .retain(|b| b.id != missing.compounds[0].rear_building_id);
    assert!(missing.validate().is_err());
    let mut split = input.clone();
    let rear = split.buildings.remove(0);
    split.distant_buildings.push(DistantBuildingPlacement {
        prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
        id: rear.id,
        archetype: rear.program.archetype,
        usage: rear.program.usage,
        service_size: rear.program.service_size,
        seed: rear.program.seed,
        centre_metres: rear.centre_metres,
        orientation: rear.orientation,
        base_elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
    });
    assert!(split.validate().is_err());
    let mut malformed = input;
    malformed.compounds[0].boundary.gate.width_metres =
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(101.0)
            .unwrap();
    assert!(malformed.validate().is_err());
}

fn rebind(mut input: TacticalSceneInput) -> TacticalSceneInput {
    use crate::city_layout::{CitySceneLayout, CompoundGradingPolicy};
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        ..Default::default()
    };
    input.grounding = None;
    input
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap()
}

#[test]
fn sloped_compound_enclosures_bind_to_accepted_soil_without_rewriting_source_samples() {
    let mut input = fixture();
    for (i, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (i / usize::from(input.playable.width)) as f32 * 0.08;
    }
    let source = input.playable.clone();
    let original = input.buildings.clone();
    let input = rebind(input);
    assert_eq!(input.playable, source);
    for (before, after) in original.iter().zip(&input.buildings) {
        assert_eq!(before.id, after.id);
        assert_eq!(before.program, after.program);
        assert_eq!(before.centre_metres, after.centre_metres);
        assert_eq!(before.orientation, after.orientation);
    }
    let generated = input.generate().unwrap();
    assert_eq!(generated.buildings.len(), 4);
    assert_eq!(generated.boundaries.len(), 2);
    assert_eq!(generated.repairs.levelled_building_samples, 0);
    for compound in &input.compounds {
        let boundary = generated
            .boundaries
            .iter()
            .find(|boundary| boundary.scene.property_id == compound.id)
            .unwrap();
        let projected = GeneratedBoundary::project(compound, &generated.terrain).unwrap();
        assert_eq!(boundary.scene, projected.scene);
        assert_eq!(boundary.elevation_metres, projected.elevation_metres);
        for item in &generated.furniture.instances {
            if matches!(item.scene.location, FurnitureLocation::Interior { storey, .. } if storey.index() > 0)
            {
                continue;
            }
            let point = Vec2::new(
                item.position_metres.metres().x,
                item.position_metres.metres().z,
            );
            for route in &compound.access {
                let delta = route.end_metres() - route.start_metres();
                let t = ((point - route.start_metres()).dot(delta) / delta.length_squared())
                    .clamp(0.0, 1.0);
                assert!(
                    point.distance(route.start_metres() + delta * t) >= route.half_width_metres(),
                    "furniture {item:?} blocks property {:?}",
                    compound.id
                );
            }
        }
    }
}

#[test]
fn compound_rejects_a_loaded_route_ending_short_of_the_actual_store_door() {
    let mut input = fixture();
    let route = input.compounds[0].access.last_mut().unwrap();
    route
        .update_endpoints(
            route.start(),
            route
                .end()
                .translated(crate::scene_coordinates::PlanDisplacement::try_from(Vec2::X).unwrap())
                .unwrap(),
        )
        .unwrap();
    let result = input.generate();
    assert!(matches!(
        result,
        Err(SceneInputError::Validation(SceneValidationError::City(_)))
    ));
}

#[test]
fn adjacent_compounds_keep_separate_support_owners_after_terrain_refinement() {
    let mut input = fixture();
    // This proof intentionally builds a touching pair from one source property.
    input.compounds.truncate(1);
    let property = &input.compounds[0];
    input
        .buildings
        .retain(|b| [property.front_building_id, property.rear_building_id].contains(&b.id));
    input.yards.truncate(1);
    let offset = Vec2::new(16.5, 0.0);
    let mut neighbour = input.compounds[0].clone();
    neighbour.id.0 += 1;
    neighbour.front_building_id.0 += 1;
    neighbour.rear_building_id.0 += 1;
    neighbour
        .plot
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                neighbour.plot.centre_metres() + (offset),
            )
            .unwrap(),
        )
        .unwrap();
    neighbour
        .court
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                neighbour.court.centre_metres() + (offset),
            )
            .unwrap(),
        )
        .unwrap();
    neighbour.boundary.gate.centre_metres = neighbour
        .boundary
        .gate
        .centre_metres
        .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
        .unwrap();
    for route in &mut neighbour.access {
        route
            .update_endpoints(
                crate::scene_coordinates::ScenePlanPoint::try_from(route.start_metres() + (offset))
                    .unwrap(),
                route.end(),
            )
            .unwrap();
        route
            .update_endpoints(
                route.start(),
                crate::scene_coordinates::ScenePlanPoint::try_from(route.end_metres() + (offset))
                    .unwrap(),
            )
            .unwrap();
    }
    for wall in &mut neighbour.boundary.walls {
        wall.start_metres = wall
            .start_metres
            .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
            .unwrap();
        wall.end_metres = wall
            .end_metres
            .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
            .unwrap();
    }
    let neighbours = input
        .buildings
        .iter()
        .cloned()
        .map(|mut building| {
            building.id.0 += 1;
            building.centre_metres = building
                .centre_metres
                .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
                .unwrap();
            building
        })
        .collect::<Vec<_>>();
    input.buildings.extend(neighbours);
    input.compounds.push(neighbour);
    for (i, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (i % usize::from(input.playable.width)) as f32 * 1.25;
    }
    let input = rebind(input);
    let generated = input.generate().unwrap();
    assert_eq!(generated.boundaries.len(), 2);
    assert_ne!(
        generated.boundaries[0].scene.property_id,
        generated.boundaries[1].scene.property_id
    );
    for compound in &input.compounds {
        let boundary = generated
            .boundaries
            .iter()
            .find(|b| b.scene.property_id == compound.id)
            .unwrap();
        let exact = GeneratedBoundary::project(compound, &generated.terrain).unwrap();
        assert_eq!(exact.scene, boundary.scene);
        assert_eq!(exact.elevation_metres, boundary.elevation_metres);
        let foundation = generated.terrain.property_foundation(compound.id).unwrap();
        assert_eq!(
            foundation.member_building_ids(),
            [compound.front_building_id, compound.rear_building_id]
        );
    }
}
