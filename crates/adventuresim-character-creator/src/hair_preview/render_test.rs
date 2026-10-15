//! Hair on the default body drawn headless, as the studio draws it -- the
//! same meshes, materials, scalp shade and lights -- in every level of
//! detail and from three views: how a change to hair is looked at without
//! the studio's window.
//!
//! ```text
//! MHR_ASSETS=target/mhr-assets/v1.0.1/assets HAIR_PRESET=long cargo test --release --manifest-path crates/adventuresim-character-creator/Cargo.toml --bin adventuresim-character-creator hair_renders -- --ignored --nocapture
//! ```
//!
//! writes `target/hair-render/<preset>-<form>-<view>.png`. `HAIR_PRESET`
//! takes a recipe's hairstyle name, `long` when unset.
use super::*;
use adventuresim_character_creator::hair::{HairGeometry, HairPigment};
use adventuresim_character_creator::studio_environment;
use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use fabelgeist_hair::MeshCollider;
use fabelgeist_hair::collider::Collider;
use fabelgeist_math::Vec3 as HairVec3;
use std::path::PathBuf;
use std::time::Duration;

/// Each image's side, pixels.
const SIDE: u32 = 640;
/// Frames drawn before the capture: the atlas, the environment map and the
/// pipelines settle.
const SETTLE_FRAMES: u32 = 90;
/// Frames waited for the captures to be written.
const WRITE_FRAMES: u32 = 30;

/// A camera round the head: where it stands and where it looks, as offsets
/// from the head's centre, metres; the face looks along +Z.
struct View {
    name: &'static str,
    eye: Vec3,
    target: Vec3,
}

const VIEWS: [View; 3] = [
    // From the front and above: the parting and the crown.
    View {
        name: "front",
        eye: Vec3::new(0.0, 0.3, 0.5),
        target: Vec3::new(0.0, 0.0, 0.0),
    },
    // From behind: the hair down the back.
    View {
        name: "back",
        eye: Vec3::new(0.0, 0.0, -0.75),
        target: Vec3::new(0.0, -0.2, 0.0),
    },
    // Over the left shoulder from behind, close.
    View {
        name: "shoulder",
        eye: Vec3::new(0.45, 0.05, -0.35),
        target: Vec3::new(0.12, -0.22, 0.0),
    },
];

/// The captures still to write.
#[derive(Resource)]
struct Captures {
    shots: Vec<(Handle<Image>, PathBuf)>,
    frames: u32,
}

#[test]
#[ignore = "requires MHR_ASSETS and a GPU"]
fn hair_renders() -> Result<()> {
    let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
    let preset_name = std::env::var("HAIR_PRESET").unwrap_or_else(|_| "long".to_owned());
    let preset: Preset = serde_json::from_value(serde_json::Value::String(preset_name.clone()))?;
    let model = load_body_model(
        std::path::Path::new(&assets),
        adventuresim_character_creator::lod::MIN_CHARACTER_LOD,
        false,
        &Device::default(),
    )?;
    let generated = generate_character(&model, &CharacterRecipe::default())?;
    let head = BodyHead::new(&model, &generated);
    let grown = GroomedHair::grow(preset);
    let placed = head.place(&grown)?;
    let cover = placed.cover(&head.positions, &head.normals);
    let design = HairDesign {
        preset,
        pigment: HairPigment::default(),
    };
    let center = Vec3::from_array(
        placed
            .placement()
            .point(grown.head().cranium.center.to_array()),
    );
    let mut body =
        studio_generation::visible_body_mesh(&generated, &model.mhr.character.mesh.faces);
    body.insert_attribute(Mesh::ATTRIBUTE_COLOR, scalp_shade::colors(design, &cover));
    let forms = placed.forms();
    let out = PathBuf::from("../../target/hair-render");
    std::fs::create_dir_all(&out)?;

    let skin = MeshCollider::new(
        head.positions
            .iter()
            .copied()
            .map(HairVec3::from_array)
            .collect(),
        head.faces.clone(),
        BURIED_REACH,
    );
    let mut shots = Vec::new();
    let mut hairs = Vec::new();
    for &form in &forms {
        let started = std::time::Instant::now();
        let geometry = placed.geometry(form);
        let built = started.elapsed();
        let buried = buried_share(&geometry, &skin);
        println!(
            "{form:?}: built in {:.2} s, {:.1}% of hair triangles buried in the skin",
            built.as_secs_f32(),
            buried * 100.0
        );
        let form_name = match form {
            Form::Strands { level, .. } => format!("strands{level}"),
            Form::Cards { level } => format!("cards{level}"),
        };
        let mut mesh = assets::mesh(geometry, placed.placement().head_joint)?;
        // Drawn in the bind pose, without the skeleton the studio skins it to.
        mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
        mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
        let paths = VIEWS
            .iter()
            .map(|view| out.join(format!("{preset_name}-{form_name}-{}.png", view.name)))
            .collect();
        hairs.push((form, mesh, paths));
    }
    let stage = Stage {
        body: Some(body),
        atlas: Some(assets::atlas_image(&placed)),
        hairs,
        design,
        center,
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(studio_scene::BACKDROP))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO))
        .insert_resource(stage)
        .insert_resource(Captures {
            shots: std::mem::take(&mut shots),
            frames: 0,
        })
        .add_systems(Startup, spawn)
        .add_systems(Update, capture);
    app.run();
    println!("wrote {}", out.display());
    Ok(())
}

