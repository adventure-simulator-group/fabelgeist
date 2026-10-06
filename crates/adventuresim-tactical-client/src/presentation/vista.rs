mod details;
pub(in crate::presentation) mod grass;
mod natural;
use details::*;
mod pigment;
use pigment::{VistaVertexColors, vista_sward_coverage};
#[cfg(test)]
use pigment::{presented_color, stitch_vista_color_to_playable_edge, vista_sample_color};
mod ground;
use ground::{tree_root_height, vista_lod_meshes_with_morph, vista_scatter_transform};
pub(super) mod owned;
pub(super) mod pending;
mod streams;
use super::ground_scatter::TacticalGrassInstancedMaterial;
use super::*;
use adventuresim_tactical_core::scene_input::VistaLevelIndex;
use adventuresim_tactical_core::vista_surface::*;
use grass::spawn_near_vista_scatter;

pub(super) mod streets;
mod surface;

pub(crate) use streets::CityGroundMaterial;
use streets::UrbanGround;
pub(super) use surface::ActiveVistaSurface;
pub(crate) use surface::{VistaTerrain, VistaTerrainMesh};

/// Marker for a distant tree billboard spawned as part of a vista ring.
#[derive(Component)]
pub(crate) struct VistaTreePresentation;

/// Presentation-only scatter outside the authoritative gameplay heightfield.
#[derive(Component, Clone, Copy)]
pub(crate) struct VistaGrassPresentation;

#[derive(Component)]
pub(crate) struct VistaRockPresentation;

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects vista scene state, presentation asset stores, and the shared tree cache independently"
)]
pub(super) fn on_scene_vista_bundle(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    mut active_surface: ResMut<ActiveVistaSurface>,
    mut pending: ResMut<pending::PendingVista>,
    existing: Query<Entity, With<VistaTerrain>>,
    playable_scenes: Query<(
        &SceneTerrain,
        &SceneGround,
        &SceneEnvironment,
        Option<&TerrainLandformRecipe>,
    )>,
    settings: Res<TacticalGraphicsSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TacticalVistaMaterial>>,
    mut grass_materials: ResMut<Assets<TacticalGrassInstancedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut tree_materials: ResMut<Assets<TacticalTreeImpostorMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut vista_tree_cache: ResMut<VistaTreePresentationCache>,
    prepared_tree_impostors: PreparedTreeImpostors,
    mut city_ground: streets::CityGroundAssets,
) {
    let playable_scene = playable_scenes
        .iter()
        .find(|(_, _, environment, _)| environment.scene_digest == bundle.scene_digest);
    if !pending.accept(&bundle, playable_scene.iter().map(|scene| scene.2)) {
        return;
    }
    let started = web_time::Instant::now();
    let mut presented_chunk_count = 0_usize;
    info!("Generating tactical vista presentation");
    active_surface.update(&bundle);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let visible_lods = bundle
        .lods
        .iter()
        .take(settings.config.rendering.vista.maximum_lods)
        .collect::<Vec<_>>();
    let (terrain, ground, environment, landform) =
        playable_scene.expect("matching canonical terrain");
    let playable_terrain = Some(terrain);
    let playable_environment = Some(environment);
    active_surface.retain_playable(terrain, landform);
    let weather = environment.weather;
    let vista_grass_color = grass_terminal_pigment(environment);
    let material = materials.add(vista_material(weather, vista_grass_color));
    let mut inner_half_extent = bundle.playable_half_extent_metres;
    for (index, lod) in visible_lods.iter().copied().enumerate() {
        let meshes_for_lod = vista_lod_meshes_with_morph(
            lod,
            inner_half_extent,
            visible_lods.get(index + 1).copied(),
            playable_terrain,
            (index == 0).then_some(playable_environment).flatten(),
            weather,
            landform.map(|recipe| recipe.transition_collar()),
        );
        if meshes_for_lod.is_empty() {
            let level = lod.level.index();
            warn!(level, "Rejected malformed tactical vista LOD");
            continue;
        }
        let half_extent = f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres * 0.5;
        for (chunk, mesh) in meshes_for_lod.into_iter().enumerate() {
            presented_chunk_count += 1;
            active_surface.present_chunk(
                &mut commands,
                &mut meshes,
                material.clone(),
                mesh,
                lod,
                chunk,
            );
        }
        if index <= 1 {
            spawn_vista_trees(
                &mut commands,
                lod,
                visible_lods.get(index + 1).copied(),
                inner_half_extent,
                &bundle.scene_digest,
                Some(environment),
                terrain,
                &mut meshes,
                &mut tree_materials,
                &mut images,
                &mut vista_tree_cache,
                prepared_tree_impostors.get(),
            );
        }
        inner_half_extent = Vec2::new(
            half_extent,
            f32::from(lod.depth.saturating_sub(1)) * lod.spacing_metres * 0.5,
        );
    }
    if let Some(lod) = visible_lods.first().copied() {
        spawn_near_vista_details(
            &mut commands,
            &bundle,
            &active_surface,
            lod,
            visible_lods.get(1).copied(),
            terrain,
            ground,
            environment,
            &mut meshes,
            &mut grass_materials,
            &mut standard_materials,
            &mut images,
            &settings.config.grass,
            &mut city_ground,
        );
    }
    log_vista_generation(presented_chunk_count, visible_lods.len(), started);
}

