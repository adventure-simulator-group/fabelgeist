//! The displacement map: a small top-down render target over the patch that
//! holds, per texel, how far the grass tip at that spot is pushed (metres,
//! xz) by every affector, plus the trails. Wind stays analytic per blade in
//! the vertex shader (it is per-blade motion TAA wants to see; putting it in
//! the map smeared it). One 2D camera on its own
//! render layer draws one quad with `shaders/displacement.wgsl` into an
//! `Rgba16Float` image before the main view renders (2D, not 3D: no light
//! clustering, no prepass machinery, and its passes carry 2D names so the
//! overlay's 3D pass timings stay the main view's); grass vertices
//! in the map mode (`InstancingMode::MeshChunksMap`) sample it once instead
//! of looping over the affectors. The count of affectors never reaches the
//! grass shader at all.
//!
//! Mapping: the quad's 2D x is world x and its 2D y is world -z (the
//! fragment flips it), so `u = (x - x0) / size`, `v = (z - z0) / size` with
//! `(x0, z0)` the map's min corner. `CustomGlobals::map` carries
//! `(x0, z0, 1 / size)`. `BENCH_MAP_DEBUG=1` exaggerates the affector pushes
//! so the mapping can be checked from afar.
//!
//! Trails: a second map (`shaders/trample.wgsl`) on layer 2 accumulates
//! where the grass has been walked over, and the displacement pass adds it
//! in. It is a ping-pong pair of images, not one blended-into target: each
//! frame one quad reads last frame's image, relaxes it toward standing
//! grass (`trample_recover`, 0 = never), takes the greater of that and this
//! frame's footprints, and writes the result to the other image with
//! blending off. Reading the previous map explicitly is what makes both
//! ends work: a trail lasts exactly as long as the regrow knob says, and
//! nothing else dilutes it. (A
//! camera target is not a safe accumulator: what a pass loads is the view's
//! pooled intermediate texture, which the map and trample views hand back
//! and forth, so blending into it lost the trail after a few frames.)
//! `trample_scorch` widens the flattened core of the path, presses it to
//! the ground and burns its colour (the displacement map's w channel
//! carries the trail to the grass fragment shader for that).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ClearColorConfig, Hdr, ImageRenderTarget, RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::render::render_resource::{
    AsBindGroup, Buffer, Extent3d, ShaderType, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::shader::ShaderRef;

use crate::custom_material::AffectorBuffer;
use crate::scene::PATCH_RADIUS;
use crate::settings::{BenchSettings, InstancingMode};

pub const DISPLACEMENT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3c1f5f4e-0a6b-4a0e-9a5f-1d2b4c6e8f04");
pub const TRAMPLE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3c1f5f4e-0a6b-4a0e-9a5f-1d2b4c6e8f05");

/// The map's render layer; the sun and the main camera stay on layer 0, so
/// this view builds no shadow cascades and the main view never sees the quad.
const MAP_LAYER: usize = 1;
/// The trample map's layer: its camera sees only the decay and push quads.
const TRAMPLE_LAYER: usize = 2;

/// World extent of the map (m), a square around the origin covering the patch.
pub const MAP_SIZE: f32 = (PATCH_RADIUS + 4.0) * 2.0;

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct DisplacementParams {
    /// x: affector radius scale, y: affector push scale (1, 1; the debug
    /// env var makes them 4, 3), z: trample map weight (0 off), w unused.
    pub debug: Vec4,
    /// Map min corner (x, z), 1 / size: the trample map's uv.
    pub map: Vec4,
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct DisplacementMaterial {
    #[uniform(0)]
    pub params: DisplacementParams,
    #[storage(1, read_only, buffer)]
    pub affectors: Buffer,
    #[texture(2)]
    #[sampler(3)]
    pub trample: Handle<Image>,
}

impl Material2d for DisplacementMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(DISPLACEMENT_SHADER_HANDLE)
    }
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct TrampleParams {
    /// x: affector radius scale, y: push scale (the debug exaggeration),
    /// z: trail strength (flattening and crush scale), w: scorch (0..1).
    pub params: Vec4,
    /// x: regrow seconds (0 = never), yzw unused.
    pub recover: Vec4,
}

/// The accumulating quad of the trample pass: last frame's trails in, this
/// frame's trails out. Opaque, so the fragment's value is written as is.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct TrampleMaterial {
    #[uniform(0)]
    pub params: TrampleParams,
    #[storage(1, read_only, buffer)]
    pub affectors: Buffer,
    /// The other image of the pair: what the map held last frame.
    #[texture(2)]
    pub previous: Handle<Image>,
}

