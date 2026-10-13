use super::*;
use bevy::math::Vec2;
fn fixture() -> TacticalSceneInput {
    serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/garden-review.json"
    ))
    .unwrap()
}

#[test]
fn explicit_garden_audit_checks_detailed_geometry() {
    let input = fixture();
    let mut generated = input.generate().unwrap();
    input.audit_garden_clearance(&generated).unwrap();
    generated.buildings[0].placement.centre_metres =
        crate::scene_coordinates::ScenePlanPoint::try_from(
            input.gardens[0].cultivated_bounds.centre_metres(),
        )
        .unwrap();
    assert!(input.audit_garden_clearance(&generated).is_err());
}

#[test]
fn garden_input_rejects_missing_ownership_and_escape_from_property() {
    let input = fixture();
    input.validate().unwrap();
    let mut broken = input.clone();
    broken.gardens[0].front_building_id.0 += 1;
    assert!(broken.validate().is_err());
    let mut broken = input.clone();
    broken.buildings[0].centre_metres = broken.buildings[0]
        .centre_metres
        .translated(
            crate::scene_coordinates::PlanDisplacement::try_from(Vec2::splat(30.0)).unwrap(),
        )
        .unwrap();
    assert!(broken.generate().is_err());
    let mut broken = input.clone();
    let plot = broken.gardens[0].plot;
    let route = &mut broken.gardens[0].access[0];
    route
        .update_endpoints(
            route.start(),
            crate::scene_coordinates::ScenePlanPoint::try_from(bevy::math::Vec2::new(
                plot.centre_metres().x + plot.dimensions_metres().x * 0.5 - 0.01,
                route.end_metres().y,
            ))
            .unwrap(),
        )
        .unwrap();
    assert!(broken.validate().is_err());
    let mut broken = input.clone();
    broken.gardens.push(broken.gardens[0].clone());
    assert!(broken.validate().is_err());
    assert!(crate::city_layout::GardenPlantScale::new(f32::NAN).is_none());
}

fn rebind(input: TacticalSceneInput) -> TacticalSceneInput {
    use crate::city_layout::{CitySceneLayout, CitySingleProperty, CompoundGradingPolicy};
    let layout = CitySceneLayout {
        playable: input.buildings.clone(),
        distant: input.distant_buildings.clone(),
        compounds: input.compounds.clone(),
        gardens: input.gardens.clone(),
        streets: input.streets.clone(),
        yards: input.yards.clone(),
        parishes: input.parishes.clone(),
        single_properties: input
            .gardens
            .iter()
            .map(|garden| CitySingleProperty {
                id: garden.owner,
                building_id: garden.front_building_id,
                plot: garden.plot,
            })
            .collect(),
        ..Default::default()
    };
    let mut draft = input;
    draft.grounding = None;
    draft
        .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
        .unwrap()
}

fn sloped_fixture() -> TacticalSceneInput {
    let mut input = fixture();
    let half = f32::from(input.playable.depth - 1) * 0.5;
    for (index, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (index / usize::from(input.playable.width)) as f32
            * 0.03
            * input.playable.spacing_metres
            - half * 0.03 * input.playable.spacing_metres;
    }
    for lod in &mut input.vista.lods {
        let half = f32::from(lod.depth - 1) * 0.5;
        for (index, height) in lod.heights_metres.iter_mut().enumerate() {
            *height = ((index / usize::from(lod.width)) as f32 - half) * lod.spacing_metres * 0.03;
        }
    }
    input
}

#[test]
fn sloped_garden_preserves_soil_relief_and_binds_each_retained_root() {
    let input = rebind(sloped_fixture());
    let natural = input
        .prepare_geographic_terrain(&mut GeneratedBuildingRecipes::default())
        .unwrap();
    let source = crate::city_layout::grounding::GeographicSurface::from_presented_scene(
        &natural.terrain,
        &input.vista,
    )
    .unwrap();
    let generated = input.generate().unwrap();
    let garden = &generated.gardens[0];
    assert_eq!(garden.garden(), &input.gardens[0]);
    assert_eq!(garden.plant_support().len(), garden.garden().plants.len());
    for (plant, support) in garden.garden().plants.iter().zip(garden.plant_support()) {
        assert_eq!(plant.id, support.plant_id);
        assert!(
            (support.elevation.metres()
                - generated
                    .terrain
                    .height_at(plant.centre_metres.metres())
                    .unwrap())
            .abs()
                < 0.001
        );
        assert!(
            (support.elevation.metres()
                - source.elevation_at(plant.centre_metres).unwrap().metres())
            .abs()
                < 0.001
        );
    }
    assert!(garden.plant_support().iter().any(|root| {
        (root.elevation.metres()
            - generated.buildings[0]
                .placement
                .base_elevation_metres
                .metres())
        .abs()
            > 0.1
    }));
    for point in garden.garden().cultivated_bounds.corners() {
        assert!(
            (generated.terrain.height_at(point).unwrap()
                - source
                    .elevation_at(
                        crate::scene_coordinates::ScenePlanPoint::try_from(point).unwrap()
                    )
                    .unwrap()
                    .metres())
            .abs()
                < 0.001
        );
        if let Some(ground) = generated.ground.ground_at(point) {
            assert_eq!(ground.cover_density_bps, 0);
        }
    }
    assert!(generated.furniture.instances.iter().all(|item| {
        !garden.garden.cultivated_bounds.contains(Vec2::new(
            item.position_metres.metres().x,
            item.position_metres.metres().z,
        ))
    }));
    let encoded = serde_json::to_vec(garden).unwrap();
    let restored: SceneGarden = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&restored, garden);
}