fn log_vista_generation(chunks: usize, lods: usize, started: web_time::Instant) {
    info!(
        chunks,
        lods,
        elapsed_ms = started.elapsed().as_millis(),
        "Generated tactical vista presentation"
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "near-vista details share the scene stores already injected at the observer boundary"
)]
fn spawn_near_vista_details(
    commands: &mut Commands,
    bundle: &SceneVistaBundle,
    active_surface: &ActiveVistaSurface,
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: &SceneTerrain,
    playable_ground: &SceneGround,
    environment: &SceneEnvironment,
    meshes: &mut Assets<Mesh>,
    grass_materials: &mut Assets<TacticalGrassInstancedMaterial>,
    standard_materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    grass: &crate::presentation::config::GrassConfig,
    city_ground: &mut streets::CityGroundAssets,
) {
    city_ground.spawn(
        commands,
        bundle,
        active_surface.ground_support(),
        environment,
        meshes,
        images,
    );
    let urban_ground = UrbanGround::new(&bundle.streets, &bundle.yards);
    spawn_near_vista_scatter(
        commands,
        lod,
        coarser_lod,
        bundle.playable_half_extent_metres,
        playable_terrain,
        playable_ground,
        environment,
        meshes,
        grass_materials,
        grass,
        &urban_ground,
    );
    spawn_vista_rocks(
        commands,
        lod,
        coarser_lod,
        bundle.playable_half_extent_metres,
        playable_terrain,
        stable_text_seed(&environment.scene_digest),
        meshes,
        standard_materials,
    );
}

