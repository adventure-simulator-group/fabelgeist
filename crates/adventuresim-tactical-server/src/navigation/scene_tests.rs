//! The default generated terrain must produce a usable server navigation mesh.

use crate::navigation::{TacticalNavigation, TacticalNavigationPlugin};
use adventuresim_tactical_core::{prelude::*, scene_coordinates::ScenePlanPoint};
use bevy::prelude::*;
use std::time::{Duration, Instant};

#[derive(Resource, Default)]
struct GeneratedBounds(Option<bevy::math::bounding::Aabb3d>);

#[test]
fn dense_woodland_terrain_generates_a_navigable_surface() {
    let input =
        TacticalSceneInput::load(&crate::startup::default_scene_input_path().unwrap()).unwrap();
    let generated = input.generate().unwrap();
    let elevation = generated
        .terrain
        .height_at(ScenePlanPoint::ORIGIN)
        .unwrap()
        .metres();
    let colliders = crate::terrain_collision::colliders(
        &generated.terrain,
        input.landform.as_ref(),
        generated.terrain_patch.as_ref(),
    )
    .unwrap();
    assert_scene_navigation(generated.terrain, colliders, Vec3::new(0.0, elevation, 0.0));
}

#[test]
fn a_geographic_horizon_does_not_expand_the_playable_navigation_grid() {
    let terrain = SceneTerrain::from_heightmap(101, 101, 1.0, vec![0.0; 101 * 101]).unwrap();
    // The accepted geographic surface may span the 50-km presentation horizon.
    // Its navmesh must still fit the 100-m playable window and remain usable.
    assert_scene_navigation(
        terrain,
        vec![Collider::cuboid(50_000.0, 1.0, 50_000.0)],
        Vec3::new(0.0, 0.5, 0.0),
    );
}

fn assert_scene_navigation(terrain: SceneTerrain, colliders: Vec<Collider>, point: Vec3) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        PhysicsPlugins::default(),
        TacticalNavigationPlugin,
    ));
    app.init_resource::<TacticalCombatConfig>();
    app.init_resource::<GeneratedBounds>();
    app.add_observer(capture_generated_bounds);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    let half_width = terrain.width() * 0.5;
    let half_depth = terrain.depth() * 0.5;
    app.add_systems(Startup, move |mut commands: Commands| {
        crate::scene_setup::spawn_world_bounds(&mut commands, half_width * 2.0, half_depth * 2.0);
    });
    app.world_mut().spawn(terrain);
    for collider in colliders {
        app.world_mut()
            .spawn((RigidBody::Static, collider, Transform::default()));
    }
    app.finish();
    app.cleanup();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !app.world().resource::<TacticalNavigation>().ready {
        assert!(
            Instant::now() < deadline,
            "bounded scene navigation mesh did not become ready"
        );
        app.update();
        std::thread::yield_now();
    }
    let entity = app
        .world()
        .resource::<TacticalNavigation>()
        .archipelago
        .unwrap();
    let archipelago = app
        .world()
        .get::<bevy_landmass::Archipelago3d>(entity)
        .unwrap();
    let bounds = app.world().resource::<GeneratedBounds>().0.unwrap();
    assert!(bounds.min.x >= -half_width);
    assert!(bounds.max.x <= half_width);
    assert!(bounds.min.z >= -half_depth);
    assert!(bounds.max.z <= half_depth);
    assert!(
        archipelago
            .sample_point(
                point,
                &archipelago.get_agent_options().point_sample_distance
            )
            .is_ok()
    );
}

fn capture_generated_bounds(
    event: On<bevy_rerecast::prelude::NavmeshReady>,
    meshes: Res<Assets<bevy_rerecast::Navmesh>>,
    mut bounds: ResMut<GeneratedBounds>,
) {
    // Landmass consumes the generated asset when it installs the island. Read
    // the actual completed request before that conversion removes the asset.
    bounds.0 = meshes.get(event.0).unwrap().settings.aabb;
}
