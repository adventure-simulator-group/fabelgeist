//! The scene: the game's terrain heightfield with the Fabelgeist forest
//! floor textures, one sun with two cascades, an orbiting (or free) camera,
//! N trees (Fabelgeist oak glTF or a procedural placeholder), N characters.
//! Grass lives in `grass/`.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::VisibilityRange;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, light_consts::lux};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::settings::{BenchSettings, LodMode, TreeModel};
use crate::shading::SunSky;
use crate::sky::bake_sky_image;

pub const TERRAIN_SIZE: f32 = 704.0;
pub const TERRAIN_SAMPLES: usize = 129;
const TERRAIN_MAX_HEIGHT: f32 = 38.0;
/// Ground texture repeat, metres (Fabelgeist's forest litter tile).
pub const GROUND_TILE_M: f32 = 4.0;
/// Trees, grass and characters live within this radius of the origin.
pub const PATCH_RADIUS: f32 = 60.0;
const SUN_ILLUMINANCE: f32 = lux::DIRECT_SUNLIGHT;

/// The game's terrain (core.rs): sum-of-sines hills fading to a flat plain.
pub fn terrain_height(x: f32, z: f32) -> f32 {
    let s = (x * 0.012).sin() * (z * 0.014).cos()
        + 0.6 * (x * 0.031 + 1.7).sin() * (z * 0.026 + 0.4).cos()
        + 0.3 * (x * 0.061 + 4.2).sin() * (z * 0.055 + 2.1).sin();
    let normalized = (s / 1.9) * 0.5 + 0.5;
    let range = (x * x + z * z).sqrt();
    let edge = 1.0 - ((range - 260.0) / 60.0).clamp(0.0, 1.0);
    normalized * TERRAIN_MAX_HEIGHT * edge * edge
}

pub fn terrain_normal(x: f32, z: f32) -> Vec3 {
    let e = 1.0;
    Vec3::new(
        terrain_height(x - e, z) - terrain_height(x + e, z),
        2.0 * e,
        terrain_height(x, z - e) - terrain_height(x, z + e),
    )
    .normalize()
}

/// Deterministic 0..1 hash (splitmix64), for placement.
pub fn hash01(seed: u64, salt: u64) -> f32 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(salt.wrapping_mul(0xD1B5_4A32_D192_ED03))
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct TreeRoot;

#[derive(Component)]
pub struct CharacterRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TreeLod {
    Lod0,
    Lod1,
}

#[derive(Component, Clone)]
pub struct BenchCamera {
    orbit_angle: f32,
    yaw: f32,
    pitch: f32,
}

/// The bench camera minus the anti-aliasing / prepass components, which
/// `settings::apply_camera_settings` adds. Spawned fresh on every AA change:
/// bevy 0.19 keeps stale prepass state (the background motion-vector
/// pipeline specialized under TAA) on a retained render view after the
/// `MotionVectorPrepass` component is removed, which then fails validation
/// once the depth pre-pass runs at another sample count.
pub fn camera_bundle(state: BenchCamera, transform: Transform) -> impl Bundle {
    (
        Name::new("camera"),
        // bevy_egui keeps its primary context on a camera entity; the camera is
        // respawned on AA changes, so it carries the context explicitly and
        // auto-creation is off (main.rs).
        bevy_egui::PrimaryEguiContext,
        Camera3d::default(),
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection {
            fov: 80f32.to_radians(),
            far: 2000.0,
            ..default()
        }),
        transform,
        Exposure::SUNLIGHT,
        Tonemapping::TonyMcMapface,
        AmbientLight {
            color: Color::srgb(0.75, 0.85, 1.0),
            brightness: AMBIENT_BRIGHTNESS,
            ..default()
        },
        state,
    )
}

/// Meshes and materials of the procedural stand-in tree, built once.
#[derive(Resource)]
struct PlaceholderTree {
    trunk: Handle<Mesh>,
    crown_lod0: Handle<Mesh>,
    crown_lod1: Handle<Mesh>,
    bark: Handle<StandardMaterial>,
    leaves: Handle<StandardMaterial>,
}

