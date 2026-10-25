//! Hair on the default body drawn headless, as the studio draws it -- the
//! same meshes, materials, scalp paint and lights -- in every level of
//! detail and from five views: how a change to hair is looked at without
//! the studio's window.
//!
//! ```text
//! MHR_ASSETS=target/mhr-assets/v1.0.1/assets HAIR_PRESET=long cargo test --release --manifest-path crates/adventuresim-character-creator/Cargo.toml --bin adventuresim-character-creator hair_renders -- --ignored --nocapture
//! ```
//!
//! writes `target/hair-render/<preset>-<form>-<view>.png`. `HAIR_PRESET` takes
//! a recipe's hairstyle name, `long` when unset. `HAIR_ON_BUST=1` draws the
//! hair on the hair crate's own bust instead, as it grew there, to
//! `bust-<preset>-<form>-<view>.png`: what placing it on a character is
//! measured against. Beside them it writes the scalp's baked textures,
//! `<preset>-scalp-texture.png` and `<preset>-scalp-strands.png`. `HAIR_SHAPE`
//! varies the preset as the Hair page does, taking the shape's recipe fields
//! that differ from its default -- for instance `{"length": 0.6, "curl": 0.5}`
//! -- and names the images `<preset>-shaped-...`. `HAIR_PIGMENT` colours it as
//! a recipe's pigment does -- `{"melanin": 0.1, "redness": 0.05, "dye": [0.12,
//! 0.38, 1.0]}`, say -- and names the images `<preset>-coloured-...`.
//! `HAIR_SHOW=scalp` or `HAIR_SHOW=hair` draws the scalp or the hair alone, as
//! the Hair page's switches do, to `<preset>-scalp-only-...` or
//! `<preset>-hair-only-...`.
use super::*;
use adventuresim_character_creator::hair::{
    GpuGeometry, HairGeometry, HairGpu, HairPigment, HairShape, PlacedHair, ScalpTextureChain,
};
use adventuresim_character_creator::studio_environment;
use bevy::app::{PluginsState, ScheduleRunnerPlugin};
use bevy::camera::RenderTarget;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use fabelgeist_gpu::globals::WgpuContext;
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
    fn character(assets: &std::ffi::OsStr, gpu: &HairGpu, grown: &GroomedHair) -> Result<Self> {
        let model = load_body_model(
            std::path::Path::new(assets),
            adventuresim_character_creator::lod::MIN_CHARACTER_LOD,
            false,
            &Device::default(),
        )?;
        let generated = generate_character(&model, &CharacterRecipe::default())?;
        let head = BodyHead::new(&model, &generated);
        let started = std::time::Instant::now();
        let placed = head.place(gpu, grown)?;
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
    fn bust(gpu: &HairGpu, grown: &GroomedHair) -> Result<Self> {
        let (placed, bust) = grown.on_bust(gpu)?;
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
        Ok(Self {
            placed,
            positions,
            faces,
            body,
        })
    }
}

/// The captures still to write.
#[derive(Resource)]
struct Captures {
    shots: Vec<(Handle<Image>, PathBuf)>,
    frames: u32,
}

/// The layers `HAIR_SHOW` asks for: `hair` or `scalp` alone, or both.
fn test_layers() -> Result<ShownLayers> {
    match std::env::var("HAIR_SHOW").as_deref() {
        Err(_) => Ok(ShownLayers::default()),
        Ok("hair") => Ok(ShownLayers {
            hair: true,
            scalp: false,
        }),
        Ok("scalp") => Ok(ShownLayers {
            hair: false,
            scalp: true,
        }),
        Ok(other) => anyhow::bail!("HAIR_SHOW takes hair or scalp, not {other}"),
    }
}

