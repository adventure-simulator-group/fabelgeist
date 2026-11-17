//! The default generated terrain must produce a usable server navigation mesh.

use crate::navigation::{TacticalNavigation, TacticalNavigationPlugin};
use adventuresim_tactical_core::{prelude::*, scene_coordinates::ScenePlanPoint};
use bevy::prelude::*;
use std::time::{Duration, Instant};

#[test]
fn dense_woodland_terrain_generates_a_navigable_surface() {
    let input = TacticalSceneInput::load(&crate::startup::default_scene_input_path()).unwrap();
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
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        PhysicsPlugins::default(),
        TacticalNavigationPlugin,
    ));
    app.init_resource::<TacticalCombatConfig>();
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
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
            "dense woodland navigation mesh did not become ready"
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
    assert!(
        archipelago
            .sample_point(
                Vec3::new(0.0, elevation, 0.0),
                &archipelago.get_agent_options().point_sample_distance
            )
            .is_ok()
    );
}