/// Where the sun stands for a given elevation and compass angle, at the
/// distance the cascade config was built around.
fn sun_position(elevation_deg: f32, azimuth_deg: f32) -> Vec3 {
    let (elevation, azimuth) = (elevation_deg.to_radians(), azimuth_deg.to_radians());
    Vec3::new(
        azimuth.sin() * elevation.cos(),
        elevation.sin().max(0.02),
        azimuth.cos() * elevation.cos(),
    ) * 76.0
}

/// The sun knobs. Backlit foliage only shows with the sun low and behind the
/// grass, so the fixed sun had no angle that could show it. The instanced
/// grass reads bevy's light bindings and follows on its own; the custom
/// materials carry their own copy of the direction, so it is pushed to them.
fn move_sun(
    settings: Res<BenchSettings>,
    mut sun_sky: Option<ResMut<SunSky>>,
    mut materials: ResMut<Assets<crate::custom_material::CustomMaterial>>,
    mut sun: Query<&mut Transform, With<DirectionalLight>>,
    mut last: Local<Option<(f32, f32)>>,
) {
    let angles = (settings.sun_elevation, settings.sun_azimuth);
    if *last == Some(angles) {
        return;
    }
    *last = Some(angles);
    let position = sun_position(angles.0, angles.1);
    for mut transform in &mut sun {
        *transform = Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y);
    }
    let direction = position.normalize();
    if let Some(sun_sky) = sun_sky.as_mut() {
        sun_sky.sun_dir = direction;
    }
    for (_, material) in materials.iter_mut() {
        material.globals.sun_dir = direction.extend(0.0);
    }
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DirectionalLightShadowMap { size: 2048 })
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    spawn_trees,
                    spawn_characters,
                    tag_tree_meshes,
                    apply_lod,
                    drive_camera,
                    move_sun,
                ),
            );
    }
}

fn setup(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut standards: ResMut<Assets<StandardMaterial>>,
) {
    // Sun: the tactical client's shape (two cascades, 72 m) at direct-sunlight
    // illuminance with the matching exposure.
    let sun_transform = Transform::from_translation(sun_position(
        settings.sun_elevation,
        settings.sun_azimuth,
    ))
    .looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Name::new("sun"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.96, 0.9),
            illuminance: SUN_ILLUMINANCE,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 18.0,
            maximum_distance: 72.0,
            ..default()
        }
        .build(),
        sun_transform,
    ));

    // The non-PBR materials shade in exposure-scaled units: sun and sky are
    // scaled by the same exposure the PBR path applies.
    let exposure = Exposure::SUNLIGHT;
    let scale = exposure.exposure();
    let sky_cube = images.add(bake_sky_image(Vec3::new(0.30, 0.26, 0.16)));
    commands.insert_resource(SunSky {
        sun_dir: sun_transform.translation.normalize(),
        sun_color: Vec4::from_array(
            Color::srgb(1.0, 0.96, 0.9)
                .to_linear()
                .to_f32_array(),
        ) * SUN_ILLUMINANCE
            * scale,
        // Sky irradiance (~0.5 average) against ambient of the same size.
        sky_strength: AMBIENT_BRIGHTNESS * scale,
        sky_cube,
    });

    let look_at = Vec3::new(0.0, terrain_height(0.0, 0.0) + 1.6, 0.0);
    commands.spawn(camera_bundle(
        BenchCamera {
            orbit_angle: 0.0,
            yaw: 0.0,
            pitch: -0.2,
        },
        Transform::from_translation(look_at + Vec3::new(0.0, 6.0, 24.0)).looking_at(look_at, Vec3::Y),
    ));

    // Ground: the Fabelgeist forest-floor set, tiled every 4 m.
    let texture = |channel: &str, is_srgb: bool| {
        asset_server
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                settings.is_srgb = is_srgb;
                settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                });
            })
            .load(format!("textures/ground/{channel}.png"))
    };
    let arm = texture("arm", false);
    let ground = standards.add(StandardMaterial {
        base_color_texture: Some(texture("albedo", true)),
        normal_map_texture: Some(texture("normal", false)),
        metallic_roughness_texture: Some(arm.clone()),
        occlusion_texture: Some(arm),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        Name::new("ground"),
        Ground,
        Mesh3d(meshes.add(terrain_mesh())),
        MeshMaterial3d(ground),
    ));

    let placeholder = build_placeholder_tree(&mut meshes, &mut images, &mut standards);
    commands.insert_resource(placeholder);
}

