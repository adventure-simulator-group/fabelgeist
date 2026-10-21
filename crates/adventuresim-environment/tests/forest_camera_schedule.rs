#![cfg(feature = "viewer")]

//! Exercise the actual forest plugins and Bevy schedules, including the first
//! update after a camera or viewport change. This is not a timing benchmark.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use adventuresim_environment::{
    prepared::*,
    render::{
        ForestDetail, ForestGeometry, ForestLodView, ForestViewCamera, Level, LevelSelection,
        trees::{TreeFeed, TreeForestPlugin},
    },
};
use bevy::{
    app::PluginsState,
    camera::{CameraProjection, RenderTarget, Viewport},
    prelude::*,
    render::render_resource::TextureFormat,
    window::ExitCondition,
    winit::WinitPlugin,
};
use bevy_eidolon::prelude::InstanceMaterialData;

mod support;

#[test]
fn movement_and_viewport_changes_select_bindings_in_their_first_propagated_frame() {
    let unique = support::padded_fixture();
    let library = unique.bake(VariantBudget::AllPlants).unwrap();
    let (bank_bytes, bank_id) = encode_library(&library).unwrap();
    let population = unique.population(&library, bank_id).unwrap();
    let population_bytes = encode_population(&population, &library, bank_id).unwrap();
    let bank = Arc::new(TreeBank::from_bytes(&bank_bytes).unwrap());
    let forest = Arc::new(SharedForest::from_population_bytes(bank, &population_bytes).unwrap());
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
    );
    app.add_plugins(TreeForestPlugin);
    let started = Instant::now();
    while app.plugins_state() == PluginsState::Adding {
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "GPU plugin deadline"
        );
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1600,
            1000,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            ForestViewCamera,
            RenderTarget::Image(target.into()),
            Transform::from_xyz(0.0, 3.0, 10_000.0),
            Projection::Perspective(PerspectiveProjection {
                fov: 80.0_f32.to_radians(),
                far: 60_000.0,
                ..default()
            }),
        ))
        .id();
    app.world_mut().resource_mut::<TreeFeed>().show(forest);
    app.world_mut().resource_mut::<ForestDetail>().level = LevelSelection::Fixed(Level::Full);
    app.update();
    let full = bindings(app.world_mut());
    assert!(
        !full.is_empty(),
        "fixture produced no actual shared batches"
    );
    app.world_mut().resource_mut::<ForestDetail>().level = LevelSelection::Adaptive;
    app.update();
    let far = bindings(app.world_mut());
    assert_eq!(full.len(), far.len());
    assert!(
        full.iter()
            .all(|(entity, mesh)| far.get(entity) != Some(mesh))
    );

    // GlobalTransform still carries the far pose here. One ordinary update
    // must propagate the new local pose before the production LOD selectors.
    *app.world_mut().get_mut::<Transform>(camera).unwrap() = Transform::from_xyz(0.0, 3.0, 0.0);
    app.update();
    assert_eq!(
        bindings(app.world_mut()),
        full,
        "camera movement selected an older frame's meshes"
    );
    assert_source(&app, camera, UVec2::new(1600, 1000));

    app.world_mut().get_mut::<Camera>(camera).unwrap().viewport = Some(Viewport {
        physical_size: UVec2::new(800, 600),
        ..default()
    });
    let Projection::Perspective(mut projection) =
        app.world().get::<Projection>(camera).unwrap().clone()
    else {
        panic!("fixture camera is not perspective");
    };
    projection.fov = 60.0_f32.to_radians();
    app.world_mut()
        .entity_mut(camera)
        .insert(Projection::Perspective(projection));
    app.update();
    assert_source(&app, camera, UVec2::new(800, 600));

    // A second marked active camera is rejected instead of choosing an
    // arbitrary view; deactivation restores the original source that frame.
    let competing = app
        .world_mut()
        .spawn((Camera3d::default(), ForestViewCamera))
        .id();
    app.update();
    assert!(app.world().resource::<ForestLodView>().camera().is_none());
    app.world_mut()
        .get_mut::<Camera>(competing)
        .unwrap()
        .is_active = false;
    app.update();
    assert_source(&app, camera, UVec2::new(800, 600));
}

fn bindings(world: &mut World) -> HashMap<Entity, AssetId<Mesh>> {
    let mut query = world
        .query_filtered::<(Entity, &Mesh3d), (With<ForestGeometry>, With<InstanceMaterialData>)>();
    query
        .iter(world)
        .map(|(entity, mesh)| (entity, mesh.id()))
        .collect()
}

fn assert_source(app: &App, camera: Entity, viewport: UVec2) {
    let source = app.world().resource::<ForestLodView>().camera().unwrap();
    assert_eq!(source.entity, camera);
    assert_eq!(source.physical_viewport, viewport);
    assert_eq!(
        source.world_from_view.to_matrix(),
        app.world()
            .get::<GlobalTransform>(camera)
            .unwrap()
            .to_matrix()
    );
    let Projection::Perspective(selected) = &source.projection else {
        panic!("invalid selection projection")
    };
    let Projection::Perspective(actual) = app.world().get::<Projection>(camera).unwrap() else {
        panic!("invalid camera projection")
    };
    assert_eq!(selected.get_clip_from_view(), actual.get_clip_from_view());
}
