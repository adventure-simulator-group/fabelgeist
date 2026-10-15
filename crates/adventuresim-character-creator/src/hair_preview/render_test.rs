//! Hair on the default body drawn headless, as the studio draws it -- the
//! same meshes, materials, scalp shade and lights -- in every level of
//! detail and from five views: how a change to hair is looked at without
//! the studio's window.
//!
//! ```text
//! MHR_ASSETS=target/mhr-assets/v1.0.1/assets HAIR_PRESET=long cargo test --release --manifest-path crates/adventuresim-character-creator/Cargo.toml --bin adventuresim-character-creator hair_renders -- --ignored --nocapture
//! ```
//!
//! writes `target/hair-render/<preset>-<form>-<view>.png`. `HAIR_PRESET`
//! takes a recipe's hairstyle name, `long` when unset. `HAIR_ON_BUST=1`
//! draws the hair on the hair crate's own bust instead, as it grew there,
//! to `bust-<preset>-<form>-<view>.png`: what placing it on a character is
//! measured against. `HAIR_SHAPE` varies the preset as the Hair page does,
//! taking the shape's recipe fields that differ from its default -- for
//! instance `{"length": 0.6, "curl": 0.5}` -- and names the images
//! `<preset>-shaped-...`.
use super::*;
use adventuresim_character_creator::hair::{HairGeometry, HairPigment, HairShape, PlacedHair};
use adventuresim_character_creator::studio_environment;
use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use fabelgeist_hair::collider::Collider;
use fabelgeist_hair::{MeshCollider, Preset};
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

const VIEWS: [View; 5] = [
    // Close over the parting: how the roots leave the scalp.
    View {
        name: "parting",
        eye: Vec3::new(0.05, 0.2, 0.1),
        target: Vec3::new(0.0, 0.1, 0.03),
    },
    // From above: the roots and the scalp between them.
    View {
        name: "crown",
        eye: Vec3::new(0.0, 0.42, 0.1),
        target: Vec3::new(0.0, 0.05, 0.0),
    },
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

/// The hair placed, and the body it is drawn on.
struct Subject {
    placed: PlacedHair,
    positions: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
    body: Mesh,
}

impl Subject {
    /// `grown` placed on the default character, as the studio places it.
    fn character(assets: &std::ffi::OsStr, grown: &GroomedHair) -> Result<Self> {
        let model = load_body_model(
            std::path::Path::new(assets),
            adventuresim_character_creator::lod::MIN_CHARACTER_LOD,
            false,
            &Device::default(),
        )?;
        let generated = generate_character(&model, &CharacterRecipe::default())?;
        let head = BodyHead::new(&model, &generated);
        let started = std::time::Instant::now();
        let placed = head.place(grown)?;
        println!("placed in {:.2} s", started.elapsed().as_secs_f32());
        let body =
            studio_generation::visible_body_mesh(&generated, &model.mhr.character.mesh.faces);
        Ok(Self {
            placed,
            positions: head.positions,
            faces: head.faces,
            body,
        })
    }

    /// `grown` left on its own bust, as the hair crate grew it.
    fn bust(grown: &GroomedHair) -> Self {
        let (placed, bust) = grown.on_bust();
        let positions: Vec<[f32; 3]> = bust.positions.iter().map(|p| p.to_array()).collect();
        let normals: Vec<[f32; 3]> = bust.normals.iter().map(|n| n.to_array()).collect();
        let faces = bust.triangles.clone();
        let body = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(faces.as_flattened().to_vec()));
        Self {
            placed,
            positions,
            faces,
            body,
        }
    }
}

/// The captures still to write.
#[derive(Resource)]
struct Captures {
    shots: Vec<(Handle<Image>, PathBuf)>,
    frames: u32,
}

/// The shape `HAIR_SHAPE` asks for: the default, its fields replaced.
fn test_shape() -> Result<HairShape> {
    let mut shape = serde_json::to_value(HairShape::default())?;
    if let Ok(changes) = std::env::var("HAIR_SHAPE") {
        let changes: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&changes)?;
        let fields = shape
            .as_object_mut()
            .context("a hair shape is a JSON object")?;
        fields.extend(changes);
    }
    Ok(serde_json::from_value(shape)?)
}

#[test]
#[ignore = "requires MHR_ASSETS and a GPU"]
fn hair_renders() -> Result<()> {
    let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
    let preset_name = std::env::var("HAIR_PRESET").unwrap_or_else(|_| "long".to_owned());
    let preset: Preset = serde_json::from_value(serde_json::Value::String(preset_name.clone()))?;
    let design = HairDesign {
        preset,
        shape: test_shape()?,
        pigment: HairPigment::default(),
    };
    let grown = GroomedHair::grow(design.groom());
    let on_bust = std::env::var_os("HAIR_ON_BUST").is_some();
    let subject = if on_bust {
        Subject::bust(&grown)
    } else {
        Subject::character(&assets, &grown)?
    };
    let Subject {
        placed,
        positions,
        faces,
        body,
    } = subject;
    let center = Vec3::from_array(
        placed
            .placement()
            .point(grown.head().cranium.center.to_array()),
    );
    let mut cap = assets::cap_mesh(placed.cap(), design, placed.placement().head_joint)?;
    cap.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
    cap.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
    let forms = placed.forms();
    let out = PathBuf::from("../../target/hair-render");
    std::fs::create_dir_all(&out)?;
    let subject_name = if on_bust { "bust-" } else { "" };
    let preset_name = if design.shape == HairShape::default() {
        preset_name
    } else {
        format!("{preset_name}-shaped")
    };

    let skin = MeshCollider::new(
        positions
            .iter()
            .copied()
            .map(HairVec3::from_array)
            .collect(),
        faces,
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
            .map(|view| {
                out.join(format!(
                    "{subject_name}{preset_name}-{form_name}-{}.png",
                    view.name
                ))
            })
            .collect();
        hairs.push((form, mesh, paths));
    }
    let stage = Stage {
        body: Some(body),
        cap: Some(cap),
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
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::ZERO),
            HairShadingPlugin,
        ))
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
    cap: Option<Mesh>,
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
    mut hair_materials: ResMut<Assets<strand_shading::HairShading>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(body), Some(cap), Some(atlas)) =
        (stage.body.take(), stage.cap.take(), stage.atlas.take())
    else {
        return;
    };
    let atlas = images.add(atlas);
    let environment = images.add(studio_environment::studio_environment());
    let every_layer = RenderLayers::from_layers(&(0..=stage.hairs.len()).collect::<Vec<_>>());
    let skin = materials.add(preview::skin_material());
    commands.spawn((
        Mesh3d(meshes.add(body)),
        MeshMaterial3d(skin),
        every_layer.clone(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(cap)),
        MeshMaterial3d(materials.add(scalp_shade::material())),
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
            MeshMaterial3d(hair_materials.add(assets::material(design, form, atlas.clone()))),
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