impl Material2d for TrampleMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TRAMPLE_SHADER_HANDLE)
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Opaque
    }
}

/// The current map image and its resolution; the grass material binds it.
#[derive(Resource, Clone)]
pub struct DisplacementMap {
    pub image: Handle<Image>,
    /// The trail pair. `write` is the one this frame renders into (and the
    /// displacement pass reads, the trample camera running first); the
    /// other one is what the accumulating quad reads.
    pub trample: [Handle<Image>; 2],
    pub write: usize,
    pub resolution: u32,
    /// Min corner (x, z) and 1 / size, as the shader wants them.
    pub mapping: Vec4,
}

impl DisplacementMap {
    fn written(&self) -> Handle<Image> {
        self.trample[self.write].clone()
    }
    fn previous(&self) -> Handle<Image> {
        self.trample[self.write ^ 1].clone()
    }
}

#[derive(Component)]
struct DisplacementCamera;

#[derive(Component)]
struct TrampleCamera;

/// The material handles the knobs rewrite.
#[derive(Resource)]
struct MapMaterials {
    displacement: Handle<DisplacementMaterial>,
    trample: Handle<TrampleMaterial>,
}

pub struct DisplacementPlugin;

impl Plugin for DisplacementPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            DISPLACEMENT_SHADER_HANDLE,
            "shaders/displacement.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TRAMPLE_SHADER_HANDLE,
            "shaders/trample.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins((
            Material2dPlugin::<DisplacementMaterial>::default(),
            Material2dPlugin::<TrampleMaterial>::default(),
        ))
            .add_systems(Startup, setup)
            .add_systems(Update, (sync, ping_pong).chain());
    }
}

fn map_image(resolution: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8; 8],
        TextureFormat::Rgba16Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image.sampler = ImageSampler::linear();
    image
}

fn mapping() -> Vec4 {
    Vec4::new(-MAP_SIZE * 0.5, -MAP_SIZE * 0.5, 1.0 / MAP_SIZE, 0.0)
}

fn debug_scales() -> (f32, f32) {
    if std::env::var("BENCH_MAP_DEBUG").is_ok() {
        (4.0, 3.0)
    } else {
        (1.0, 1.0)
    }
}

fn setup(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DisplacementMaterial>>,
    mut tramples: ResMut<Assets<TrampleMaterial>>,
    affector_buffer: Res<AffectorBuffer>,
) {
    let resolution = settings.displacement_res.clamp(32, 2048);
    let image = images.add(map_image(resolution));
    let trample = [
        images.add(map_image(resolution)),
        images.add(map_image(resolution)),
    ];
    let map = DisplacementMap {
        image: image.clone(),
        trample,
        write: 0,
        resolution,
        mapping: mapping(),
    };
    let (radius_scale, push_scale) = debug_scales();
    let active = settings.instancing == InstancingMode::MeshChunksMap;
    commands.spawn((
        Name::new("displacement camera"),
        DisplacementCamera,
        Camera2d,
        Camera {
            order: -10,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            is_active: active,
            ..default()
        },
        RenderTarget::Image(ImageRenderTarget {
            handle: image,
            scale_factor: 1.0,
        }),
        Hdr,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: MAP_SIZE,
                height: MAP_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::IDENTITY,
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        RenderLayers::layer(MAP_LAYER),
    ));
    let quad = meshes.add(Rectangle::new(MAP_SIZE, MAP_SIZE));
    let displacement = materials.add(DisplacementMaterial {
        params: DisplacementParams {
            debug: Vec4::new(radius_scale, push_scale, settings.trample as u32 as f32, 0.0),
            map: mapping(),
        },
        affectors: affector_buffer.0.clone(),
        trample: map.written(),
    });
    commands.spawn((
        Name::new("displacement quad"),
        Mesh2d(quad.clone()),
        MeshMaterial2d(displacement.clone()),
        Transform::IDENTITY,
        RenderLayers::layer(MAP_LAYER),
    ));

    // The trail map: its own camera, drawn before the displacement pass
    // reads it. The quad covers the target and writes every texel from the
    // other image of the pair, so what the view loads never matters.
    commands.spawn((
        Name::new("trample camera"),
        TrampleCamera,
        Camera2d,
        Camera {
            order: -20,
            clear_color: ClearColorConfig::None,
            is_active: active && settings.trample,
            ..default()
        },
        RenderTarget::Image(ImageRenderTarget {
            handle: map.written(),
            scale_factor: 1.0,
        }),
        Hdr,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: MAP_SIZE,
                height: MAP_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::IDENTITY,
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        RenderLayers::layer(TRAMPLE_LAYER),
    ));
    let trample = tramples.add(TrampleMaterial {
        params: trample_params(&settings, radius_scale, push_scale),
        affectors: affector_buffer.0.clone(),
        previous: map.previous(),
    });
    commands.spawn((
        Name::new("trample quad"),
        Mesh2d(quad),
        MeshMaterial2d(trample.clone()),
        Transform::IDENTITY,
        RenderLayers::layer(TRAMPLE_LAYER),
    ));
    commands.insert_resource(MapMaterials {
        displacement,
        trample,
    });
    commands.insert_resource(map);
}

