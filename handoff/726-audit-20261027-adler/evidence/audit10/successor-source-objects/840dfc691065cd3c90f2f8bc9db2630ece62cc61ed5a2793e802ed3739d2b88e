use super::terrain::TerrainContactHeightmap;
use super::*;
use adventuresim_tactical_core::scene::SceneElevationRange;
use occlusion::empty_weather_occlusion_image;
pub(super) use occlusion::update_weather_occlusion_map;

use fabelgeist_determinism::{Seed, StreamId};

mod occlusion;

const WEATHER_SHADER: &str = "shaders/tactical_weather.wgsl";
const FALLING_PARTICLE_CAPACITY: usize = 3_072;
const IMPACT_PARTICLE_CAPACITY: usize = 384;
const DISTANT_SHEET_SEGMENTS: usize = 16;
const DISTANT_SHEET_LAYERS: usize = 6;
const WEATHER_OCCLUSION_RESOLUTION: u32 = 512;

const ATTRIBUTE_WEATHER_PARTICLE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("WeatherParticleData", 2_180_101, VertexFormat::Float32x4);
const ATTRIBUTE_WEATHER_PARTICLE_CORNER: MeshVertexAttribute =
    MeshVertexAttribute::new("WeatherParticleCorner", 2_180_102, VertexFormat::Float32x2);

struct WeatherParticleBatch {
    name: &'static str,
    layer: WeatherParticle,
    mesh: Mesh,
    material: TacticalWeatherMaterial,
}

#[derive(Resource, Default)]
pub(in crate::presentation) struct WeatherOcclusionState {
    scene: Option<Entity>,
    tree_signature: Option<u64>,
    map: Option<WeatherOcclusionMap>,
}

struct WeatherOcclusionMap {
    image: Handle<Image>,
    range: SceneElevationRange,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(in crate::presentation) struct TacticalWeatherMaterial {
    /// Precipitation kind, intensity, wind strength, deterministic seed.
    #[uniform(0)]
    weather: Vec4,
    /// Horizontal wind direction, camera-local radius, fall volume height.
    #[uniform(1)]
    motion: Vec4,
    /// Playable half-width/depth and encoded minimum/maximum terrain height.
    #[uniform(2)]
    terrain: Vec4,
    /// Row-major playable heightfield encoded into two eight-bit channels.
    #[texture(3)]
    heightmap: Handle<Image>,
    /// Top-down woody shelter height, with alpha marking covered texels.
    #[texture(4)]
    occlusion_map: Handle<Image>,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WeatherParticle {
    Falling,
    Impact,
    DistantSheet,
}

impl Material for TacticalWeatherMaterial {
    fn vertex_shader() -> ShaderRef {
        WEATHER_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        WEATHER_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_WEATHER_PARTICLE_DATA.at_shader_location(1),
            ATTRIBUTE_WEATHER_PARTICLE_CORNER.at_shader_location(2),
        ])?];
        Ok(())
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects active weather scene data, presentation assets, and occlusion state independently"
)]
pub(super) fn apply_active_scene_weather(
    active: Res<ActiveTacticalScene>,
    scenes: Query<(
        &SceneEnvironment,
        Option<&SceneTerrain>,
        Option<&StrategicBackdrop>,
    )>,
    particles: Query<Entity, With<WeatherParticle>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TacticalWeatherMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut occlusion: ResMut<WeatherOcclusionState>,
) {
    if !active.is_changed() {
        return;
    }
    for entity in &particles {
        commands.entity(entity).despawn();
    }
    occlusion.scene = active.entity;
    occlusion.tree_signature = None;
    occlusion.map = None;
    let Some((environment, terrain, backdrop)) =
        active.entity.and_then(|entity| scenes.get(entity).ok())
    else {
        return;
    };
    let Some(terrain) = weather_surface(terrain, backdrop) else {
        return;
    };
    if environment.weather.precipitation == Precipitation::Clear
        || environment.weather.intensity_bps == 0
    {
        return;
    }

    let contact = match TerrainContactHeightmap::from_terrain(terrain) {
        Ok(contact) => contact,
        Err(error) => {
            warn!(%error, "Weather contact heightmap rejected");
            return;
        }
    };
    let range = contact.range();
    let heightmap = images.add(contact.into_image());
    let occlusion_map = images.add(empty_weather_occlusion_image());
    occlusion.map = Some(WeatherOcclusionMap {
        image: occlusion_map.clone(),
        range,
    });
    for batch in weather_particle_batches(environment, terrain, range, heightmap, occlusion_map) {
        let material = materials.add(batch.material);
        commands.spawn((
            Name::new(batch.name),
            batch.layer,
            NoFrustumCulling,
            NotShadowCaster,
            Mesh3d(meshes.add(batch.mesh)),
            MeshMaterial3d(material),
            Transform::default(),
        ));
    }
}