/// Ambient (cd/m²) that lands near the sky bake's ~0.5 irradiance after
/// sunlight exposure, so PBR and the flat-shaded paths read alike.
const AMBIENT_BRIGHTNESS: f32 = 18_000.0;

/// Vertices on the collider's sample grid, UVs in ground-tile units.
fn terrain_mesh() -> Mesh {
    let n = TERRAIN_SAMPLES;
    let half = TERRAIN_SIZE / 2.0;
    let step = TERRAIN_SIZE / (n - 1) as f32;
    let mut positions = Vec::with_capacity(n * n);
    let mut normals = Vec::with_capacity(n * n);
    let mut uvs = Vec::with_capacity(n * n);
    for xi in 0..n {
        let x = -half + xi as f32 * step;
        for zi in 0..n {
            let z = -half + zi as f32 * step;
            positions.push([x, terrain_height(x, z), z]);
            normals.push(terrain_normal(x, z).to_array());
            uvs.push([x / GROUND_TILE_M, z / GROUND_TILE_M]);
        }
    }
    let mut indices = Vec::with_capacity((n - 1) * (n - 1) * 6);
    for xi in 0..n - 1 {
        for zi in 0..n - 1 {
            let a = (xi * n + zi) as u32;
            let b = ((xi + 1) * n + zi) as u32;
            let c = (xi * n + zi + 1) as u32;
            let d = ((xi + 1) * n + zi + 1) as u32;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices));
    if let Err(err) = mesh.generate_tangents() {
        warn!("terrain tangents: {err:?}");
    }
    mesh
}

/// A trunk cylinder and a crown of alpha-tested leaf cards: 48 cards for
/// LOD0, 8 for LOD1, one procedural leaf-cluster texture.
fn build_placeholder_tree(
    meshes: &mut Assets<Mesh>,
    images: &mut Assets<Image>,
    standards: &mut Assets<StandardMaterial>,
) -> PlaceholderTree {
    let size = 256u32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    for k in 0..44u64 {
        let cx = hash01(k, 1) * size as f32;
        let cy = hash01(k, 2) * size as f32;
        let r = 14.0 + hash01(k, 3) * 22.0;
        let tint = hash01(k, 4);
        let rgb = [
            (40.0 + tint * 60.0) as u8,
            (110.0 + tint * 90.0) as u8,
            (30.0 + tint * 30.0) as u8,
        ];
        for y in 0..size {
            for x in 0..size {
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                if d < r {
                    let i = ((y * size + x) * 4) as usize;
                    let a = ((r - d) / 2.0).clamp(0.0, 1.0);
                    if a >= data[i + 3] as f32 / 255.0 {
                        data[i..i + 3].copy_from_slice(&rgb);
                        data[i + 3] = (a * 255.0) as u8;
                    }
                }
            }
        }
    }
    let leaf_texture = images.add(Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));

    let crown = |cards: usize, card_size: f32, salt: u64| -> Mesh {
        let mut positions = Vec::with_capacity(cards * 4);
        let mut normals = Vec::with_capacity(cards * 4);
        let mut uvs = Vec::with_capacity(cards * 4);
        let mut indices = Vec::with_capacity(cards * 6);
        for i in 0..cards as u64 {
            let s = i + salt * 1000;
            let dir = Vec3::new(hash01(s, 10) - 0.5, hash01(s, 11) - 0.5, hash01(s, 12) - 0.5).normalize_or(Vec3::Y);
            let center = Vec3::new(0.0, 6.8, 0.0) + dir * 2.6 * hash01(s, 13).cbrt();
            let n = Vec3::new(hash01(s, 14) - 0.5, hash01(s, 15) - 0.5, hash01(s, 16) - 0.5).normalize_or(Vec3::Z);
            let t = n.cross(if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X }).normalize();
            let b = n.cross(t);
            let h = card_size * 0.5;
            let base = positions.len() as u32;
            for (u, v) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                positions.push((center + t * (u * h) + b * (v * h)).to_array());
                normals.push(n.to_array());
                uvs.push([u * 0.5 + 0.5, v * 0.5 + 0.5]);
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices))
    };

    PlaceholderTree {
        trunk: meshes.add(Cylinder::new(0.3, 7.0)),
        crown_lod0: meshes.add(crown(48, 2.4, 1)),
        crown_lod1: meshes.add(crown(8, 5.0, 2)),
        bark: standards.add(StandardMaterial {
            base_color: Color::srgb(0.38, 0.27, 0.17),
            perceptual_roughness: 0.95,
            ..default()
        }),
        leaves: standards.add(StandardMaterial {
            base_color_texture: Some(leaf_texture),
            alpha_mode: AlphaMode::AlphaToCoverage,
            double_sided: true,
            cull_mode: None,
            perceptual_roughness: 0.8,
            ..default()
        }),
    }
}

