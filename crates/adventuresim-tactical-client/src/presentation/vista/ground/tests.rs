use super::*;
use adventuresim_tactical_core::city_layout::grounding::{
    BoundedSettlementTerrain, FoundationEmbedment, GeographicSurface,
};

fn owned_terrain(grade: f32) -> SceneTerrain {
    let point = |x, z| Vec3::new(x, 7.0 + x * grade, z);
    let source = GeographicSurface::from_triangles([
        [
            point(-256.0, -256.0),
            point(-256.0, 256.0),
            point(256.0, -256.0),
        ],
        [
            point(256.0, -256.0),
            point(-256.0, 256.0),
            point(256.0, 256.0),
        ],
    ])
    .unwrap();
    let surface = BoundedSettlementTerrain::compile(
        &[],
        &source,
        FoundationEmbedment::from_metres(0.2).unwrap(),
    )
    .unwrap();
    SceneTerrain::new(3, 3, 1.0, |_| 0.0).with_property_surface(surface)
}

fn raw_lod() -> VistaLod {
    VistaLod {
        level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(0),
        spacing_metres: 50.0,
        width: 11,
        depth: 11,
        origin_east_metres: 0.0,
        origin_north_metres: 0.0,
        heights_metres: vec![20.0; 121],
        environment: vec![EnvironmentalSample::default(); 121],
    }
}

#[test]
fn owned_vista_tree_roots_follow_support_across_detail_levels() {
    let terrain = owned_terrain(0.0);
    let lod = raw_lod();
    let mut coarse = lod.clone();
    coarse.level = adventuresim_tactical_core::scene_input::VistaLevelIndex::new(1);
    coarse.spacing_metres = 250.0;
    for point in [Vec2::new(40.0, 30.0), Vec2::new(-45.0, -30.0)] {
        assert!((presented_height_at(&lod, point, Some(&coarse)).unwrap() - 7.0).abs() > 1.0);
        for (lod, next) in [(&lod, Some(&coarse)), (&coarse, None)] {
            assert!((tree_root_height(&terrain, lod, next, point).unwrap() - 7.0).abs() < 0.001);
        }
    }
    assert!(tree_root_height(&terrain, &coarse, None, Vec2::splat(400.0)).is_none());
}

#[test]
fn owned_vista_rocks_use_support_normal_and_keep_the_existing_slope_gate() {
    let lod = raw_lod();
    let point = Vec2::new(40.0, 30.0);
    let terrain = owned_terrain(0.5);
    let hit = terrain
        .surface_below(Vec3::new(point.x, f32::INFINITY, point.y))
        .unwrap();
    let height = hit.elevation.metres();
    let normal = *hit.normal;
    let rock = vista_scatter_transform(&lod, None, &terrain, Vec2::ONE, point, 42, 0.08).unwrap();
    assert!((rock.translation.y - height - 0.08).abs() < 0.001);
    assert!((rock.rotation * Vec3::Y - normal).length() < 0.001);
    assert!(
        vista_scatter_transform(&lod, None, &owned_terrain(1.0), Vec2::ONE, point, 42, 0.08)
            .is_none()
    );
    assert!(
        vista_scatter_transform(
            &lod,
            None,
            &terrain,
            Vec2::ONE,
            Vec2::splat(400.0),
            42,
            0.08
        )
        .is_none()
    );
}

#[test]
fn sampled_vista_trees_and_rocks_retain_the_presented_heightfield() {
    let terrain = SceneTerrain::new(3, 3, 1.0, |_| 7.0);
    let lod = raw_lod();
    let point = Vec2::new(40.0, 30.0);
    assert_eq!(
        tree_root_height(&terrain, &lod, None, point),
        presented_height_at(&lod, point, None)
    );
    let expected =
        presented_vista_vertex_height(&lod, None, Some(&terrain), point, Vec2::ONE).unwrap();
    let rock = vista_scatter_transform(&lod, None, &terrain, Vec2::ONE, point, 42, 0.08).unwrap();
    assert!((rock.translation.y - expected - 0.08).abs() < 0.001);
}