fn weather_particle_batches(
    environment: &SceneEnvironment,
    terrain: &SceneTerrain,
    range: SceneElevationRange,
    heightmap: Handle<Image>,
    occlusion_map: Handle<Image>,
) -> Vec<WeatherParticleBatch> {
    let mut batches = vec![WeatherParticleBatch {
        name: "GPU tactical falling precipitation",
        layer: WeatherParticle::Falling,
        mesh: weather_particle_mesh(FALLING_PARTICLE_CAPACITY),
        material: weather_material(
            environment,
            terrain,
            range,
            heightmap.clone(),
            occlusion_map.clone(),
            WeatherParticle::Falling,
        ),
    }];
    if environment.weather.precipitation == Precipitation::Rain {
        batches.push(WeatherParticleBatch {
            name: "GPU tactical rain impacts",
            layer: WeatherParticle::Impact,
            mesh: weather_particle_mesh(IMPACT_PARTICLE_CAPACITY),
            material: weather_material(
                environment,
                terrain,
                range,
                heightmap.clone(),
                occlusion_map.clone(),
                WeatherParticle::Impact,
            ),
        });
        if environment.weather.intensity_bps >= 8_000 {
            batches.push(WeatherParticleBatch {
                name: "GPU tactical distant rain sheets",
                layer: WeatherParticle::DistantSheet,
                mesh: weather_sheet_mesh(DISTANT_SHEET_SEGMENTS, DISTANT_SHEET_LAYERS),
                material: weather_material(
                    environment,
                    terrain,
                    range,
                    heightmap,
                    occlusion_map,
                    WeatherParticle::DistantSheet,
                ),
            });
        }
    }
    batches
}

// Exactly one scene source owns precipitation heights. A sampled strategic
// backdrop is never admitted as a canonical terrain component.
fn weather_surface<'a>(
    terrain: Option<&'a SceneTerrain>,
    backdrop: Option<&'a StrategicBackdrop>,
) -> Option<&'a SceneTerrain> {
    match (terrain, backdrop) {
        (Some(terrain), None) => Some(terrain),
        (None, Some(backdrop)) => Some(backdrop.weather_surface()),
        _ => None,
    }
}

fn weather_material(
    environment: &SceneEnvironment,
    terrain: &SceneTerrain,
    range: SceneElevationRange,
    heightmap: Handle<Image>,
    occlusion_map: Handle<Image>,
    layer: WeatherParticle,
) -> TacticalWeatherMaterial {
    let kind = match (environment.weather.precipitation, layer) {
        (Precipitation::Rain, WeatherParticle::Falling) => 1.0,
        (Precipitation::Snow, WeatherParticle::Falling) => 2.0,
        (Precipitation::Rain, WeatherParticle::Impact) => 3.0,
        (Precipitation::Rain, WeatherParticle::DistantSheet) => 4.0,
        _ => 0.0,
    };
    let bearing = f32::from(environment.weather.atmosphere.wind_direction_degrees).to_radians();
    let seed = StreamId::new("visual.weather.interval").seed(
        stable_text_seed(&environment.scene_digest),
        &[environment.weather.interval_start_minute.get()],
    );
    let seed = StreamId::new("visual.weather.shader-seed")
        .rng(seed, &[])
        .index(65_521) as f32;
    let radius = match layer {
        WeatherParticle::Impact => 18.0,
        WeatherParticle::Falling => 30.0,
        WeatherParticle::DistantSheet => 36.0,
    };
    TacticalWeatherMaterial {
        weather: Vec4::new(
            kind,
            bps(environment.weather.intensity_bps),
            bps(environment.weather.wind_speed_bps),
            seed,
        ),
        motion: Vec4::new(bearing.sin(), -bearing.cos(), radius, 24.0),
        terrain: Vec4::new(
            terrain.width() * 0.5,
            terrain.depth() * 0.5,
            range.minimum().metres(),
            range.maximum().metres(),
        ),
        heightmap,
        occlusion_map,
    }
}

