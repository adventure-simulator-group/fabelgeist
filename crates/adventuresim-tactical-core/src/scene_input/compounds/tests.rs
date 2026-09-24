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
        id: rear.id,
        archetype: rear.program.archetype,
        usage: rear.program.usage,
        service_size: rear.program.service_size,
        seed: rear.program.seed,
        centre_metres: rear.centre_metres,
        orientation: rear.orientation,
        base_elevation_metres: 0.0,
    });
    assert!(split.validate().is_err());
    let mut malformed = input;
    malformed.compounds[0].boundary.gate.width_metres = f32::NAN;
    assert!(malformed.validate().is_err());
}

#[test]
fn compound_levels_court_routes_and_both_members_together_on_sloped_ground() {
    let mut input = fixture();
    for (i, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (i / usize::from(input.playable.width)) as f32 * 0.25;
    }
    let generated = input.generate().unwrap();
    assert_eq!(generated.buildings.len(), 4);
    assert_eq!(generated.boundaries.len(), 2);
    for compound in &input.compounds {
        let elevation = generated
            .boundaries
            .iter()
            .find(|boundary| boundary.scene.property_id == compound.id)
            .unwrap()
            .elevation_metres;
        assert!(
            generated
                .buildings
                .iter()
                .filter(|b| [compound.front_building_id, compound.rear_building_id]
                    .contains(&b.placement.id))
                .all(|b| b.pad_elevation_metres == elevation)
        );
        for route in &compound.access {
            for fraction in [0.25, 0.5, 0.75, 1.0] {
                let p = route.start_metres.lerp(route.end_metres, fraction);
                assert!(
                    (generated.terrain.height_at(p).unwrap() - elevation).abs() < 0.03,
                    "courtyard route is not level at {p:?}"
                );
            }
        }
        for item in &generated.furniture.instances {
            // Upper-room furniture cannot obstruct this ground-level route.
            if matches!(item.scene.location, FurnitureLocation::Interior { storey, .. } if storey > 0)
            {
                continue;
            }
            let point = Vec2::new(item.position_metres.x, item.position_metres.z);
            for route in &compound.access {
                let delta = route.end_metres - route.start_metres;
                let t = ((point - route.start_metres).dot(delta) / delta.length_squared())
                    .clamp(0.0, 1.0);
                assert!(
                    point.distance(route.start_metres + delta * t) >= route.half_width_metres,
                    "furniture {item:?} blocks property {:?} route {route:?}",
                    compound.id
                );
            }
        }
    }
}

#[test]
fn compound_rejects_a_loaded_route_ending_short_of_the_actual_store_door() {
    let mut input = fixture();
    input.compounds[0].access.last_mut().unwrap().end_metres += Vec2::X;
    let result = input.generate();
    assert!(
        matches!(result, Err(SceneInputError::Validation(message)) if message.contains("MissingRangeDoor"))
    );
}

#[test]
fn adjacent_compound_walls_remain_grounded_after_terrain_refinement() {
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
    neighbour.front_building_id += 1;
    neighbour.rear_building_id += 1;
    neighbour.plot.centre_metres += offset;
    neighbour.court.centre_metres += offset;
    neighbour.boundary.gate.centre_metres += offset;
    for route in &mut neighbour.access {
        route.start_metres += offset;
        route.end_metres += offset;
    }
    for wall in &mut neighbour.boundary.walls {
        wall.start_metres += offset;
        wall.end_metres += offset;
    }
    let neighbours = input
        .buildings
        .iter()
        .cloned()
        .map(|mut building| {
            building.id += 1;
            building.centre_metres += offset;
            building
        })
        .collect::<Vec<_>>();
    input.buildings.extend(neighbours);
    input.compounds.push(neighbour);
    for (i, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (i % usize::from(input.playable.width)) as f32 * 1.25;
    }
    let generated = input.generate().unwrap();
    assert_eq!(generated.boundaries.len(), 2);
    assert_eq!(
        generated.boundaries[0].elevation_metres,
        generated.boundaries[1].elevation_metres
    );
    for boundary in &generated.boundaries {
        for member in boundary.scene.boundary.fixed_members() {
            let point = Vec2::new(member.centre_metres.x, member.centre_metres.z);
            assert!(
                (generated.terrain.height_at(point).unwrap() - boundary.elevation_metres).abs()
                    < 0.01,
                "boundary is detached from the refined ground at {point:?}"
            );
        }
    }
}