#[test]
#[ignore = "requires the accepted Goslar terrain input"]
fn goslar_property_1236_vista_scenery_uses_accepted_support() {
    let path = std::env::var_os("FABELGEIST_SCENERY_ACCEPTANCE_INPUT")
        .expect("set the accepted Goslar input path");
    let input = TacticalSceneInput::load(std::path::Path::new(&path)).unwrap();
    let generated = input.generate_unfurnished(Default::default()).unwrap();
    let point = Vec2::new(-298.93, -344.28);
    let terrain = &generated.terrain;
    let surface = terrain.property_surface().unwrap();
    let foundation = surface
        .foundations
        .iter()
        .find(|f| f.property_id == adventuresim_tactical_core::city_layout::CityPropertyId(1236))
        .unwrap();
    let owner = GeographicSurface::from_triangles(
        foundation
            .support_triangles
            .iter()
            .map(|t| t.map(|i| foundation.positions[i as usize])),
    )
    .unwrap();
    assert!(
        owner.elevation_at(point).is_some(),
        "reported point must lie on property 1236"
    );
    let hit = terrain
        .surface_below(Vec3::new(point.x, f32::INFINITY, point.y))
        .unwrap();
    let height = hit.elevation.metres();
    let normal = *hit.normal;
    let lod = &input.vista.lods[0];
    let coarser = input.vista.lods.get(1);
    let raw = presented_height_at(lod, point, coarser).unwrap();
    assert!(
        (height - raw).abs() > 1.0,
        "required reproduction: {height} vs {raw}"
    );
    assert!((tree_root_height(terrain, lod, coarser, point).unwrap() - height).abs() < 0.001);
    let rock = vista_scatter_transform(
        lod,
        coarser,
        terrain,
        Vec2::new(terrain.width(), terrain.depth()) * 0.5,
        point,
        42,
        0.08,
    )
    .unwrap();
    assert!((rock.translation.y - height - 0.08).abs() < 0.001);
    assert!((rock.rotation * Vec3::Y - normal).length() < 0.001);
    println!(
        "Goslar property 1236 at {point:?}: raw {raw} m, accepted {height} m, normal {normal:?}"
    );
}

#[test]
#[ignore = "requires the frozen terrain acceptance input manifest"]
fn required_city_vista_scenery_matches_property_support() {
    let manifest = std::env::var_os("FABELGEIST_PARTITION_ACCEPTANCE_MANIFEST")
        .expect("set the absolute acceptance manifest path");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = manifest["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 12);
    for fixture in fixtures {
        let input =
            TacticalSceneInput::load(&workspace.join(fixture["input"].as_str().unwrap())).unwrap();
        let generated = input.generate_unfurnished(Default::default()).unwrap();
        let terrain = &generated.terrain;
        let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let mut examined = 0;
        for owner in &terrain.property_surface().unwrap().foundations {
            let triangle = owner
                .support_triangles
                .first()
                .expect("accepted property has support");
            let point = triangle
                .iter()
                .map(|i| owner.positions[*i as usize])
                .sum::<Vec3>()
                / 3.0;
            let world = Vec2::new(point.x, point.z);
            let hit = terrain
                .surface_below(Vec3::new(world.x, f32::INFINITY, world.y))
                .unwrap();
            let height = hit.elevation.metres();
            let normal = *hit.normal;
            assert!(
                (height - point.y).abs() < 0.001,
                "fixture {}, property {:?}",
                fixture["fixture"],
                owner.property_id
            );
            for (i, lod) in input.vista.lods.iter().take(2).enumerate() {
                let next = input.vista.lods.get(i + 1);
                assert!(
                    (tree_root_height(terrain, lod, next, world).unwrap() - point.y).abs() < 0.001
                );
                let rock = vista_scatter_transform(lod, next, terrain, half, world, 42, 0.08);
                if normal.y >= MINIMUM_VISTA_ROCK_SLOPE_NORMAL_Y {
                    let rock = rock.expect("accepted shallow support retains a rock candidate");
                    assert!((rock.translation.y - point.y - 0.08).abs() < 0.001);
                    assert!((rock.rotation * Vec3::Y - normal).length() < 0.001);
                } else {
                    assert!(rock.is_none());
                }
                examined += 1;
            }
        }
        assert!(examined > 0);
        println!(
            "{}: {examined} property/LOD scenery samples pass support",
            fixture["fixture"]
        );
    }
}
