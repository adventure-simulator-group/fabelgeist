//! Native authoring and deterministic PBR review of the production plant meshes.
mod capture;
mod controls;
mod recipe;
use adventuresim_plant_generator::{
    PlantLod,
    flower::{FlowerParameters, FlowerSpecies},
};
use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use clap::{Parser, ValueEnum};
use recipe::{Family, Recipe};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum, serde::Serialize)]
enum View {
    Full,
    Head,
    Underside,
}
impl View {
    fn slug(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Head => "head",
            Self::Underside => "underside",
        }
    }
}
#[derive(Parser, Resource)]
struct Options {
    #[arg(long, value_enum, default_value = "flowers")]
    family: Family,
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
    #[arg(long, value_enum, default_value = "high")]
    lod: PlantLod,
}
#[derive(Resource)]
struct Editor {
    lod: PlantLod,
    parameters: Recipe,
    preset: usize,
    dirty: bool,
    reframe: bool,
    error: String,
    origin: RecipeOrigin,
}
#[derive(Clone, Copy)]
enum RecipeOrigin {
    Preset,
    Custom,
}
#[derive(Component)]
struct Specimen;

fn main() {
    let options = Options::parse();
    assert!(
        options.preset < options.family.count(),
        "preset index out of range"
    );
    let parameters: Recipe = options
        .document
        .as_ref()
        .map(|path| {
            serde_json::from_slice(&std::fs::read(path).expect("read plant document"))
                .expect("valid plant document")
        })
        .unwrap_or_else(|| options.family.recipe(options.preset));
    parameters
        .validate()
        .expect("valid plant recipe dimensions and profiles");
    let editor = Editor {
        lod: options.lod,
        parameters,
        preset: if options.document.is_some() {
            0
        } else {
            options.preset
        },
        dirty: true,
        reframe: false,
        error: String::new(),
        origin: if options.document.is_some() {
            RecipeOrigin::Custom
        } else {
            RecipeOrigin::Preset
        },
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

fn camera(options: &Options, p: &Recipe) -> (Vec3, Vec3) {
    let (height, target, extent) = p.framing();
    if matches!(options.view, View::Underside) {
        (target + Vec3::new(0.7, -0.8, 1.5) * extent, target)
    } else if matches!(options.view, View::Head) {
        (target + Vec3::new(0.7, 0.95, 1.5) * extent, target)
    } else {
        let target = Vec3::Y * height * 0.52;
        (target + Vec3::new(0.6, 0.35, 1.65) * height, target)
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
    match editor.parameters.generate(options.seed, editor.lod) {
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