/// The pigment `HAIR_PIGMENT` asks for, as a recipe gives it, or the
/// default.
fn test_pigment() -> Result<HairPigment> {
    match std::env::var("HAIR_PIGMENT") {
        Ok(pigment) => Ok(serde_json::from_str(&pigment)?),
        Err(_) => Ok(HairPigment::default()),
    }
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
        pigment: test_pigment()?,
    };
    // Hair is built on the device it is drawn with, as in the studio.
    let (mut app, context) = render_app()?;
    let started = std::time::Instant::now();
    let gpu = HairGpu::new(context)?;
    println!(
        "hair kernels compiled in {:.2} s",
        started.elapsed().as_secs_f32()
    );
    let started = std::time::Instant::now();
    let grown = GroomedHair::grow(design.groom(), &gpu)?;
    println!("grown in {:.2} s", started.elapsed().as_secs_f32());
    let on_bust = std::env::var_os("HAIR_ON_BUST").is_some();
    let subject = if on_bust {
        Subject::bust(&gpu, &grown)?
    } else {
        Subject::character(&assets, &gpu, &grown)?
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
    let mut cap = assets::cap_mesh(placed.cap(), placed.placement().head_joint)?;
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
    let preset_name = if design.pigment == HairPigment::default() {
        preset_name
    } else {
        format!("{preset_name}-coloured")
    };
    let shown = test_layers()?;
    let preset_name = match (shown.hair, shown.scalp) {
        (true, true) => preset_name,
        (false, _) => format!("{preset_name}-scalp-only"),
        (true, false) => format!("{preset_name}-hair-only"),
    };
    let textures = placed.scalp_texture().read(&gpu)?;
    for (name, levels) in [("texture", &textures.shade), ("strands", &textures.strands)] {
        let texture = levels.first().context("no scalp texture")?;
        image::RgbaImage::from_raw(texture.width, texture.height, texture.rgba.clone())
            .context("the scalp texture's size")?
            .save(out.join(format!("{subject_name}{preset_name}-scalp-{name}.png")))?;
    }

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
        let Some(geometry) = placed.gpu_geometry(&gpu, form)? else {
            continue;
        };
        pollster::block_on(gpu.finish())?;
        let built = started.elapsed();
        let buried = buried_share(&geometry.read(&gpu)?, &skin);
        println!(
            "{form:?}: built in {:.2} s, {:.1}% of hair triangles buried in the skin",
            built.as_secs_f32(),
            buried * 100.0
        );
        let form_name = match form {
            Form::Strands { level, .. } => format!("strands{}", level.index()),
            Form::Cards { level } => format!("cards{}", level.index()),
        };
        // Drawn in the bind pose, skinned as the studio skins it but to one
        // joint left where it is.
        let mesh = assets::mesh(&geometry, fabelgeist_rig::RigJointOrdinal::from(0usize))?;
        let paths = VIEWS
            .iter()
            .map(|view| {
                out.join(format!(
                    "{subject_name}{preset_name}-{form_name}-{}.png",
                    view.name
                ))
            })
            .collect();
        hairs.push((form, mesh, geometry, paths));
    }
    let stage = Stage {
        body: Some(body),
        cap: Some(cap),
        atlas: Some(assets::atlas_image(&placed)),
        scalp: Some([
            placed.scalp_texture().shade().clone(),
            placed.scalp_texture().strands().clone(),
        ]),
        hairs,
        design,
        center,
        shown,
    };

    app.insert_resource(stage)
        .insert_resource(Captures {
            shots: std::mem::take(&mut shots),
            frames: 0,
        })
        .add_systems(Startup, spawn)
        .add_systems(Update, capture);
    app.run();
    pollster::block_on(gpu.finish())?;
    println!("wrote {}", out.display());
    Ok(())
}

/// The studio's renderer, headless, started, and the device it draws with,
/// to build hair on: taken before the render world moves to its own thread.
fn render_app() -> Result<(App, WgpuContext)> {
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
        ));
    while app.plugins_state() == PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    let context = gpu::render_context(&app).context("the renderer has no device")?;
    app.cleanup();
    Ok((app, context))
}

/// What the startup system spawns: the shaded body, the atlas, and each
/// form's hair mesh with the paths its views are written to.
#[derive(Resource)]
struct Stage {
    body: Option<Mesh>,
    cap: Option<Mesh>,
    atlas: Option<Image>,
    /// The scalp's baked shade and strands chains.
    scalp: Option<[ScalpTextureChain; 2]>,
    hairs: Vec<(Form, Mesh, GpuGeometry, Vec<PathBuf>)>,
    design: HairDesign,
    center: Vec3,
    shown: ShownLayers,
}

/// The body, the light, and per form its hair and a camera for each view,
/// each form on its own render layer.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the stage, the captures and every asset store it fills"
)]
fn spawn(
    mut commands: Commands,
    mut stage: ResMut<Stage>,
    mut captures: ResMut<Captures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hair_materials: ResMut<Assets<strand_shading::HairShading>>,
    mut scalp_materials: ResMut<Assets<scalp_shade::ScalpShading>>,
    mut images: ResMut<Assets<Image>>,
    mut bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    uploads: Res<scalp_upload::ScalpUploads>,
) {
    let (Some(body), Some(cap), Some(atlas), Some([shade, strands])) = (
        stage.body.take(),
        stage.cap.take(),
        stage.atlas.take(),
        stage.scalp.take(),
    ) else {
        return;
    };
    let scalp = scalp_shade::ScalpImages {
        shade: uploads.image(&shade, &mut images),
        strands: uploads.image(&strands, &mut images),
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
        MeshMaterial3d(scalp_materials.add(scalp_shade::material(stage.design, &scalp))),
        stage.shown.visibility(HairLayer::Scalp),
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
    let skin = SkinnedMesh {
        inverse_bindposes: bindposes.add(vec![Mat4::IDENTITY]),
        joints: vec![commands.spawn(Transform::default()).id()],
    };
    let hairs = std::mem::take(&mut stage.hairs);
    for (index, (form, mesh, geometry, paths)) in hairs.into_iter().enumerate() {
        let layer = index + 1;
        let material = assets::material(design, form, atlas.clone(), &geometry);
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(hair_materials.add(material)),
            skin.clone(),
            stage.shown.visibility(HairLayer::Hair),
            RenderLayers::layer(layer),
            NoFrustumCulling,
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