/// Golden-angle spiral over the patch, jittered, clear of the camera orbit.
fn scatter_position(i: u32, count: u32, min_radius: f32, max_radius: f32, salt: u64) -> Vec3 {
    let t = (i as f32 + 0.5) / count.max(1) as f32;
    let r = min_radius + t.sqrt() * (max_radius - min_radius);
    let angle = i as f32 * 2.399_963 + (hash01(i as u64, salt) - 0.5) * 0.8;
    let x = r * angle.cos();
    let z = r * angle.sin();
    Vec3::new(x, terrain_height(x, z), z)
}

fn spawn_trees(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    asset_server: Res<AssetServer>,
    placeholder: Res<PlaceholderTree>,
    existing: Query<Entity, With<TreeRoot>>,
    mut spawned: Local<Option<(u32, TreeModel)>>,
) {
    let wanted = (settings.tree_count, settings.tree_model);
    if *spawned == Some(wanted) {
        return;
    }
    *spawned = Some(wanted);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for i in 0..settings.tree_count {
        let position = scatter_position(i, settings.tree_count, 9.0, PATCH_RADIUS * 0.85, 7);
        let yaw = hash01(i as u64, 8) * std::f32::consts::TAU;
        let scale = 0.85 + hash01(i as u64, 9) * 0.35;
        let transform = Transform::from_translation(position)
            .with_rotation(Quat::from_rotation_y(yaw))
            .with_scale(Vec3::splat(scale));
        let mut root = commands.spawn((Name::new(format!("tree {i}")), TreeRoot, transform, Visibility::default()));
        match settings.tree_model {
            TreeModel::Oak => {
                root.with_child((
                    Name::new("oak"),
                    WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/oak.glb"))),
                ));
            }
            TreeModel::Placeholder => {
                root.with_children(|parent| {
                    parent.spawn((
                        Name::new("placeholder_lod0_trunk"),
                        Mesh3d(placeholder.trunk.clone()),
                        MeshMaterial3d(placeholder.bark.clone()),
                        Transform::from_xyz(0.0, 3.5, 0.0),
                    ));
                    parent.spawn((
                        Name::new("placeholder_lod0_leaves"),
                        Mesh3d(placeholder.crown_lod0.clone()),
                        MeshMaterial3d(placeholder.leaves.clone()),
                    ));
                    parent.spawn((
                        Name::new("placeholder_lod1_trunk"),
                        Mesh3d(placeholder.trunk.clone()),
                        MeshMaterial3d(placeholder.bark.clone()),
                        Transform::from_xyz(0.0, 3.5, 0.0),
                    ));
                    parent.spawn((
                        Name::new("placeholder_lod1_leaves"),
                        Mesh3d(placeholder.crown_lod1.clone()),
                        MeshMaterial3d(placeholder.leaves.clone()),
                    ));
                });
            }
        }
    }
}

fn spawn_characters(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<CharacterRoot>>,
    mut spawned: Local<Option<u32>>,
) {
    if *spawned == Some(settings.character_count) {
        return;
    }
    *spawned = Some(settings.character_count);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for i in 0..settings.character_count {
        let position = scatter_position(i, settings.character_count, 3.0, 12.0, 21);
        let yaw = hash01(i as u64, 22) * std::f32::consts::TAU;
        commands.spawn((
            Name::new(format!("character {i}")),
            CharacterRoot,
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::default(),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/fabelgeist.glb"))),
        ));
    }
}

