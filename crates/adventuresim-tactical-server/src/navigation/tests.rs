//! Exercise destination admission against live colliders.

use super::*;
use bevy::ecs::system::SystemState;

fn scene() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()));
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    app.finish();
    app.cleanup();
    app
}

fn point(x: f32, y: f32, z: f32) -> Position<Scene> {
    Position::from_metres(Vec3::new(x, y, z)).unwrap()
}

fn inspect<T>(app: &mut App, run: impl FnOnce(&NavigationWorld) -> T) -> T {
    for _ in 0..3 {
        app.update();
    }
    let mut queries = SystemState::<SpatialQuery>::new(app.world_mut());
    let spatial = queries.get(app.world()).unwrap();
    let config = TacticalCombatConfig::default();
    let world = NavigationWorld {
        spatial: &spatial,
        filter: SpatialQueryFilter::default(),
        config: &config,
        dimensions: CharacterDimensions::default(),
    };
    run(&world)
}

#[test]
fn overlapping_floors_do_not_connect_without_stairs() {
    let mut app = scene();
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(8.0, 0.5, 8.0),
        Transform::from_xyz(0.0, 2.75, 0.0),
    ));
    inspect(&mut app, |world| {
        let ground = world.support_at(point(0.0, 0.0, 0.0)).unwrap();
        let upper = world.support_at(point(0.0, 3.0, 0.0)).unwrap();
        assert_ne!(ground.support, upper.support);
    });
}

#[test]
fn destination_rejects_walls_and_air() {
    let mut app = scene();
    inspect(&mut app, |world| {
        assert_eq!(
            world.admit(NavigationDestination {
                position: point(0.0, 4.0, 0.0)
            }),
            Err(ControlRejection::InvalidDestination)
        );
        assert_eq!(
            world.admit(NavigationDestination {
                position: point(40.0, 0.0, 0.0)
            }),
            Err(ControlRejection::InvalidDestination)
        );
    });
}