const VISTA_GRASS_BOUNDARY_STITCH_METRES: f32 = 12.0;
fn smoothstep01(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn stitched_vista_topology_coverage(
    lod: &VistaLod,
    playable_half_extent: Vec2,
    playable_ground: &SceneGround,
    point: Vec2,
    urban_ground: &UrbanGround,
) -> f32 {
    if urban_ground.suppresses_grass(point) {
        return 0.0;
    }
    let boundary = point.clamp(-playable_half_extent, playable_half_extent);
    let playable_coverage = playable_ground
        .ground_at(boundary)
        .filter(|sample| sample.cover == GroundCover::TallGrass)
        .map_or(0.0, |sample| bps(sample.cover_density_bps));
    let outside = (point.abs() - playable_half_extent)
        .max(Vec2::ZERO)
        .max_element();
    if outside <= 0.0 {
        return playable_coverage;
    }
    let vista_coverage = sample_vista_environment(lod, point)
        .map(vista_sward_coverage)
        .unwrap_or(0.0);
    playable_coverage.lerp(
        vista_coverage,
        smoothstep01(outside / VISTA_GRASS_BOUNDARY_STITCH_METRES),
    )
}

#[cfg(test)]
pub(super) fn vista_lod_meshes(lod: &VistaLod, inner_half_extent: Vec2) -> Vec<Mesh> {
    vista_lod_meshes_with_morph(
        lod,
        inner_half_extent,
        None,
        None,
        None,
        clear_vista_weather(),
        None,
    )
}

#[cfg(test)]
fn presented_height(
    lod: &VistaLod,
    x: usize,
    z: usize,
    world: Vec2,
    coarser_lod: Option<&VistaLod>,
) -> f32 {
    let own = lod.heights_metres[z * usize::from(lod.width) + x];
    let Some(coarser) = coarser_lod else {
        return own;
    };
    let weight = lod_transition_weight(lod, coarser, world);
    sample_vista_height(coarser, world)
        .map(|height| own.lerp(height, weight))
        .unwrap_or(own)
}

fn clear_vista_weather() -> WeatherSnapshot {
    WeatherSnapshot {
        rules_version: WEATHER_RULES_VERSION,
        interval_start_minute: adventuresim_world_schema::calendar::StrategicMinute::new(0),
        cell_latitude: 0,
        cell_longitude: 0,
        temperature_deci_c: 100,
        wind_speed_bps: 0,
        precipitation: Precipitation::Clear,
        intensity_bps: 0,
        ground_moisture_bps: 0,
        snow_cover_bps: 0,
        atmosphere: Default::default(),
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(in crate::presentation) struct TacticalVistaExtension {
    #[uniform(100)]
    weather: Vec4,
    #[uniform(100)]
    grass_color: Vec4,
}

impl MaterialExtension for TacticalVistaExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/tactical_vista.wgsl".into()
    }
}

pub(in crate::presentation) type TacticalVistaMaterial =
    ExtendedMaterial<StandardMaterial, TacticalVistaExtension>;

fn vista_material(weather: WeatherSnapshot, grass_color: Color) -> TacticalVistaMaterial {
    TacticalVistaMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.94,
            metallic: 0.0,
            ..default()
        },
        extension: TacticalVistaExtension {
            weather: Vec4::new(
                bps(weather.ground_moisture_bps),
                bps(weather.snow_cover_bps),
                bps(weather.wind_speed_bps),
                0.0,
            ),
            grass_color: color_vec4(grass_color),
        },
    }
}