/// Every new mesh under a tree root gets its LOD tier from its node name
/// (`*lod0*` / `*lod1*`; unnamed or unmatched = LOD0).
fn tag_tree_meshes(
    mut commands: Commands,
    new_meshes: Query<(Entity, Option<&Name>), Added<Mesh3d>>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    roots: Query<(), With<TreeRoot>>,
) {
    fn lod_from_name(name: Option<&Name>) -> Option<TreeLod> {
        let n = name?.as_str().to_ascii_lowercase();
        if n.contains("lod1") {
            Some(TreeLod::Lod1)
        } else if n.contains("lod0") {
            Some(TreeLod::Lod0)
        } else {
            None
        }
    }
    for (entity, name) in &new_meshes {
        let mut current = entity;
        let mut under_tree = false;
        let mut lod = lod_from_name(name);
        while let Ok(child_of) = parents.get(current) {
            current = child_of.parent();
            if roots.get(current).is_ok() {
                under_tree = true;
                break;
            }
            if lod.is_none() {
                lod = lod_from_name(names.get(current).ok());
            }
        }
        if under_tree {
            commands.entity(entity).try_insert(lod.unwrap_or(TreeLod::Lod0));
        }
    }
}

fn lod_range(lod: TreeLod, mode: LodMode) -> VisibilityRange {
    let far = 900.0..1000.0;
    let switch = 22.0..26.0;
    let (start, end) = match (mode, lod) {
        (LodMode::Distance, TreeLod::Lod0) => (0.0..0.0, switch),
        (LodMode::Distance, TreeLod::Lod1) => (switch, far),
        (LodMode::Lod0, TreeLod::Lod0) | (LodMode::Lod1, TreeLod::Lod1) => (0.0..0.0, far),
        _ => (0.0..0.0, 0.0..0.0),
    };
    VisibilityRange {
        start_margin: start,
        end_margin: end,
        use_aabb: false,
    }
}

fn apply_lod(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    all: Query<(Entity, &TreeLod)>,
    added: Query<(Entity, &TreeLod), Added<TreeLod>>,
) {
    if settings.is_changed() {
        for (entity, lod) in &all {
            commands.entity(entity).try_insert(lod_range(*lod, settings.lod));
        }
    } else {
        for (entity, lod) in &added {
            commands.entity(entity).try_insert(lod_range(*lod, settings.lod));
        }
    }
}

fn drive_camera(
    time: Res<Time>,
    settings: Res<BenchSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut cameras: Query<(&mut Transform, &mut BenchCamera)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut cam) in &mut cameras {
        if settings.orbit {
            cam.orbit_angle += dt * std::f32::consts::TAU / 60.0;
            let center = Vec3::new(0.0, terrain_height(0.0, 0.0) + 1.6, 0.0);
            let radius = 24.0;
            let position = center + Vec3::new(cam.orbit_angle.cos() * radius, 6.0, cam.orbit_angle.sin() * radius);
            *transform = Transform::from_translation(position).looking_at(center, Vec3::Y);
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            cam.yaw = yaw;
            cam.pitch = pitch;
            continue;
        }
        if mouse.pressed(MouseButton::Right) {
            cam.yaw -= motion.delta.x * 0.003;
            cam.pitch = (cam.pitch - motion.delta.y * 0.003).clamp(-1.5, 1.5);
        }
        transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
        let mut wish = Vec3::ZERO;
        let forward = transform.forward().as_vec3();
        let right = transform.right().as_vec3();
        if keys.pressed(KeyCode::KeyW) {
            wish += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            wish -= forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            wish += right;
        }
        if keys.pressed(KeyCode::KeyA) {
            wish -= right;
        }
        if keys.pressed(KeyCode::KeyE) {
            wish += Vec3::Y;
        }
        if keys.pressed(KeyCode::KeyQ) {
            wish -= Vec3::Y;
        }
        let speed = if keys.pressed(KeyCode::ShiftLeft) { 30.0 } else { 8.0 };
        transform.translation += wish.normalize_or_zero() * speed * dt;
    }
}
