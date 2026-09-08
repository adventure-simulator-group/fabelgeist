//! Reproducible GPU captures of the same working-building meshes consumed by the game.
use adventuresim_building_generator::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::WindowResolution,
};
use clap::{Parser, ValueEnum};
use std::{fs, path::PathBuf};

#[path = "workplace-viewer/materials.rs"]
mod materials;

const CAPTURE_WIDTH: u32 = 1440;
const CAPTURE_HEIGHT: u32 = 1000;
const SETTLE_FRAMES: u32 = 80;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Representation {
    Detail,
    Facade,
    Shell,
}

#[derive(Parser)]
struct Args {
    #[arg(long, value_enum)]
    kind: WorkplaceKind,
    #[arg(long, value_enum, default_value_t = WorkplaceSize::Medium)]
    size: WorkplaceSize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    cutaway: bool,
    #[arg(long, value_enum, default_value_t = Representation::Detail)]
    representation: Representation,
}

#[derive(Resource)]
struct Capture {
    output: PathBuf,
    frames: u32,
    in_flight: bool,
    primed: bool,
}

fn main() {
    let args = Args::parse();
    fs::create_dir_all(args.output.parent().expect("output needs a directory")).unwrap();
    let program = BuildingProgram::settlement(
        settlement_archetype(args.kind.usage()),
        Some(args.kind.usage()),
        args.seed,
    )
    .with_workplace_size(args.size);
    let plan = generate(&program).unwrap_or_else(|error| panic!("invalid workplace: {error:?}"));
    let collision = compile_building_collision(&plan);
    let metadata = serde_json::json!({ "program": program, "cutaway": args.cutaway,
        "representation": format!("{:?}", args.representation), "audit": audit_plan(&plan),
        "collision_cuboids": collision.cuboids.len(), "workplace": plan.workplace });
    fs::write(
        args.output.with_extension("json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    let dimensions = program.plot_dimensions_metres();
    let mut rendered = plan.clone();
    if args.cutaway {
        cutaway(&mut rendered);
    }
    let batches = match args.representation {
        Representation::Detail => compile_building_detail(&rendered).meshes,
        Representation::Facade => compile_building_lod(&rendered, BuildingLodLevel::Facade).meshes,
        Representation::Shell => compile_building_lod(&rendered, BuildingLodLevel::Shell).meshes,
    };
    let title = format!(
        "{} | {:?} | seed {}{}",
        args.kind.slug(),
        args.size,
        args.seed,
        if args.cutaway { " | cutaway" } else { "" }
    );
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.clone(),
                visible: false,
                resolution: WindowResolution::new(CAPTURE_WIDTH, CAPTURE_HEIGHT),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.83, 0.86, 0.85)))
        .insert_resource(Capture {
            output: args.output,
            frames: 0,
            in_flight: false,
            primed: false,
        })
        .add_systems(Startup, move |world: &mut World| {
            setup(world, &batches, dimensions, &title)
        })
        .add_systems(Update, capture)
        .run();
}

fn setup(world: &mut World, batches: &[LodMesh], dimensions: Vec2, title: &str) {
    let origin = Vec3::new(-dimensions.x * 0.5, 0.0, -dimensions.y * 0.5);
    for batch in batches {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh(batch));
        let material = materials::material(world, batch.material);
        world.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(origin),
        ));
    }
    let ground = world.resource_mut::<Assets<Mesh>>().add(Cuboid::new(
        dimensions.x + 2.0,
        0.15,
        dimensions.y + 2.0,
    ));
    let ground_material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.32, 0.23),
            perceptual_roughness: 1.0,
            ..default()
        });
    world.spawn((
        Mesh3d(ground),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.075, 0.0),
    ));
    let height = batches
        .iter()
        .flat_map(|mesh| &mesh.vertices)
        .map(|vertex| vertex.position.y)
        .fold(0.0_f32, f32::max);
    let focus = Vec3::new(0.0, height * 0.5, 0.0);
    let radius = Vec3::new(dimensions.x + 2.0, height, dimensions.y + 2.0).length() * 0.5;
    let distance = radius / (std::f32::consts::FRAC_PI_4 * 0.5).sin() * 1.1;
    let camera = focus + Vec3::new(0.83, 0.55, -0.95).normalize() * distance;
    world.spawn((
        Camera3d::default(),
        Transform::from_translation(camera).looking_at(focus, Vec3::Y),
    ));
    world.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 30.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    world.spawn((
        PointLight {
            intensity: 40_000_000.0,
            range: radius * 3.0,
            ..default()
        },
        Transform::from_translation(camera),
    ));
    world.spawn((
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(27.0),
            ..default()
        },
        TextColor(Color::srgb(0.12, 0.13, 0.12)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(35.0),
            top: Val::Px(26.0),
            ..default()
        },
    ));
}

fn mesh(batch: &LodMesh) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        batch
            .vertices
            .iter()
            .map(|v| v.position.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        batch
            .vertices
            .iter()
            .map(|v| v.normal.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        batch
            .vertices
            .iter()
            .map(|v| v.uv.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_indices(Indices::U32(batch.indices.clone()));
    mesh.generate_tangents()
        .expect("generated building triangles have metric UVs");
    mesh
}

fn capture(mut commands: Commands, mut state: ResMut<Capture>) {
    if state.in_flight {
        return;
    }
    state.frames += 1;
    if state.frames < SETTLE_FRAMES {
        return;
    }
    state.in_flight = true;
    commands.spawn(Screenshot::primary_window()).observe(
        |captured: On<ScreenshotCaptured>,
         mut state: ResMut<Capture>,
         mut exit: MessageWriter<AppExit>| {
            if !state.primed {
                state.primed = true;
                state.in_flight = false;
                state.frames = 0;
                return;
            }
            save_to_disk(&state.output)(captured);
            exit.write(AppExit::Success);
        },
    );
}

fn cutaway(plan: &mut BuildingPlan) {
    let (w, _) = plan.footprint.dimensions();
    let w = f32::from(w) * CELL_SIZE_METRES;
    let removed = plan
        .workplace
        .as_ref()
        .unwrap()
        .parts
        .iter()
        .filter_map(|part| {
            let solid = plan
                .resolved_geometry
                .solids
                .iter()
                .find(|solid| solid.id == part.solid)?;
            (matches!(
                part.feature,
                WorkplaceFeature::Wall | WorkplaceFeature::Boarding
            ) && (solid.centre.z.abs() < 0.3 || (solid.centre.x - w).abs() < 0.3))
                .then_some(part.solid)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let roof_owners = plan
        .roof_assemblies
        .iter()
        .map(|roof| roof.owner)
        .collect::<Vec<_>>();
    plan.roof_assemblies.clear();
    plan.resolved_geometry.solids.retain(|solid| {
        !removed.contains(&solid.id)
            && !roof_owners.contains(&solid.owner)
            && !matches!(
                solid.role,
                SolidRole::RoofFraming
                    | SolidRole::RoofPlate
                    | SolidRole::RoofFlashing
                    | SolidRole::RoofGutter
                    | SolidRole::RoofEdgeTreatment
            )
    });
    plan.workplace
        .as_mut()
        .unwrap()
        .parts
        .retain(|part| !removed.contains(&part.solid));
    plan.wall_assemblies
        .retain(|wall| !wall.host_solids.iter().any(|id| removed.contains(id)));
}