fn move_first_property(mut input: TacticalSceneInput, offset: Vec2) -> TacticalSceneInput {
    input.gardens.truncate(1);
    input.distant_buildings.clear();
    input.streets.truncate(1);
    input.yards.truncate(3);
    input.buildings[0].centre_metres = input.buildings[0]
        .centre_metres
        .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
        .unwrap();
    let garden = &mut input.gardens[0];
    garden
        .plot
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                garden.plot.centre_metres() + (offset),
            )
            .unwrap(),
        )
        .unwrap();
    garden
        .cultivated_bounds
        .relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                garden.cultivated_bounds.centre_metres() + (offset),
            )
            .unwrap(),
        )
        .unwrap();
    for bed in &mut garden.beds {
        bed.relocate(
            crate::scene_coordinates::ScenePlanPoint::try_from(bed.centre_metres() + (offset))
                .unwrap(),
        )
        .unwrap();
    }
    for route in &mut garden.access {
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
    for plant in &mut garden.plants {
        plant.centre_metres = plant
            .centre_metres
            .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
            .unwrap();
    }
    for yard in &mut input.yards {
        for point in &mut yard.corners_metres {
            *point = point
                .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
                .unwrap();
        }
    }
    for street in &mut input.streets {
        if let crate::city_layout::CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            ..
        } = street
        {
            *start_metres = start_metres
                .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
                .unwrap();
            *end_metres = end_metres
                .translated(crate::scene_coordinates::PlanDisplacement::try_from(offset).unwrap())
                .unwrap();
        }
    }
    input
}

#[test]
fn straddling_garden_roots_follow_the_same_stitched_soil_inside_and_outside_playable_bounds() {
    let input = rebind(move_first_property(sloped_fixture(), Vec2::new(0.0, 45.0)));
    let generated = input.generate().unwrap();
    assert!(
        generated.gardens[0]
            .garden
            .plants
            .iter()
            .any(|plant| plant.centre_metres.metres().y > 50.0)
    );
    let projected = SceneGarden::project(input.gardens[0].clone(), &generated.terrain).unwrap();
    assert_eq!(generated.gardens[0], projected);
    for support in projected.plant_support() {
        let point = projected
            .garden
            .plants
            .iter()
            .find(|plant| plant.id == support.plant_id)
            .unwrap()
            .centre_metres;
        assert!(
            (support.elevation.metres() - generated.terrain.height_at(point.metres()).unwrap())
                .abs()
                < 0.001
        );
    }
    let mut stale = input;
    let lod = &mut stale.vista.lods[0];
    let index =
        usize::from(lod.depth / 2) * usize::from(lod.width) + usize::from(lod.width / 2 + 2);
    lod.heights_metres[index] += 0.1;
    assert!(
        matches!(
            stale.generate(),
            Err(SceneInputError::GroundingProjection(_))
        ),
        "roots cannot bypass stale source support"
    );
}

#[test]
fn distant_garden_projection_retains_roots_membership_and_horizontal_geometry() {
    let mut draft = move_first_property(sloped_fixture(), Vec2::new(0.0, 70.0));
    let owner = draft.buildings.remove(0);
    draft.distant_buildings.push(DistantBuildingPlacement {
        prosperity: adventuresim_world_schema::ProsperityTier::Comfortable,
        id: owner.id,
        archetype: owner.program.archetype,
        usage: owner.program.usage,
        service_size: owner.program.service_size,
        seed: owner.program.seed,
        centre_metres: owner.centre_metres,
        orientation: owner.orientation,
        base_elevation_metres: owner.base_elevation_metres,
    });
    let accepted = draft.gardens[0].clone();
    let input = rebind(draft);
    let generated = input.generate().unwrap();
    assert!(
        generated.gardens.is_empty(),
        "distant owner retains presentation authority"
    );
    let projected = SceneGarden::project(input.gardens[0].clone(), &generated.terrain).unwrap();
    assert_eq!(projected.garden(), &accepted);
    for (plant, support) in projected
        .garden()
        .plants
        .iter()
        .zip(projected.plant_support())
    {
        assert_eq!(plant.id, support.plant_id);
        assert!(
            (generated
                .terrain
                .height_at(plant.centre_metres.metres())
                .unwrap()
                - support.elevation.metres())
            .abs()
                < 0.001
        );
    }
}