fn trample_params(settings: &BenchSettings, radius_scale: f32, push_scale: f32) -> TrampleParams {
    TrampleParams {
        params: Vec4::new(
            radius_scale,
            push_scale,
            settings.trample_strength,
            settings.trample_scorch,
        ),
        recover: Vec4::new(settings.trample_recover.max(0.0), 0.0, 0.0, 0.0),
    }
}

/// Swaps the trail pair every frame: the quad reads what the map held last
/// frame and writes the other image, which the displacement pass (a later
/// camera order) then reads. Without the swap the pass would sample the
/// texture it renders into.
fn ping_pong(
    settings: Res<BenchSettings>,
    mut map: ResMut<DisplacementMap>,
    handles: Res<MapMaterials>,
    mut materials: ResMut<Assets<DisplacementMaterial>>,
    mut tramples: ResMut<Assets<TrampleMaterial>>,
    mut cameras: Query<&mut RenderTarget, With<TrampleCamera>>,
) {
    if !settings.trample || settings.instancing != InstancingMode::MeshChunksMap {
        return;
    }
    map.write ^= 1;
    for mut target in &mut cameras {
        *target = RenderTarget::Image(ImageRenderTarget {
            handle: map.written(),
            scale_factor: 1.0,
        });
    }
    if let Some(mut m) = tramples.get_mut(&handles.trample) {
        m.previous = map.previous();
    }
    if let Some(mut m) = materials.get_mut(&handles.displacement) {
        m.trample = map.written();
    }
}

/// Activates the map camera only in the map mode (its pass is then part of
/// that mode's cost and of no other) and recreates the image when the
/// resolution knob moves.
fn sync(
    settings: Res<BenchSettings>,
    mut map: ResMut<DisplacementMap>,
    handles: Res<MapMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<DisplacementMaterial>>,
    mut tramples: ResMut<Assets<TrampleMaterial>>,
    mut cameras: Query<(&mut Camera, &mut RenderTarget), With<DisplacementCamera>>,
    mut trample_cameras: Query<&mut Camera, (With<TrampleCamera>, Without<DisplacementCamera>)>,
) {
    if !settings.is_changed() {
        return;
    }
    let resolution = settings.displacement_res.clamp(32, 2048);
    let recreated = map.resolution != resolution;
    if recreated {
        map.image = images.add(map_image(resolution));
        map.trample = [
            images.add(map_image(resolution)),
            images.add(map_image(resolution)),
        ];
        map.resolution = resolution;
    }
    let active = settings.instancing == InstancingMode::MeshChunksMap;
    for (mut camera, mut target) in &mut cameras {
        if camera.is_active != active {
            camera.is_active = active;
        }
        if recreated {
            *target = RenderTarget::Image(ImageRenderTarget {
                handle: map.image.clone(),
                scale_factor: 1.0,
            });
        }
    }
    // `ping_pong` hands the trample camera its target every frame.
    for mut camera in &mut trample_cameras {
        let wanted = active && settings.trample;
        if camera.is_active != wanted {
            camera.is_active = wanted;
        }
    }
    let (radius_scale, push_scale) = debug_scales();
    if let Some(mut m) = materials.get_mut(&handles.displacement) {
        let weight = settings.trample as u32 as f32;
        if m.params.debug.z != weight {
            m.params.debug.z = weight;
        }
    }
    let params = trample_params(&settings, radius_scale, push_scale);
    if let Some(mut m) = tramples.get_mut(&handles.trample)
        && (m.params.params != params.params || m.params.recover != params.recover)
    {
        m.params = params;
    }
}
