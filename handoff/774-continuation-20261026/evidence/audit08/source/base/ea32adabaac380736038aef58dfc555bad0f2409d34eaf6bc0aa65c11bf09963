//! Behavioural reference for exact pre-field colours, including weather and seams.
use super::*;

fn varied_lod(level: u8, spacing: f32) -> VistaLod {
    let samples = (0..25)
        .map(|i| EnvironmentalSample {
            surface: [
                TacticalSurface::Open,
                TacticalSurface::DeepWoods,
                TacticalSurface::Wetland,
                TacticalSurface::Road,
                TacticalSurface::Water,
            ][i % 5],
            canopy_bps: (i * 379) as u16,
            wetland_bps: [0, 3_999, 4_000, 8_000, 10_000][i % 5],
            cultivation_bps: [0, 3_999, 4_000, 7_000, 10_000][(i / 5) % 5],
            water_bps: [0, 4_999, 5_000, 8_000, 10_000][(i + 2) % 5],
            hilly_bps: (i * 401) as u16,
            crossing_bps: 0,
        })
        .collect();
    VistaLod {
        level: adventuresim_tactical_core::scene_input::VistaLevelIndex::new(level),
        spacing_metres: spacing,
        width: 5,
        depth: 5,
        origin_east_metres: 125.0,
        origin_north_metres: -275.0,
        heights_metres: vec![0.0; 25],
        environment: samples,
    }
}

#[test]
fn prepared_pigments_preserve_weather_interpolation_and_edge_bytes() {
    let own = varied_lod(0, 50.0);
    let coarser = varied_lod(1, 250.0);
    for (moisture, snow) in [(0, 0), (7_500, 0), (10_000, 6_500)] {
        let weather = WeatherSnapshot {
            ground_moisture_bps: moisture,
            snow_cover_bps: snow,
            ..clear_vista_weather()
        };
        let environment = SceneEnvironment {
            weather,
            ..TacticalSceneInput::load(std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/tactical-scenes/flat-dry-grassland.json"
            )))
            .unwrap()
            .environment_snapshot("pigment-fixture".into())
        };
        for coarse in [None, Some(&coarser)] {
            for playable in [None, Some(&environment)] {
                let fields = VistaVertexColors::new(&own, coarse, playable, weather);
                for z in -17..=17 {
                    for x in -17..=17 {
                        let point = Vec2::new(x as f32 * 6.25, z as f32 * 6.25);
                        let expected = uncached_vertex_color(
                            &own,
                            coarse,
                            playable,
                            point,
                            Vec2::new(37.0, 23.0),
                            weather,
                        );
                        let actual = fields.at(point, Vec2::new(37.0, 23.0));
                        assert_eq!(
                            actual.map(|c| c.map(f32::to_bits)),
                            expected.map(|c| c.map(f32::to_bits)),
                            "weather {moisture}/{snow}, point {point:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the frozen terrain acceptance input manifest"]
fn required_owned_mesh_pigments_preserve_uncached_vertex_bytes() {
    let manifest = std::env::var_os("FABELGEIST_COLOR_ACCEPTANCE_MANIFEST")
        .expect("set the absolute fixed-camera manifest path");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = manifest["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 12);
    for fixture in fixtures {
        let input =
            TacticalSceneInput::load(&workspace.join(fixture["input"].as_str().unwrap())).unwrap();
        let generated = input.generate_unfurnished(Default::default()).unwrap();
        let environment = input.environment_snapshot(generated.digest);
        let mut inner = Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
        let mut examined = 0usize;
        for (i, lod) in input.vista.lods.iter().enumerate() {
            let coarser = input.vista.lods.get(i + 1);
            let playable = (i == 0).then_some(&environment);
            let meshes = owned::vista_meshes(
                &generated.terrain,
                lod,
                inner,
                coarser,
                playable,
                environment.weather,
                None,
            );
            for mesh in meshes {
                let positions = mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap();
                let bevy::mesh::VertexAttributeValues::Float32x4(colors) =
                    mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
                else {
                    panic!("RGBA colour format");
                };
                assert_eq!(positions.len(), colors.len());
                for (point, actual) in positions.iter().zip(colors) {
                    let expected = uncached_vertex_color(
                        lod,
                        coarser,
                        playable,
                        Vec2::new(point[0], point[2]),
                        inner,
                        environment.weather,
                    )
                    .unwrap();
                    assert_eq!(
                        actual.map(f32::to_bits),
                        expected.map(f32::to_bits),
                        "fixture {}, LOD {}, point {point:?}",
                        fixture["fixture"],
                        lod.level
                    );
                    examined += 1;
                }
            }
            inner = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1))
                * lod.spacing_metres
                * 0.5;
        }
        assert!(examined > 0);
        println!(
            "{}: {examined} exact ordinary terrain vertex colours",
            fixture["fixture"]
        );
    }
}

fn uncached_vertex_color(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_environment: Option<&SceneEnvironment>,
    local: Vec2,
    playable_half_extent: Vec2,
    weather: WeatherSnapshot,
) -> Option<[f32; 4]> {
    let world = local
        + Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let vista_color = Vec4::from_array(presented_color_at(lod, world, coarser_lod, weather)?);
    Some(
        playable_environment
            .map(|environment| {
                stitch_vista_color_to_playable_edge(
                    local,
                    playable_half_extent,
                    // Ground-cover proportions summarize a wider ecological
                    // patch than height samples. Ease pigment over several
                    // vista cells so the playable rectangle cannot read as a
                    // terrain tile from an overhead or grazing camera.
                    lod.spacing_metres * 4.0,
                    vista_color,
                    Vec4::from_array(scene_ground_color(environment).to_linear().to_f32_array()),
                )
            })
            .unwrap_or(vista_color)
            .to_array(),
    )
}

fn presented_color_at(
    lod: &VistaLod,
    world: Vec2,
    coarser_lod: Option<&VistaLod>,
    weather: WeatherSnapshot,
) -> Option<[f32; 4]> {
    let own = sample_vista_color(lod, world, weather)?;
    let Some(coarser) = coarser_lod else {
        return Some(own.to_array());
    };
    let weight = lod_transition_weight(lod, coarser, world);
    Some(
        sample_vista_color(coarser, world, weather)
            .map(|color| own.lerp(color, weight))
            .unwrap_or(own)
            .to_array(),
    )
}

fn sample_vista_color(lod: &VistaLod, world: Vec2, weather: WeatherSnapshot) -> Option<Vec4> {
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    let local = world
        - Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
    let coordinate =
        local / lod.spacing_metres + Vec2::new((width - 1) as f32 * 0.5, (depth - 1) as f32 * 0.5);
    if coordinate.x < 0.0
        || coordinate.y < 0.0
        || coordinate.x > (width - 1) as f32
        || coordinate.y > (depth - 1) as f32
    {
        return None;
    }
    let lower = coordinate.floor().as_uvec2();
    let upper = (lower + UVec2::ONE).min(UVec2::new(width as u32 - 1, depth as u32 - 1));
    let fraction = coordinate.fract();
    let at = |x: u32, z: u32| {
        vista_sample_color(lod.environment[z as usize * width + x as usize], weather)
    };
    let near = at(lower.x, lower.y).lerp(at(upper.x, lower.y), fraction.x);
    let far = at(lower.x, upper.y).lerp(at(upper.x, upper.y), fraction.x);
    Some(near.lerp(far, fraction.y))
}
