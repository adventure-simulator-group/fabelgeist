//! Exercise the retained controller's physical-face check at both contact edges.
use avian3d::{character_controller::move_and_slide::MoveHitData, prelude::*};
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_ahoy::CharacterController;

#[path = "../../../vendor/bevy_ahoy/src/kcc/stair.rs"]
mod stair;

#[test]
fn physical_bearing_query_covers_both_sides_of_the_existing_contact_skin() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()));
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(4.0, 0.2, 4.0),
            Transform::from_xyz(0.0, -0.1, 0.0),
        ))
        .id();
    app.finish();
    app.cleanup();
    app.update();
    app.update();
    let mut state = SystemState::<MoveAndSlide>::new(app.world_mut());
    let query = state.get(app.world()).unwrap();
    let config = CharacterController::default();
    let corner = |height| MoveHitData {
        entity,
        distance: 0.2,
        collision_distance: config.step_size,
        point1: Vec3::new(0.0, height, 0.0),
        point2: Vec3::new(0.0, height, 0.0),
        normal1: Vec3::new(0.0, 0.5, 0.8660254),
        normal2: Vec3::new(0.0, -0.5, -0.8660254),
    };
    // The same physical face lies a few micrometres below or above the shape
    // cast's rounded contact point. Neither sign may lose valid support.
    for height in [-0.000004, 0.000004] {
        assert!(stair::has_walkable_support(
            corner(height),
            Vec3::NEG_Z,
            &query,
            &config
        ));
    }
    for height in [-0.025, 0.025] {
        assert!(!stair::has_walkable_support(
            corner(height),
            Vec3::NEG_Z,
            &query,
            &config
        ));
    }
}
