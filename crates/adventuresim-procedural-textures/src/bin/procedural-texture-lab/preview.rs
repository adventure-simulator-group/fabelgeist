//! Mipmapped, configurable-light Bevy captures of exported texture maps, with matching before/after crops.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    asset::RenderAssetUsages,
    camera::{RenderTarget, ScalingMode},
    core_pipeline::tonemapping::Tonemapping,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    math::Affine2,
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
    window::ExitCondition,
    winit::WinitPlugin,
};

use adventuresim_procedural_textures::TextureRecipeId;

const CAPTURE_WIDTH: u32 = 1600;
const CAPTURE_HEIGHT: u32 = 1000;
const WARMUP_FRAMES: u32 = 120;
const CAPTURE_FRAME_SECONDS: f64 = 1.0 / 30.0;
const PANEL_SIZE: f32 = 1.50;
const PANEL_OFFSET: f32 = 0.80;
const VIEW_HEIGHT: f32 = 2.12;
const DISTANCE_REPEATS: f32 = 4.0;
const RAKING_LIGHT_DEPTH: f32 = 0.28;
const OBLIQUE_PANEL_RADIANS: f32 = 0.65;
const DIAGNOSTIC_ROUGHNESS: u8 = 210;
const DIAGNOSTIC_GRAY: f32 = 0.42;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub(super) enum View {
    Detail,
    Overview,
    Oblique,
    Distance,
}

#[derive(Clone, Copy)]
pub(super) struct Settings {
    pub(super) view: View,
    pub(super) diagnostic: bool,
    pub(super) light_angle: f32,
}

#[derive(Resource)]
struct Capture {
    target: Handle<Image>,
    output: PathBuf,
    frames: u32,
}

pub(super) fn run(
    recipe: TextureRecipeId,
    directory: &Path,
    output: &Path,
    settings: Settings,
) -> Result<(), String> {
    if !settings.light_angle.is_finite() {
        return Err("light angle must be finite".to_owned());
    }
    let (crop, offset) = match (recipe, settings.view) {
        (
            TextureRecipeId::HewnOak
            | TextureRecipeId::DressedStone
            | TextureRecipeId::HandmadeBrick,
            View::Overview,
        ) => (1.0, Vec2::ZERO),
        (TextureRecipeId::HewnOak, View::Detail | View::Oblique) => (0.38, Vec2::new(0.523, 0.177)),
        (TextureRecipeId::DressedStone, View::Detail | View::Oblique) => {
            (0.25, Vec2::new(0.30, 0.36))
        }
        (
            TextureRecipeId::HewnOak
            | TextureRecipeId::DressedStone
            | TextureRecipeId::HandmadeBrick,
            View::Distance,
        ) => (DISTANCE_REPEATS, Vec2::ZERO),
        (TextureRecipeId::HandmadeBrick, View::Detail | View::Oblique) => {
            (0.65, Vec2::new(0.17, 0.21))
        }
        _ => {
            return Err(
                "comparison previews support hewn-oak, handmade-brick and dressed-stone".to_owned(),
            );
        }
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
        CAPTURE_FRAME_SECONDS,
    )))
    .insert_resource(ClearColor(Color::srgb(0.035, 0.043, 0.052)))
    .insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 120.0,
        ..default()
    });
    setup_scene(
        app.world_mut(),
        recipe,
        directory,
        output,
        crop,
        offset,
        settings,
    )?;
    app.add_systems(Update, capture).run();
    Ok(())
}

fn setup_scene(
    world: &mut World,
    recipe: TextureRecipeId,
    directory: &Path,
    output: &Path,
    crop: f32,
    offset: Vec2,
    settings: Settings,
) -> Result<(), String> {
    let target = world
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            CAPTURE_WIDTH,
            CAPTURE_HEIGHT,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    world.spawn((
        Camera3d::default(),
        IsDefaultUiCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_3d()
        }),
        RenderTarget::Image(target.clone().into()),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::TonyMcMapface,
    ));
    world.spawn((
        DirectionalLight {
            illuminance: 7_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(
            settings.light_angle.to_radians().sin(),
            settings.light_angle.to_radians().cos(),
            RAKING_LIGHT_DEPTH,
        ))
        .looking_at(Vec3::ZERO, Vec3::Y),
    ));
    spawn_panels(world, recipe, directory, crop, offset, settings)?;
    let title = format!(
        "{}  /  {:?}  /  {}",
        recipe.slug().to_uppercase(),
        settings.view,
        if settings.diagnostic {
            "CONSTANT COLOR + ROUGHNESS"
        } else {
            "PRODUCTION MAPS"
        }
    );
    spawn_label(world, title, 28.0, 0.0, CAPTURE_WIDTH as f32, 26.0);
    spawn_label(
        world,
        format!(
            "Bevy StandardMaterial | Full mip chains | Light azimuth {} degrees",
            settings.light_angle
        ),
        930.0,
        0.0,
        CAPTURE_WIDTH as f32,
        22.0,
    );
    world.insert_resource(Capture {
        target,
        output: output.to_owned(),
        frames: 0,
    });
    Ok(())
}