/// What the startup system spawns: the shaded body, the atlas, and each
/// form's hair mesh with the paths its views are written to.
#[derive(Resource)]
struct Stage {
    body: Option<Mesh>,
    atlas: Option<Image>,
    hairs: Vec<(Form, Mesh, Vec<PathBuf>)>,
    design: HairDesign,
    center: Vec3,
}

/// The body, the light, and per form its hair and a camera for each view,
/// each form on its own render layer.
fn spawn(
    mut commands: Commands,
    mut stage: ResMut<Stage>,
    mut captures: ResMut<Captures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(body), Some(atlas)) = (stage.body.take(), stage.atlas.take()) else {
        return;
    };
    let atlas = images.add(atlas);
    let environment = images.add(studio_environment::studio_environment());
    let every_layer = RenderLayers::from_layers(&(0..=stage.hairs.len()).collect::<Vec<_>>());
    commands.spawn((
        Mesh3d(meshes.add(body)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: preview::SKIN_COLOR,
            perceptual_roughness: 0.52,
            reflectance: 0.46,
            ..default()
        })),
        every_layer.clone(),
    ));
    commands.spawn((
        SpotLight {
            color: Color::srgb(1.0, 0.90, 0.79),
            intensity: 1_050_000.0,
            range: 8.0,
            radius: 0.38,
            inner_angle: 0.30,
            outer_angle: 0.58,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.025,
            shadow_normal_bias: 1.9,
            shadow_map_near_z: 0.1,
            ..default()
        },
        Transform::from_xyz(-2.4, 4.2, 3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        every_layer,
    ));
    let design = stage.design;
    let center = stage.center;
    for (index, (form, mesh, paths)) in std::mem::take(&mut stage.hairs).into_iter().enumerate() {
        let layer = index + 1;
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(assets::material(design, form, atlas.clone()))),
            RenderLayers::layer(layer),
        ));
        for (view, path) in VIEWS.iter().zip(paths) {
            let mut image =
                Image::new_target_texture(SIDE, SIDE, TextureFormat::Rgba8UnormSrgb, None);
            image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            let image = images.add(image);
            commands.spawn((
                Camera3d::default(),
                RenderTarget::Image(image.clone().into()),
                Transform::from_translation(center + view.eye)
                    .looking_at(center + view.target, Vec3::Y),
                GeneratedEnvironmentMapLight {
                    environment_map: environment.clone(),
                    intensity: studio_environment::STUDIO_ENVIRONMENT_INTENSITY,
                    ..default()
                },
                RenderLayers::from_layers(&[0, layer]),
            ));
            captures.shots.push((image, path));
        }
    }
}

/// How far the skin collider answers, metres: hair farther out is outside.
const BURIED_REACH: f32 = 0.05;

/// The share of `geometry`'s triangles whose centre lies under the skin.
fn buried_share(geometry: &HairGeometry, skin: &MeshCollider) -> f32 {
    let buried = geometry
        .faces
        .iter()
        .filter(|face| {
            let centre = face
                .iter()
                .map(|&vertex| HairVec3::from_array(geometry.positions[vertex as usize]))
                .fold(HairVec3::new(0.0, 0.0, 0.0), |sum, point| sum + point)
                / 3.0;
            skin.distance(centre) < 0.0
        })
        .count();
    buried as f32 / geometry.faces.len().max(1) as f32
}

/// Settle, capture every image, wait for them to be written, and leave.
fn capture(
    mut commands: Commands,
    mut captures: ResMut<Captures>,
    mut exit: MessageWriter<AppExit>,
) {
    captures.frames += 1;
    if captures.frames == SETTLE_FRAMES {
        for (image, path) in captures.shots.drain(..) {
            commands
                .spawn(Screenshot::image(image))
                .observe(save_to_disk(path));
        }
    }
    if captures.frames == SETTLE_FRAMES + WRITE_FRAMES {
        exit.write(AppExit::Success);
    }
}
