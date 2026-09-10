use std::time::Duration;

use adventuresim_tactical_core::{
    avian3d::collision::collider::contact_query,
    prelude::{PhysicsPlugins, RigidBody},
};
use bevy::{ecs::system::SystemState, time::TimeUpdateStrategy};

use super::*;

struct Obstacle {
    position: Vec3,
    shape: Collider,
}

impl Obstacle {
    fn cuboid(position: Vec3, size: Vec3) -> Self {
        Self {
            position,
            shape: Collider::cuboid(size.x, size.y, size.z),
        }
    }
}

fn scene(obstacles: &[Obstacle]) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        1.0 / 60.0,
    )));
    for obstacle in obstacles {
        app.world_mut().spawn((
            RigidBody::Static,
            obstacle.shape.clone(),
            Transform::from_translation(obstacle.position),
        ));
    }
    app.finish();
    app.cleanup();
    for _ in 0..3 {
        app.update();
    }
    app
}

fn frame() -> CameraFrame {
    CameraFrame {
        origin: Vec3::ZERO,
        anchor: Vec3::Y * 0.5,
        focus: Vec3::Y * 0.5,
        rotation: Quat::IDENTITY,
        shoulder_offset: 0.5,
        distance: 2.75,
    }
}

fn projection() -> Projection {
    Projection::Perspective(PerspectiveProjection {
        fov: 80.0_f32.to_radians(),
        aspect_ratio: 16.0 / 9.0,
        near: 0.1,
        ..default()
    })
}

fn place(
    app: &mut App,
    boom: &mut BoomRecovery,
    frame: &CameraFrame,
    projection: &Projection,
) -> CameraPlacement {
    let config = CameraRigConfig::default();
    let mut system = SystemState::<SpatialQuery>::new(app.world_mut());
    let spatial = system.get(app.world()).unwrap();
    boom.place(
        frame,
        &CameraVolume::new(projection, config.collision_margin),
        &spatial,
        &SpatialQueryFilter::default(),
        &|_| true,
        &config,
        1.0 / 60.0,
    )
}

fn assert_clear(position: Vec3, rotation: Quat, projection: &Projection, obstacles: &[Obstacle]) {
    let volume = CameraVolume::new(projection, CameraRigConfig::default().collision_margin);
    for obstacle in obstacles {
        let contact = contact_query::contact(
            &volume.shape,
            position + rotation * volume.local_center,
            rotation,
            &obstacle.shape,
            obstacle.position,
            Quat::IDENTITY,
            0.0,
        )
        .unwrap();
        assert!(
            contact.is_none_or(|contact| contact.penetration <= 0.0001),
            "camera at {position:?} penetrated obstacle at {:?}: {contact:?}",
            obstacle.position
        );
    }
}

#[test]
fn close_view_clears_wall_and_keeps_subject_beside_manual_aim() {
    let obstacles = [Obstacle::cuboid(
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(8.0, 4.0, 0.2),
    )];
    let mut app = scene(&obstacles);
    let frame = frame();
    let projection = projection();
    let mut boom = BoomRecovery::default();
    boom.reset(frame.distance);
    let placement = place(&mut app, &mut boom, &frame, &projection);
    assert!(placement.position.z < 0.9 && placement.position.z > 0.5);
    assert!(
        placement.position.x > 0.25,
        "close framing must retain a shoulder view"
    );
    assert!(placement.position.y > frame.anchor.y);
    assert_clear(placement.position, frame.rotation, &projection, &obstacles);
}

#[test]
fn final_close_offset_cannot_enter_an_inside_corner() {
    let obstacles = [
        Obstacle::cuboid(Vec3::new(0.0, 0.0, 1.2), Vec3::new(6.0, 4.0, 0.2)),
        Obstacle::cuboid(Vec3::new(0.38, 0.5, 0.65), Vec3::new(0.16, 2.0, 0.2)),
    ];
    let mut app = scene(&obstacles);
    let projection = projection();
    let mut frame = frame();
    // The exploration camera starts centered, then close framing moves right.
    frame.shoulder_offset = 0.0;
    let mut boom = BoomRecovery::default();
    boom.reset(frame.distance);
    let mut previous = None;
    for step in 0..90 {
        let placement = place(&mut app, &mut boom, &frame, &projection);
        assert_clear(placement.position, frame.rotation, &projection, &obstacles);
        if step > 60 {
            assert!(
                placement.position.distance(previous.unwrap()) < 0.001,
                "a stationary corner must settle instead of pumping the view"
            );
        }
        previous = Some(placement.position);
    }
}