fn weather_particle_mesh(capacity: usize) -> Mesh {
    let mut positions = Vec::with_capacity(capacity * 4);
    let mut particle_data = Vec::with_capacity(capacity * 4);
    let mut corners = Vec::with_capacity(capacity * 4);
    let mut indices = Vec::with_capacity(capacity * 6);
    const QUAD_CORNERS: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    for index in 0..capacity {
        let seed_a = StreamId::new("visual.weather.particle-primary")
            .rng(0.into(), &[index as u64])
            .inclusive_unit_f32();
        let seed_b = StreamId::new("visual.weather.particle-secondary")
            .rng(0.into(), &[index as u64])
            .inclusive_unit_f32();
        let rank = (index as f32 + 0.5) / capacity as f32;
        let base = positions.len() as u32;
        for corner in QUAD_CORNERS {
            positions.push([0.0, 0.0, 0.0]);
            particle_data.push([seed_a, seed_b, rank, index as f32]);
            corners.push(corner);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(ATTRIBUTE_WEATHER_PARTICLE_DATA, particle_data);
    mesh.insert_attribute(ATTRIBUTE_WEATHER_PARTICLE_CORNER, corners);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn weather_sheet_mesh(segments: usize, layers: usize) -> Mesh {
    let capacity = segments * layers;
    let mut positions = Vec::with_capacity(capacity * 4);
    let mut particle_data = Vec::with_capacity(capacity * 4);
    let mut corners = Vec::with_capacity(capacity * 4);
    let mut indices = Vec::with_capacity(capacity * 6);
    const QUAD_CORNERS: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    // Transparent primitives in a single draw are not depth-sorted for us.
    // Emit the distant shells first so their soft veils blend back-to-front.
    for layer in (0..layers).rev() {
        for segment in 0..segments {
            let angle = (segment as f32 + 0.5) / segments as f32;
            let layer_fraction = (layer as f32 + 0.5) / layers as f32;
            let rank = (segment as f32 + 0.5) / segments as f32;
            let seed = (layer * segments + segment) as f32;
            let base = positions.len() as u32;
            for corner in QUAD_CORNERS {
                positions.push([0.0, 0.0, 0.0]);
                particle_data.push([angle, layer_fraction, rank, seed]);
                corners.push(corner);
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(ATTRIBUTE_WEATHER_PARTICLE_DATA, particle_data);
    mesh.insert_attribute(ATTRIBUTE_WEATHER_PARTICLE_CORNER, corners);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_mesh_is_one_indexed_quad_batch() {
        let mesh = weather_particle_mesh(17);
        assert_eq!(mesh.count_vertices(), 17 * 4);
        assert_eq!(mesh.indices().unwrap().len(), 17 * 6);
        assert!(mesh.attribute(ATTRIBUTE_WEATHER_PARTICLE_DATA).is_some());
        assert!(mesh.attribute(ATTRIBUTE_WEATHER_PARTICLE_CORNER).is_some());
    }

    #[test]
    fn heightmap_encoding_preserves_terrain_range() {
        let terrain = SceneTerrain::from_heightmap(2, 2, 1.0, vec![-2.0, 0.0, 1.0, 4.0]).unwrap();
        let image = TerrainContactHeightmap::from_terrain(&terrain)
            .unwrap()
            .into_image();
        assert_eq!(image.texture_descriptor.size.width, 2);
        assert_eq!(image.texture_descriptor.size.height, 2);
        let pixels = image.data.as_deref().unwrap();
        let first = u16::from_le_bytes([pixels[0], pixels[1]]);
        let last = u16::from_le_bytes([pixels[12], pixels[13]]);
        assert_eq!(first, 0);
        assert_eq!(last, u16::MAX);
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/flat-dry-grassland.json"
        ))
        .unwrap();
        let backdrop = StrategicBackdrop::new(&input, input.vista.lods.len()).unwrap();
        assert_eq!(
            TerrainContactHeightmap::from_terrain(weather_surface(None, Some(&backdrop)).unwrap())
                .unwrap()
                .into_image()
                .data,
            TerrainContactHeightmap::from_terrain(backdrop.weather_surface())
                .unwrap()
                .into_image()
                .data
        );
        assert!(std::ptr::eq(
            weather_surface(Some(&terrain), None).unwrap(),
            &terrain
        ));
        assert!(weather_surface(Some(&terrain), Some(&backdrop)).is_none());
        assert!(weather_surface(None, None).is_none());
    }

    #[test]
    fn distant_sheet_mesh_batches_every_shell_panel() {
        let mesh = weather_sheet_mesh(8, 6);
        assert_eq!(mesh.count_vertices(), 8 * 6 * 4);
        assert_eq!(mesh.indices().unwrap().len(), 8 * 6 * 6);
    }
}
