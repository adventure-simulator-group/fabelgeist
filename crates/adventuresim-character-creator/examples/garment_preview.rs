//! Render an exported garment GLB with the runtime renderer.
//! cargo run --example garment_preview -- INPUT.glb OUTPUT.png
use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    camera::RenderTarget,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::{ExitCondition, WindowPlugin},
    winit::WinitPlugin,
};
use std::path::PathBuf;

#[derive(Resource)]
struct Preview {
    scene: Handle<WorldAsset>,
    image: Handle<Image>,
    output: PathBuf,
    ready_frames: u32,
}

fn main() {
    let input = PathBuf::from(std::env::args().nth(1).expect("input GLB required"))
        .canonicalize()
        .expect("input GLB exists");
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.055)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: input.parent().unwrap().to_string_lossy().into_owned(),
                    ..default()
                })
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

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let input = PathBuf::from(std::env::args().nth(1).unwrap());
    let scene = assets.load(
        GltfAssetLabel::Scene(0)
            .from_asset(input.file_name().unwrap().to_string_lossy().into_owned()),
    );
    commands.spawn(WorldAssetRoot(scene.clone()));
    let image = images.add(Image::new_target_texture(
        900,
        1100,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands.spawn((
        Camera3d::default(),
        bevy::camera::ShadowLodOrigin,
        RenderTarget::Image(image.clone().into()),
        Transform::from_xyz(1.7, 1.6, 3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
    for (position, intensity) in [
        (Vec3::new(-1.5, 3.0, 2.0), 100000.0),
        (Vec3::new(2.0, 1.8, 1.0), 80000.0),
        (Vec3::new(0.0, 2.0, -1.0), 120000.0),
    ] {
        commands.spawn((
            PointLight {
                intensity,
                radius: 0.6,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
    commands.insert_resource(Preview {
        scene,
        image,
        output: PathBuf::from(std::env::args().nth(2).expect("output PNG required")),
        ready_frames: 0,
    });
}

fn capture(
    mut commands: Commands,
    mut preview: ResMut<Preview>,
    assets: Res<AssetServer>,
    materials: Res<Assets<StandardMaterial>>,
    meshes: Query<&Mesh3d>,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
) {
    *frames += 1;
    if assets.is_loaded_with_dependencies(preview.scene.id()) && !meshes.is_empty() {
        preview.ready_frames += 1;
    }
    if preview.ready_frames == 120 {
        let mail = materials
            .iter()
            .filter(|(_, m)| {
                matches!(m.alpha_mode, AlphaMode::Mask(_))
                    && m.metallic == 1.0
                    && m.normal_map_texture.is_some()
            })
            .count();
        assert!(
            mail > 0,
            "runtime scene omitted the cutout metallic normal-mapped garment"
        );
        println!(
            "Runtime loaded {} meshes and {mail} chainmail material(s)",
            meshes.iter().count()
        );
        let output = preview.output.clone();
        commands
            .spawn(Screenshot::image(preview.image.clone()))
            .observe(
                move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .unwrap()
                        .save(&output)
                        .unwrap();
                    println!("Saved {}", output.display());
                    exit.write(AppExit::Success);
                },
            );
    }
    if *frames > 1800 {
        exit.write(AppExit::error());
    }
}