#[test]
fn focus_lag_on_the_other_side_of_a_wall_cannot_move_camera_through_it() {
    let obstacles = [Obstacle::cuboid(
        Vec3::new(0.6, 0.0, 0.0),
        Vec3::new(0.2, 4.0, 8.0),
    )];
    let mut app = scene(&obstacles);
    let mut frame = frame();
    frame.focus.x = 0.85;
    let projection = projection();
    let mut boom = BoomRecovery::default();
    boom.reset(frame.distance);
    let placement = place(&mut app, &mut boom, &frame, &projection);
    assert!(placement.position.x < 0.5);
    assert_clear(placement.position, frame.rotation, &projection, &obstacles);
}

#[test]
fn near_plane_clears_doorway_ceiling_and_stairs_during_manual_orbit() {
    let obstacles = [
        Obstacle::cuboid(Vec3::new(-1.0, 0.0, 1.5), Vec3::new(1.0, 4.0, 0.2)),
        Obstacle::cuboid(Vec3::new(1.0, 0.0, 1.5), Vec3::new(1.0, 4.0, 0.2)),
        Obstacle::cuboid(Vec3::new(0.0, 1.0, 1.5), Vec3::new(6.0, 0.2, 6.0)),
        Obstacle::cuboid(Vec3::new(0.0, -0.6, 1.0), Vec3::new(4.0, 0.2, 0.6)),
        Obstacle::cuboid(Vec3::new(0.0, -0.4, 1.6), Vec3::new(4.0, 0.2, 0.6)),
    ];
    let mut app = scene(&obstacles);
    for aspect_ratio in [4.0 / 3.0, 16.0 / 9.0, 32.0 / 9.0] {
        let projection = Projection::Perspective(PerspectiveProjection {
            fov: 100.0_f32.to_radians(),
            aspect_ratio,
            near: 0.15,
            ..default()
        });
        let mut boom = BoomRecovery::default();
        boom.reset(2.75);
        for yaw in -12..=12 {
            for pitch in [-0.65, 0.0, 0.65] {
                let mut frame = frame();
                frame.rotation = Quat::from_euler(EulerRot::YXZ, yaw as f32 * 0.1, pitch, 0.0);
                let placement = place(&mut app, &mut boom, &frame, &projection);
                assert_clear(placement.position, frame.rotation, &projection, &obstacles);
            }
        }
    }
}

#[test]
fn open_space_preserves_full_distance_and_shoulder() {
    let mut app = scene(&[]);
    let frame = frame();
    let mut boom = BoomRecovery::default();
    boom.reset(frame.distance);
    let placement = place(&mut app, &mut boom, &frame, &projection());
    assert!(
        placement
            .position
            .abs_diff_eq(Vec3::new(0.5, 0.5, 2.75), 0.0001)
    );
    assert!(placement.hit.is_none());
}

#[test]
fn brief_doorway_gaps_do_not_pump_distance_and_recovery_is_monotonic() {
    let config = CameraRigConfig::default();
    let mut boom = BoomRecovery::default();
    boom.reset(3.75);
    boom.recover(3.75, 1.2, &config, 1.0 / 60.0);
    assert_eq!(boom.distance, 1.2);
    // The camera may have been pressed against the jamb for several seconds.
    for _ in 0..120 {
        boom.recover(3.75, 1.2, &config, 1.0 / 60.0);
    }
    for _ in 0..4 {
        boom.recover(3.75, 3.75, &config, 1.0 / 60.0);
        assert_eq!(boom.distance, 1.2);
    }
    let mut previous = boom.distance;
    for _ in 0..180 {
        boom.recover(3.75, 3.75, &config, 1.0 / 60.0);
        assert!(boom.distance >= previous && boom.distance <= 3.75);
        previous = boom.distance;
    }
    assert!(boom.distance > 3.74);
    boom.recover(3.75, 0.6, &config, 1.0 / 60.0);
    assert_eq!(
        boom.distance, 0.6,
        "a new obstruction must override recovery immediately"
    );
}
