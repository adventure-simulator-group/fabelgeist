use adventuresim_character_creator::plate_gpu;
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    asset::RenderAssetUsages,
    camera::RenderTarget,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{ExitCondition, WindowPlugin},
    winit::WinitPlugin,
};
use fabelgeist_armor::{Armor, Construction};

/// Side of the baked metal maps, in texels.
const PREVIEW_TEXTURE_SIZE: u32 = 512;

#[derive(Resource)]
struct Target(Handle<Image>);
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.055)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(16),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let target = images.add(Image::new_target_texture(
        1600,
        900,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.insert_resource(Target(target.clone()));
    commands.spawn((
        Camera3d::default(),
        bevy::camera::ShadowLodOrigin,
        RenderTarget::Image(target.into()),
        Projection::Perspective(PerspectiveProjection {
            fov: 0.58,
            ..default()
        }),
        Transform::from_xyz(0.25, 1.6, 2.35).looking_at(Vec3::new(0.0, 1.18, 0.0), Vec3::Y),
    ));
    for (position, color, intensity) in [
        (
            Vec3::new(-1.5, 3.0, 2.0),
            Color::srgb(0.8, 0.88, 1.0),
            100000.0,
        ),
        (
            Vec3::new(2.0, 1.8, 1.0),
            Color::srgb(1.0, 0.85, 0.65),
            80000.0,
        ),
        (Vec3::new(0.0, 2.0, -1.0), Color::WHITE, 120000.0),
    ] {
        commands.spawn((
            PointLight {
                color,
                intensity,
                radius: 0.6,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
    for (i, mode) in [
        Construction::Solid,
        Construction::Lamellar,
        Construction::Scale,
    ]
    .into_iter()
    .enumerate()
    {
        let mut a = Armor {
            construction: mode,
            ..default()
        };
        a.translation[0] = (i as f32 - 1.0) * 0.58;
        if mode == Construction::Lamellar {
            a.plate.roundness = 0.2;
            a.plate.stagger = 0.0;
            a.plate.hole_pairs = 3;
        }
        if mode == Construction::Scale {
            a.plate.roundness = 1.0;
            a.plate.hole_pairs = 1;
        }
        let material = metal_material(&a, &mut images, &mut materials);
        for p in plate_gpu().unwrap().build(&a).unwrap() {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.mesh.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.mesh.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.mesh.uvs)
            .with_inserted_indices(Indices::U32(p.mesh.faces.into_iter().flatten().collect()));
            mesh.generate_tangents().unwrap();
            commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone())));
        }
    }
}
/// The armor's baked metal as a repeating preview material.
fn metal_material(
    armor: &Armor,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let textures = plate_gpu()
        .unwrap()
        .textures(&armor.metal, PREVIEW_TEXTURE_SIZE)
        .unwrap();
    let mut texture = |data| {
        let mut image = Image::new(
            Extent3d {
                width: PREVIEW_TEXTURE_SIZE,
                height: PREVIEW_TEXTURE_SIZE,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
        images.add(image)
    };
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.65, 0.68),
        metallic: 1.0,
        perceptual_roughness: 1.0,
        normal_map_texture: Some(texture(textures.normal)),
        metallic_roughness_texture: Some(texture(textures.metal_roughness)),
        ..default()
    })
}

fn capture(
    mut commands: Commands,
    target: Res<Target>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;
    if *frames == 120 {
        commands.spawn(Screenshot::image(target.0.clone())).observe(
            |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                let path = std::env::args()
                    .nth(1)
                    .unwrap_or_else(|| "target/armor-preview.png".into());
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .unwrap()
                    .save(&path)
                    .unwrap();
                println!("Saved {path}");
                exit.write(AppExit::Success);
            },
        );
    }
    if *frames > 600 {
        exit.write(AppExit::error());
    }
}