// Chunking is a CPU/ECS submission boundary, not a visual tessellation
// boundary. Thirty-two cells retains the exact terrain vertices while cutting
// a 50 km three-ring vista to about one sixteenth as many render entities.
const VISTA_CHUNK_CELLS: usize = 32;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn vista_ground_uses_continuous_palette_colors_and_geometry_normals() {
        let shader = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/shaders/tactical_vista.wgsl"
        ));
        assert!(!shader.contains("texture_2d"));
        assert!(!shader.contains("textureSample"));
        assert!(!shader.contains("composed_normal"));
        assert!(shader.contains("let sward_color = vista.grass_color.rgb"));
        assert!(shader.contains("color = mix(color, sward_target, sward)"));
        assert!(!shader.contains("sward_color = color *"));
        assert!(shader.contains("let molded_rock = vec3<f32>(0.31, 0.30, 0.275)"));
    }

    #[test]
    fn vista_terminal_sward_uses_the_optically_compensated_grass_pigment() {
        let pigment = Color::srgb_u8(91, 126, 47);
        let material = vista_material(clear_vista_weather(), pigment);
        assert_eq!(material.extension.grass_color, color_vec4(pigment));
    }

    #[test]
    fn vista_grass_reuses_the_playable_terminal_sward_handoff() {
        // `spawn_near_vista_scatter` uses this same range for its globally
        // aligned lattice, so the playable-to-vista seam cannot extend the
        // physical-grass budget beyond the terrain handoff.
        let vista = grass_lod_visibility(GrassMeshLod::Vista);
        assert_eq!(vista.end_margin, 42.0..50.0);
        assert_eq!(vista.end_margin.start, TERMINAL_SWARD_FADE_START_METRES);
        assert_eq!(vista.end_margin.end, TERMINAL_SWARD_FADE_END_METRES);
    }

    #[test]
    fn vista_rock_lod_is_a_bounded_twelve_face_silhouette() {
        let mesh = vista_rock_mesh();
        assert_eq!(mesh.count_vertices(), 12 * 3);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .expect("vista rock positions");
        assert!(
            positions
                .iter()
                .all(|position| Vec3::from_array(*position).length() < 1.0)
        );
    }

    #[test]
    fn vista_lods_build_independent_overlapping_rings() {
        let input = TacticalSceneInput::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../assets/tactical-scenes/valley-distant-ridge.json"),
        )
        .unwrap();
        let mut inner = Vec2::splat(55.0);
        for (index, lod) in input.vista.lods.iter().enumerate() {
            let meshes = vista_lod_meshes(lod, inner);
            assert!(!meshes.is_empty());
            assert!(meshes.iter().all(|mesh| mesh.count_vertices() > 0));
            assert!(meshes.iter().all(|mesh| {
                mesh.count_vertices() <= VISTA_CHUNK_CELLS * VISTA_CHUNK_CELLS * 4 * 4
            }));
            if index > 0 {
                assert!(
                    meshes.len() > 1,
                    "regional LODs must be independently culled"
                );
            }
            inner = Vec2::new(
                f32::from(lod.width - 1) * lod.spacing_metres * 0.5,
                f32::from(lod.depth - 1) * lod.spacing_metres * 0.5,
            );
        }
    }

    #[test]
    fn coarse_vista_cells_are_clipped_to_the_playable_hole() {
        let lod = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 250.0,
            width: 9,
            depth: 9,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![8.0; 81],
            environment: vec![EnvironmentalSample::default(); 81],
        };
        let inner_half_extent = Vec2::new(55.0, 42.0);
        let meshes = vista_lod_meshes(&lod, inner_half_extent);
        let mut touches_boundary = false;
        for mesh in meshes {
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("vista mesh must expose Float32x3 positions");
            };
            for quad in positions.as_chunks::<4>().0 {
                let outside = quad
                    .iter()
                    .all(|position| position[0] <= -inner_half_extent.x)
                    || quad
                        .iter()
                        .all(|position| position[0] >= inner_half_extent.x)
                    || quad
                        .iter()
                        .all(|position| position[2] <= -inner_half_extent.y)
                    || quad
                        .iter()
                        .all(|position| position[2] >= inner_half_extent.y);
                assert!(
                    outside,
                    "vista quad overlaps the playable terrain: {quad:?}"
                );
                touches_boundary |= quad.iter().any(|position| {
                    (position[0].abs() - inner_half_extent.x).abs() < 0.001
                        || (position[2].abs() - inner_half_extent.y).abs() < 0.001
                });
            }
        }
        assert!(
            touches_boundary,
            "coarse cells must be split at the exact playable boundary"
        );
    }

    #[test]
    fn first_vista_ring_stitches_to_playable_height_then_blends_outward() {
        let terrain = SceneTerrain::from_heightmap(
            3,
            3,
            50.0,
            vec![12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0],
        )
        .unwrap();
        let half_extent = Vec2::splat(50.0);

        assert_eq!(
            stitch_vista_height_to_playable_edge(
                &terrain,
                Vec2::new(50.0, 10.0),
                half_extent,
                250.0,
                112.0,
            ),
            12.0
        );
        assert_eq!(
            stitch_vista_height_to_playable_edge(
                &terrain,
                Vec2::new(175.0, 10.0),
                half_extent,
                250.0,
                112.0,
            ),
            62.0
        );
        assert_eq!(
            stitch_vista_height_to_playable_edge(
                &terrain,
                Vec2::new(300.0, 10.0),
                half_extent,
                250.0,
                112.0,
            ),
            112.0
        );
    }

    #[test]
    fn first_vista_ring_stitches_substrate_color_without_changing_sward_coverage() {
        let playable = Vec4::new(0.08, 0.12, 0.04, 1.0);
        let vista = Vec4::new(0.24, 0.31, 0.14, 0.37);
        let half_extent = Vec2::splat(50.0);

        let boundary = stitch_vista_color_to_playable_edge(
            Vec2::new(50.0, 10.0),
            half_extent,
            250.0,
            vista,
            playable,
        );
        let midpoint = stitch_vista_color_to_playable_edge(
            Vec2::new(175.0, 10.0),
            half_extent,
            250.0,
            vista,
            playable,
        );
        let outside = stitch_vista_color_to_playable_edge(
            Vec2::new(300.0, 10.0),
            half_extent,
            250.0,
            vista,
            playable,
        );

        assert_eq!(boundary.truncate(), playable.truncate());
        assert_eq!(midpoint.truncate(), playable.lerp(vista, 0.5).truncate());
        assert_eq!(outside.truncate(), vista.truncate());
        assert_eq!(boundary.w, vista.w);
        assert_eq!(midpoint.w, vista.w);
        assert_eq!(outside.w, vista.w);
    }

    #[test]
    fn first_vista_ring_reuses_every_playable_boundary_sample() {
        let heights = (0..5)
            .flat_map(|z| (0..5).map(move |_| z as f32 * 7.0))
            .collect::<Vec<_>>();
        let terrain = SceneTerrain::from_heightmap(5, 5, 25.0, heights).unwrap();
        let lod = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 250.0,
            width: 3,
            depth: 3,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![100.0; 9],
            environment: vec![EnvironmentalSample::default(); 9],
        };
        let half_extent = Vec2::splat(50.0);
        let meshes = vista_lod_meshes_with_morph(
            &lod,
            half_extent,
            None,
            Some(&terrain),
            None,
            clear_vista_weather(),
            None,
        );
        let mut east_edge = Vec::new();
        for mesh in meshes {
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("vista mesh must expose Float32x3 positions");
            };
            east_edge.extend(
                positions
                    .iter()
                    .copied()
                    .filter(|position| (position[0] - half_extent.x).abs() < 0.001),
            );
        }

        for sample in 0..terrain.grid_depth() {
            let z = -half_extent.y + sample as f32 * terrain.grid_scale();
            let expected_height = terrain.height_at(Vec2::new(half_extent.x, z)).unwrap();
            assert!(
                east_edge.iter().any(|position| {
                    (position[2] - z).abs() < 0.001 && (position[1] - expected_height).abs() < 0.001
                }),
                "vista edge omitted playable boundary sample z={z}, height={expected_height}"
            );
        }
    }

    #[test]
    fn finer_ring_morphs_onto_the_coarse_surface_at_its_outer_boundary() {
        let sample = EnvironmentalSample::default();
        let finer = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 10.0,
            width: 5,
            depth: 5,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![12.0; 25],
            environment: vec![sample; 25],
        };
        let coarse = VistaLod {
            level: VistaLevelIndex::new(1),
            spacing_metres: 20.0,
            width: 5,
            depth: 5,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![38.0; 25],
            environment: vec![sample; 25],
        };
        assert_eq!(
            presented_height(&finer, 4, 2, Vec2::new(20.0, 0.0), Some(&coarse)),
            38.0
        );
        assert_eq!(
            presented_height(&finer, 2, 2, Vec2::ZERO, Some(&coarse)),
            12.0
        );
        assert_eq!(
            presented_height(&finer, 3, 2, Vec2::new(15.0, 0.0), Some(&coarse)),
            31.5
        );
    }

    #[test]
    fn vista_vertex_colors_reuse_ground_palette_in_linear_space() {
        let open = EnvironmentalSample::default();
        let expected = scene_ground_color(&SceneEnvironment {
            scene_digest: String::new(),
            generation_version: TACTICAL_SCENE_GENERATION_VERSION,
            latitude_microdegrees:
                adventuresim_world_schema::coordinates::LatitudeMicrodegrees::new(53_500_000)
                    .unwrap(),
            longitude_microdegrees:
                adventuresim_world_schema::coordinates::LongitudeMicrodegrees::new(10_000_000)
                    .unwrap(),
            absolute_minute: adventuresim_world_schema::calendar::StrategicMinute::new(12 * 60),
            lunar_phase_minute: adventuresim_world_schema::calendar::StrategicMinute::new(12 * 60),
            absolute_elevation_metres: adventuresim_world_schema::ElevationMeters::new(20).unwrap(),
            weather: clear_vista_weather(),
            canopy_bps: 0,
            wetland_bps: 0,
            cultivation_bps: 0,
            water_bps: 0,
            hilly_bps: 0,
        })
        .to_linear()
        .to_f32_array();
        assert_eq!(
            vista_sample_color(open, clear_vista_weather()).to_array(),
            expected
        );

        let lod = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 10.0,
            width: 3,
            depth: 3,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 9],
            environment: vec![open; 9],
        };
        assert!(
            vista_lod_meshes(&lod, Vec2::ZERO)
                .iter()
                .all(|mesh| mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some())
        );
    }

    #[test]
    fn distant_sward_respects_surface_and_land_cover() {
        let open = EnvironmentalSample::default();
        let deep_woods = EnvironmentalSample {
            surface: TacticalSurface::DeepWoods,
            ..open
        };
        let mountain = EnvironmentalSample {
            hilly_bps: 10_000,
            ..open
        };
        let road = EnvironmentalSample {
            surface: TacticalSurface::Road,
            ..open
        };
        assert_eq!(vista_sward_coverage(open), 1.0);
        assert!(vista_sward_coverage(deep_woods) < 0.3);
        assert!(vista_sward_coverage(mountain) < 0.2);
        assert_eq!(vista_sward_coverage(road), 0.0);
    }

    #[test]
    fn grass_coverage_stitches_continuously_across_the_playable_boundary() {
        let ground = SceneGround::from_samples(
            3,
            3,
            10.0,
            vec![
                GroundSurface {
                    cover: GroundCover::TallGrass,
                    cover_density_bps: 10_000,
                    cover_height_cm: 82,
                    ..default()
                };
                9
            ],
        )
        .unwrap();
        let deep_woods = EnvironmentalSample {
            surface: TacticalSurface::DeepWoods,
            ..default()
        };
        let lod = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 10.0,
            width: 7,
            depth: 7,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 49],
            environment: vec![deep_woods; 49],
        };
        let coverage = |x| {
            stitched_vista_topology_coverage(
                &lod,
                Vec2::splat(10.0),
                &ground,
                Vec2::new(x, 0.0),
                &UrbanGround::new(&[], &[]),
            )
        };
        let boundary = coverage(10.0);
        let just_outside = coverage(10.5);
        let midpoint = coverage(16.0);
        let vista = coverage(22.0);
        assert!(boundary > 0.99);
        assert!((boundary - just_outside).abs() < 0.02);
        assert!(just_outside > midpoint && midpoint > vista);
        assert!((vista - vista_sward_coverage(deep_woods)).abs() < 0.01);

        let street = [CityStreetPatch::Corridor {
            start_metres: Vec2::new(12.0, 0.0),
            end_metres: Vec2::new(28.0, 0.0),
            half_width_metres: 2.0,
            surface: CityStreetSurface::CompactedEarth,
        }];
        assert_eq!(
            stitched_vista_topology_coverage(
                &lod,
                Vec2::splat(10.0),
                &ground,
                Vec2::new(22.0, 0.0),
                &UrbanGround::new(&street, &[]),
            ),
            0.0
        );
    }

    #[test]
    fn snow_palette_carries_into_vista_and_suppresses_sward() {
        let open = EnvironmentalSample::default();
        let clear = vista_sample_color(open, clear_vista_weather());
        let snow = vista_sample_color(
            open,
            WeatherSnapshot {
                snow_cover_bps: 10_000,
                precipitation: Precipitation::Snow,
                ..clear_vista_weather()
            },
        );
        assert!(snow.x > clear.x && snow.y > clear.y && snow.z > clear.z);
        assert!(snow.w < 0.1);
    }

    #[test]
    fn vista_tree_density_scales_with_physical_cell_area() {
        let small = (0..64_u64)
            .map(|seed| {
                vista_tree_candidate_count(
                    1.0,
                    50.0,
                    streams::TEST_COUNT.seed(seed.into(), &[]).to_u64(),
                )
            })
            .sum::<usize>();
        let large = (0..64_u64)
            .map(|seed| {
                vista_tree_candidate_count(
                    1.0,
                    100.0,
                    streams::TEST_COUNT.seed(seed.into(), &[]).to_u64(),
                )
            })
            .sum::<usize>();
        assert!(small > 0);
        assert!(large >= small * 3);
        assert_eq!(vista_tree_candidate_count(0.0, 250.0, 0), 0);
    }

    #[test]
    fn vista_tree_cards_enter_only_at_background_angular_size() {
        let card_height = 10.8;
        for spacing in [50.0, 250.0, 1_000.0] {
            for variation in [0.0, 0.5, 1.0] {
                let scale = vista_tree_scale(spacing, variation);
                let range = vista_tree_visibility(spacing, card_height, scale);
                let angular_height =
                    2.0 * ((card_height * scale * 0.5) / range.start_margin.start).atan();
                assert!(
                    angular_height <= VISTA_TREE_MAX_ANGULAR_HEIGHT_RADIANS + 0.0001,
                    "spacing={spacing} scale={scale} angle={}",
                    angular_height.to_degrees()
                );
                assert!(range.start_margin.start > 0.0);
                assert!(range.start_margin.end > range.start_margin.start);
                assert!(range.start_margin.end < range.end_margin.start);
            }
        }
        assert!((vista_tree_scale(250.0, 1.0) - 1.25).abs() < 0.0001);
        assert!((vista_tree_scale(1_000.0, 1.0) - 1.5625).abs() < 0.0001);
    }

    #[test]
    fn finer_color_morph_matches_coarse_color_at_outer_boundary() {
        let forest = EnvironmentalSample {
            canopy_bps: 8_000,
            ..default()
        };
        let cultivated = EnvironmentalSample {
            cultivation_bps: 8_000,
            ..default()
        };
        let finer = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 10.0,
            width: 5,
            depth: 5,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 25],
            environment: vec![forest; 25],
        };
        let coarse = VistaLod {
            level: VistaLevelIndex::new(1),
            spacing_metres: 20.0,
            width: 5,
            depth: 5,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: vec![0.0; 25],
            environment: vec![cultivated; 25],
        };
        assert_eq!(
            presented_color(
                &finer,
                4,
                2,
                Vec2::new(20.0, 0.0),
                Some(&coarse),
                clear_vista_weather(),
            ),
            vista_sample_color(cultivated, clear_vista_weather()).to_array()
        );
        assert_eq!(
            presented_color(
                &finer,
                2,
                2,
                Vec2::ZERO,
                Some(&coarse),
                clear_vista_weather(),
            ),
            vista_sample_color(forest, clear_vista_weather()).to_array()
        );
    }
}

