//! GPU street-level review of the same sign geometry and lettering used by the tactical client.
use adventuresim_building_generator::{signs::*, *};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::WindowResolution,
};
use clap::{Parser, ValueEnum};
use std::{fs, path::PathBuf};

#[path = "shop-sign-viewer/capture.rs"]
mod capture;
#[path = "workplace-viewer/materials.rs"]
mod materials;
const SETTLE_FRAMES: u32 = 240;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ReviewScene {
    Front,
    AlongStreet,
    Reverse,
    LongName,
}
#[derive(Parser)]
struct Args {
    #[arg(long)]
    output: PathBuf,
    #[arg(long,value_enum,default_value_t=SignFont::GrenzeGotisch)]
    font: SignFont,
    #[arg(long,value_enum,default_value_t=SignMount::Projecting)]
    mount: SignMount,
    #[arg(long,value_enum,default_value_t=ReviewScene::Front)]
    scene: ReviewScene,
}

#[derive(Resource)]
struct Capture {
    output: PathBuf,
    frames: u32,
    in_flight: bool,
    primed: bool,
}

struct ReviewShop {
    program: BuildingProgram,
    plan: BuildingPlan,
    site: SignSite,
    sign: ShopSign,
    offset: Vec3,
}

fn main() {
    let args = Args::parse();
    fs::create_dir_all(args.output.parent().unwrap()).unwrap();
    let shops = review_shops(&args);
    let metadata=shops.iter().map(|shop|serde_json::json!({
        "name":shop.sign.name.text(),"font":format!("{:?}",shop.sign.font),"mount":format!("{:?}",shop.sign.mount),
        "program":shop.program,"audit":audit_plan(&shop.plan),"clearance_metres":shop.site.board(shop.sign.mount).centre.y-shop.site.panel_size.y*0.5,
        "site_clear":shop.site.supports(&shop.plan,shop.sign.mount)
    })).collect::<Vec<_>>();
    fs::write(
        args.output.with_extension("json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    let title = format!("{:?} | {:?} | {:?}", args.font, args.mount, args.scene);
    let scene = args.scene;
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.clone(),
                visible: false,
                resolution: WindowResolution::new(1440, 1000),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.63, 0.71, 0.76)))
        .insert_resource(Capture {
            output: args.output,
            frames: 0,
            in_flight: false,
            primed: false,
        })
        .add_systems(Startup, move |world: &mut World| {
            setup(world, &shops, scene, &title)
        })
        .add_systems(Update, capture::capture)
        .run();
}

fn review_shops(args: &Args) -> Vec<ReviewShop> {
    let uses = if matches!(args.scene, ReviewScene::AlongStreet) {
        vec![
            BuildingUse::Inn,
            BuildingUse::Bakehouse,
            BuildingUse::Smithy,
        ]
    } else {
        vec![BuildingUse::Inn]
    };
    uses.into_iter()
        .enumerate()
        .map(|(index, usage)| {
            let program =
                BuildingProgram::validated_settlement(settlement_archetype(usage), usage, 42, None)
                    .unwrap();
            let plan = generate(&program).unwrap();
            let site = SignSite::for_plan(&plan).expect("review shop has an entrance sign site");
            let mut sign =
                ShopSign::for_establishment(EstablishmentId(15 + index as u64), usage).unwrap();
            sign.font = args.font;
            sign.mount = args.mount;
            sign.name = ShopName {
                proprietor: match index {
                    0 => "Heinrich’s",
                    1 => "Margarete’s",
                    _ => "Konrad’s",
                }
                .to_owned(),
                trade: shop_trade(usage).unwrap().to_owned(),
            };
            if matches!(args.scene, ReviewScene::LongName) {
                sign.name.proprietor = "Margarete Großmüller’s".to_owned();
            }
            assert!(
                site.supports(&plan, sign.mount),
                "{:?} sign overlaps its shop",
                sign.mount
            );
            ReviewShop {
                program,
                plan,
                site,
                sign,
                offset: Vec3::new(index as f32 * 20.0, 0.0, 0.0),
            }
        })
        .collect()
}

fn setup(world: &mut World, shops: &[ReviewShop], scene: ReviewScene, title: &str) {
    let mut cache = ShopSignRenderCache::default();
    for shop in shops {
        for batch in compile_building_detail(&shop.plan).meshes {
            let mesh = world
                .resource_mut::<Assets<Mesh>>()
                .add(capture::mesh(&batch));
            let material = materials::material(world, batch.material);
            world.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(shop.offset),
            ));
        }
        let parts = world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world, mut materials: Mut<Assets<StandardMaterial>>| {
                cache.compile(
                    &shop.sign,
                    shop.site,
                    -shop.offset,
                    SignDetail::Complete,
                    SignRenderAssets {
                        meshes: &mut meshes,
                        materials: &mut materials,
                        images: &mut world.resource_mut::<Assets<Image>>(),
                    },
                )
            })
        });
        for part in parts {
            let visibility = part.visibility();
            world.spawn((
                Mesh3d(part.mesh),
                MeshMaterial3d(part.material),
                part.transform,
                visibility,
            ));
        }
    }
    let ground = world
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(160.0, 0.1, 100.0));
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.31, 0.28, 0.22),
            perceptual_roughness: 1.0,
            ..default()
        });
    world.spawn((
        Mesh3d(ground),
        MeshMaterial3d(material),
        Transform::from_xyz(20.0, -0.16, 0.0),
    ));
    let board = shops[0].site.board(shops[0].sign.mount);
    let outward = shops[0].site.outward;
    let side = Vec3::Y.cross(outward);
    let (camera, focus) = match scene {
        ReviewScene::AlongStreet => (
            board.centre + outward * 5.0 + side * 7.0 - Vec3::Y * 0.65,
            board.centre + Vec3::X * 25.0,
        ),
        ReviewScene::Reverse => (
            board.centre + outward * 3.0 - side * 4.5 - Vec3::Y * 0.8,
            board.centre,
        ),
        _ => (
            board.centre + outward * 4.5 + side * 3.0 - Vec3::Y * 0.8,
            board.centre,
        ),
    };
    world.spawn((
        Camera3d::default(),
        Transform::from_translation(camera).looking_at(focus, Vec3::Y),
    ));
    world.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(board.centre + outward * 20.0 + Vec3::Y * 30.0)
            .looking_at(board.centre, Vec3::Y),
    ));
    world.spawn((
        PointLight {
            intensity: 500_000.0,
            range: 30.0,
            ..default()
        },
        Transform::from_translation(camera + Vec3::Y * 4.0),
    ));
    world.spawn((
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(25.0),
            top: Val::Px(20.0),
            ..default()
        },
    ));
}
