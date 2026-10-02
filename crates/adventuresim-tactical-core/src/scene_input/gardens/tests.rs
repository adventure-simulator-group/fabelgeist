use super::*;
use bevy::math::Vec2;
fn fixture() -> TacticalSceneInput {
    serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/garden-review.json"
    ))
    .unwrap()
}

#[test]
fn garden_input_rejects_missing_ownership_and_escape_from_property() {
    let input = fixture();
    input.validate().unwrap();
    let mut broken = input.clone();
    broken.gardens[0].front_building_id += 1;
    assert!(broken.validate().is_err());
    let mut broken = input.clone();
    broken.buildings[0].centre_metres += Vec2::splat(30.0);
    assert!(broken.generate().is_err());
    let mut broken = input.clone();
    let plot = broken.gardens[0].plot;
    broken.gardens[0].access[0].end_metres.x =
        plot.centre_metres.x + plot.dimensions_metres.x * 0.5 - 0.01;
    assert!(broken.validate().is_err());
    let mut broken = input.clone();
    broken.gardens.push(broken.gardens[0].clone());
    assert!(broken.validate().is_err());
    let mut broken = input;
    broken.gardens[0].plants[0].scale = crate::city_layout::GardenPlantScale::new(f32::NAN);
    assert!(broken.validate().is_err());
}

#[test]
fn garden_on_slope_shares_the_owner_terrace_and_reserves_working_ground() {
    let mut input = fixture();
    for (index, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = (index / usize::from(input.playable.width)) as f32 * 0.25;
    }
    let generated = input.generate().unwrap();
    let garden = &generated.gardens[0];
    assert_eq!(garden.scene.garden, input.gardens[0]);
    assert_eq!(
        garden.elevation_metres,
        generated.buildings[0].pad_elevation_metres
    );
    for point in input.gardens[0]
        .cultivated_bounds
        .corners()
        .into_iter()
        .chain(input.gardens[0].plants.iter().map(|p| p.centre_metres))
    {
        assert!(
            (generated.terrain.height_at(point).unwrap() - garden.elevation_metres).abs() < 0.001
        );
        assert_eq!(
            generated.ground.ground_at(point).unwrap().cover_density_bps,
            0
        );
    }
    assert!(generated.furniture.instances.iter().all(|item| {
        !input.gardens[0]
            .cultivated_bounds
            .contains(Vec2::new(item.position_metres.x, item.position_metres.z))
    }));
}

fn boundary_fixture(offset: Vec2) -> TacticalSceneInput {
    let mut input = fixture();
    input.gardens.truncate(1);
    input.distant_buildings.clear();
    input.streets.truncate(1);
    input.yards.truncate(3);
    input.buildings[0].centre_metres += offset;
    let garden = &mut input.gardens[0];
    garden.plot.centre_metres += offset;
    garden.cultivated_bounds.centre_metres += offset;
    for bed in &mut garden.beds {
        bed.centre_metres += offset;
    }
    for route in &mut garden.access {
        route.start_metres += offset;
        route.end_metres += offset;
    }
    for plant in &mut garden.plants {
        plant.centre_metres += offset;
    }
    for yard in &mut input.yards {
        for point in &mut yard.corners_metres {
            *point += offset;
        }
    }
    for street in &mut input.streets {
        if let crate::city_layout::CityStreetPatch::Corridor {
            start_metres,
            end_metres,
            ..
        } = street
        {
            *start_metres += offset;
            *end_metres += offset;
        }
    }
    for lod in &mut input.vista.lods {
        lod.heights_metres.fill(0.0);
    }
    for (index, height) in input.playable.heights_metres.iter_mut().enumerate() {
        *height = index as f32 * 0.1;
    }
    input
}

#[test]
fn straddling_garden_anchors_the_complete_terrace_to_the_vista() {
    let input = boundary_fixture(Vec2::new(0.0, 45.0));
    assert!(input.gardens[0].plants[0].centre_metres.y > 50.0);
    let generated = input.generate().unwrap();
    assert_eq!(generated.gardens[0].elevation_metres, 0.0);
    assert_eq!(generated.buildings[0].pad_elevation_metres, 0.0);
    assert!(
        generated
            .ground
            .samples()
            .iter()
            .all(|surface| surface.substrate != crate::scene::GroundSubstrate::Stone),
        "level garden terraces and vista supports do not manufacture exposed stone"
    );
    for point in [Vec2::new(0.0, 49.0), Vec2::new(5.0, 50.0)] {
        assert!(generated.terrain.height_at(point).unwrap().abs() < 0.001);
    }
    let mut broken = input;
    broken.vista.lods[0]
        .heights_metres
        .iter_mut()
        .enumerate()
        .for_each(|(i, h)| *h = i as f32);
    assert!(
        broken.generate().is_err(),
        "unsupported mixed-height terrace must be rejected before presentation"
    );
}

#[test]
fn distant_garden_in_stitching_band_retains_its_accepted_pose_on_sloped_playable_ground() {
    let mut input = boundary_fixture(Vec2::new(0.0, 70.0));
    let owner = input.buildings.remove(0);
    input.distant_buildings.push(DistantBuildingPlacement {
        id: owner.id,
        archetype: owner.program.archetype,
        usage: owner.program.usage,
        service_size: owner.program.service_size,
        seed: owner.program.seed,
        centre_metres: owner.centre_metres,
        orientation: owner.orientation,
        base_elevation_metres: 0.0,
    });
    let accepted = input.gardens[0].plants.clone();
    let generated = input.generate().unwrap();
    assert!(
        generated.gardens.is_empty(),
        "distant owner remains presentation authority"
    );
    assert_eq!(input.gardens[0].plants, accepted);
    let lod = &input.vista.lods[0];
    for point in accepted[0]
        .world_hull()
        .into_iter()
        .chain(std::iter::once(accepted[0].centre_metres))
    {
        let height = crate::vista_surface::vista_triangle_height(
            lod,
            input.vista.lods.get(1),
            &generated.terrain,
            point,
        )
        .unwrap();
        assert!(
            height.abs() < 0.001,
            "distant specimen must sit on the actual stitched triangle at {point:?}: {height}"
        );
    }
}
