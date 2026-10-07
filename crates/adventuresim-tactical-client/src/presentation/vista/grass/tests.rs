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
    SceneTerrain::new(3, 3, 1.0, |_| 0.0)
        .unwrap()
        .with_property_surface(surface)
}

fn check(terrain: &SceneTerrain, check: impl FnOnce(VistaTuftPlacement<'_>)) {
    let mut input = TacticalSceneInput::load(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-scenes/sparse-woodland.json"
    )))
    .unwrap();
    let lod = input.vista.lods.first_mut().unwrap();
    lod.heights_metres.fill(20.0);
    let environment = SceneEnvironmentFixture::TemperateHills.snapshot("grass-grounding");
    let ground = SceneGround::uniform_for_terrain(terrain, Default::default());
    let urban = UrbanGround::new(&[], &[]);
    check(VistaTuftPlacement {
        lod,
        coarser_lod: None,
        playable_half_extent: Vec2::splat(1.0),
        playable_terrain: terrain,
        playable_ground: &ground,
        urban_ground: &urban,
        profile: GrassCommunityProfile::from_environment(&environment),
        communities: GrassCommunityField::new(fabelgeist_determinism::Seed::from_u64(42)),
        outer_collar: 100.0,
    });
}

#[test]
fn owned_vista_roots_use_the_accepted_surface_without_restitching_raw_heights() {
    let terrain = owned_terrain(0.0);
    check(&terrain, |placement| {
        for point in [
            Vec2::new(40.0, 30.0),
            Vec2::new(50.0, -20.0),
            Vec2::new(-45.0, -30.0),
        ] {
            let old_height = presented_vista_vertex_height(
                placement.lod,
                None,
                Some(&terrain),
                point,
                placement.playable_half_extent,
            )
            .unwrap();
            assert!(
                (old_height - 7.0).abs() > 0.1,
                "fixture must expose old restitching"
            );
            assert_eq!(placement.height(point), terrain.height_at(point));
            assert!((placement.height(point).unwrap() - 7.0).abs() < 0.001);
        }
    });
}

#[test]
fn owned_vista_roots_reject_missing_support_and_the_existing_steep_slope_limit() {
    check(&owned_terrain(0.0), |placement| {
        assert!(placement.height(Vec2::splat(400.0)).is_none())
    });
    check(&owned_terrain(1.0), |placement| {
        assert!(placement.height(Vec2::new(40.0, 30.0)).is_none())
    });
}

#[test]
fn sampled_vista_scenes_retain_their_presented_heightfield_policy() {
    let terrain = SceneTerrain::new(3, 3, 1.0, |_| 7.0).unwrap();
    check(&terrain, |placement| {
        let point = Vec2::new(40.0, 30.0);
        let expected = presented_vista_vertex_height(
            placement.lod,
            None,
            Some(&terrain),
            point,
            placement.playable_half_extent,
        )
        .unwrap();
        assert_eq!(placement.height(point), Some(expected));
    });
}
