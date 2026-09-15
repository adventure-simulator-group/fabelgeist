//! Native authoring and deterministic PBR review of the production plant meshes.
mod capture;
mod controls;
use adventuresim_plant_generator::{
    Tessellation,
    flower::{FlowerParameters, FlowerSpecies},
};
use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum, serde::Serialize)]
enum View {
    Full,
    Head,
}
impl View {
    fn slug(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Head => "head",
        }
    }
}
#[derive(Parser, Resource)]
struct Options {
    #[arg(long, default_value_t = 0)]
    preset: usize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long)]
    document: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "full")]
    view: View,
    #[arg(long)]
    field: bool,
}
#[derive(Resource)]
struct Editor {
    parameters: FlowerParameters,
    preset: usize,
    dirty: bool,
    reframe: bool,
    error: String,
}
#[derive(Component)]
struct Specimen;

fn main() {
    let options = Options::parse();
    assert!(
        options.preset < FlowerSpecies::ALL.len(),
        "preset index out of range"
    );
    let parameters = options
        .document
        .as_ref()
        .map(|path| {
            serde_json::from_slice(&std::fs::read(path).expect("read plant document"))
                .expect("valid plant document")
        })
        .unwrap_or_else(|| FlowerSpecies::ALL[options.preset].parameters());
    let editor = Editor {
        parameters,
        preset: options.preset,
        dirty: true,
        reframe: false,
        error: String::new(),
    };
    App::new()
        .insert_resource(editor)
        .insert_resource(options)
        .insert_resource(ClearColor(Color::srgb(0.13, 0.15, 0.17)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fabelgeist · Plant Studio".into(),
                resolution: WindowResolution::new(1400, 1100).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), PanOrbitCameraPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (rebuild, frame_camera, capture::capture).chain())
        .add_systems(EguiPrimaryContextPass, controls::draw)
        .run();
}

fn setup(mut commands: Commands, options: Res<Options>, editor: Res<Editor>) {
    let (eye, target) = camera(&options, &editor.parameters);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: 0.001,
            ..default()
        }),
        AmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        },
        Transform::from_translation(eye).looking_at(target, Vec3::Y),
        PanOrbitCamera {
            focus: target,
            ..default()
        },
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 9000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn camera(options: &Options, p: &FlowerParameters) -> (Vec3, Vec3) {
    if matches!(options.view, View::Head) {
        let target = Vec3::new(p.height_m * p.stem_lean, p.height_m, 0.0);
        let extent = (p.petal_length_m + p.center_radius_m) * 4.0;
        (target + Vec3::new(0.7, 0.95, 1.5) * extent, target)
    } else {
        let target = Vec3::Y * p.height_m * 0.52;
        (target + Vec3::new(0.6, 0.35, 1.65) * p.height_m, target)
    }
}

fn frame_camera(
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    options: Res<Options>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    if !editor.reframe {
        return;
    }
    editor.reframe = false;
    let (eye, target) = camera(&options, &editor.parameters);
    for entity in &cameras {
        commands.entity(entity).insert((
            Transform::from_translation(eye).looking_at(target, Vec3::Y),
            PanOrbitCamera {
                focus: target,
                ..default()
            },
        ));
    }
}

fn rebuild(
    mut commands: Commands,
    mut editor: ResMut<Editor>,
    options: Res<Options>,
    old: Query<Entity, With<Specimen>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !editor.dirty {
        return;
    }
    editor.dirty = false;
    let detail = if options.field {
        Tessellation::Field
    } else {
        Tessellation::Close
    };
    match editor.parameters.generate(options.seed, detail) {
        Ok(mesh) => {
            for entity in &old {
                commands.entity(entity).despawn();
            }
            commands.spawn((
                Specimen,
                Mesh3d(meshes.add(mesh.into_bevy())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    perceptual_roughness: 0.72,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::default(),
            ));
            editor.error.clear();
        }
        Err(error) => editor.error = error.to_string(),
    }
}