fn spawn_panels(
    world: &mut World,
    recipe: TextureRecipeId,
    directory: &Path,
    crop: f32,
    offset: Vec2,
    settings: Settings,
) -> Result<(), String> {
    for (label, x) in [("before", -PANEL_OFFSET), ("after", PANEL_OFFSET)] {
        let root = directory.join(label);
        let albedo = load_map(
            world,
            &root.join(format!("{}-albedo.png", recipe.slug())),
            true,
        )?;
        let normal = load_map(
            world,
            &root.join(format!("{}-normal.png", recipe.slug())),
            false,
        )?;
        let arm = load_map(
            world,
            &root.join(format!("{}-arm.png", recipe.slug())),
            false,
        )?;
        if settings.diagnostic {
            let mut images = world.resource_mut::<Assets<Image>>();
            for pixel in images
                .get_mut(&arm)
                .unwrap()
                .data
                .as_mut()
                .unwrap()
                .as_chunks_mut::<4>()
                .0
            {
                pixel[1] = DIAGNOSTIC_ROUGHNESS;
                pixel[2] = 0;
            }
        }
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: if settings.diagnostic {
                    None
                } else {
                    Some(albedo)
                },
                base_color: if settings.diagnostic {
                    Color::srgb(DIAGNOSTIC_GRAY, DIAGNOSTIC_GRAY, DIAGNOSTIC_GRAY)
                } else {
                    Color::WHITE
                },
                normal_map_texture: Some(normal),
                metallic_roughness_texture: Some(arm.clone()),
                occlusion_texture: Some(arm),
                perceptual_roughness: 1.0,
                metallic: 1.0,
                uv_transform: Affine2::from_scale_angle_translation(Vec2::splat(crop), 0.0, offset),
                ..default()
            });
        let mut panel_mesh = Mesh::from(Rectangle::new(PANEL_SIZE, PANEL_SIZE));
        panel_mesh
            .generate_tangents()
            .map_err(|error| error.to_string())?;
        let mesh = world.resource_mut::<Assets<Mesh>>().add(panel_mesh);
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(x, -0.01, 0.0).with_rotation(Quat::from_rotation_y(
                if matches!(settings.view, View::Oblique) {
                    OBLIQUE_PANEL_RADIANS
                } else {
                    0.0
                },
            )),
        ));
        let panel_pixels = PANEL_SIZE / VIEW_HEIGHT * CAPTURE_HEIGHT as f32;
        let left = CAPTURE_WIDTH as f32 * 0.5 + x / VIEW_HEIGHT * CAPTURE_HEIGHT as f32
            - panel_pixels * 0.5;
        spawn_label(world, label.to_uppercase(), 100.0, left, panel_pixels, 25.0);
    }
    Ok(())
}

fn spawn_label(world: &mut World, text: String, top: f32, left: f32, width: f32, font_size: f32) {
    world.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(Color::srgb(0.72, 0.77, 0.82)),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            left: Val::Px(left),
            width: Val::Px(width),
            ..default()
        },
    ));
}

fn load_map(world: &mut World, path: &Path, srgb: bool) -> Result<Handle<Image>, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut image = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        srgb,
        ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            anisotropy_clamp: 8,
            ..ImageSamplerDescriptor::linear()
        }),
        RenderAssetUsages::default(),
    )
    .map_err(|error| error.to_string())?;
    let size = image.texture_descriptor.size;
    if size.width != size.height || !size.width.is_power_of_two() {
        return Err("comparison maps must be square power-of-two textures".to_owned());
    }
    let levels = size.width.ilog2() + 1;
    let expected: usize = (0..levels)
        .map(|level| ((size.width >> level).pow(2) * 4) as usize)
        .sum();
    let mips = fs::read(path.with_extension("mips"))
        .map_err(|error| format!("{}: {error}", path.with_extension("mips").display()))?;
    if mips.len() != expected {
        return Err(format!("invalid mip payload for {}", path.display()));
    }
    let base = image.data.as_deref().ok_or("PNG has no pixel data")?;
    if mips.get(..base.len()) != Some(base) {
        return Err(format!(
            "PNG and mip payload disagree for {}",
            path.display()
        ));
    }
    image.data = Some(mips);
    image.texture_descriptor.mip_level_count = levels;
    Ok(world.resource_mut::<Assets<Image>>().add(image))
}

fn capture(mut commands: Commands, mut state: ResMut<Capture>) {
    state.frames += 1;
    if state.frames != WARMUP_FRAMES {
        return;
    }
    let output = state.output.clone();
    commands
        .spawn(Screenshot::image(state.target.clone()))
        .observe(
            move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                captured
                    .image
                    .clone()
                    .try_into_dynamic()
                    .expect("RGBA capture image")
                    .save(&output)
                    .expect("write comparison screenshot");
                exit.write(AppExit::Success);
            },
        );
}