#[cfg(test)]
mod furniture_support_tests {
    use super::*;
    #[test]
    fn furniture_support_matches_presented_triangles_at_seams_and_morphs() {
        let terrain = SceneTerrain::from_heightmap(5, 5, 2.0, vec![3.0; 25]).unwrap();
        let lod = VistaLod {
            level: VistaLevelIndex::new(0),
            spacing_metres: 5.0,
            width: 9,
            depth: 9,
            origin_east_metres: 0.0,
            origin_north_metres: 0.0,
            heights_metres: (0..81).map(|i| ((i * 17) % 13) as f32).collect(),
            environment: vec![EnvironmentalSample::default(); 81],
        };
        let coarser = VistaLod {
            level: VistaLevelIndex::new(1),
            spacing_metres: 10.0,
            heights_metres: vec![7.0; 81],
            ..lod.clone()
        };
        let meshes = vista_lod_meshes_with_morph(
            &lod,
            Vec2::splat(4.0),
            Some(&coarser),
            Some(&terrain),
            None,
            clear_vista_weather(),
            None,
        );
        let mut checked = 0;
        for mesh in meshes {
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let indices = mesh.indices().unwrap().iter().collect::<Vec<_>>();
            for index in indices.as_chunks::<3>().0 {
                let t = index.map(|i| Vec3::from_array(positions[i]));
                if (t[1] - t[0]).cross(t[2] - t[0]).y <= 0.0 {
                    continue;
                }
                let p = t[0] * 0.2 + t[1] * 0.3 + t[2] * 0.5;
                if p.x.abs() >= 20.0 || p.z.abs() >= 20.0 {
                    continue;
                }
                let actual = adventuresim_tactical_core::vista_surface::vista_triangle_height(
                    &lod,
                    Some(&coarser),
                    &terrain,
                    p.xz(),
                )
                .unwrap();
                assert!((actual - p.y).abs() < 0.0001, "{p:?}: {actual}");
                checked += 1;
            }
        }
        assert!(checked > 100);
    }
}
