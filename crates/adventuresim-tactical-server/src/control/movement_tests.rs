//! Exercise orders through the actual fixed-step character motor and physics.

use super::*;
use adventuresim_tactical_core::physics::AdventureSimulatorPhysicsPlugin;
use adventuresim_tactical_core::scene_input::SceneBuildingId;
use adventuresim_tactical_netcode::replication::AdventureSimulatorReplicationPlugin;
use std::time::Duration;

fn movement_scene(wall: bool) -> (App, Entity) {
    let mut app = App::new();
    let tick = Duration::from_secs_f64(1.0 / 30.0);
    app.add_plugins((
        MinimalPlugins,
        bevy::input::InputPlugin,
        TransformPlugin,
        EnhancedInputPlugin,
        AdventureSimulatorPhysicsPlugin::default(),
        AdventureSimulatorReplicationPlugin,
        CharacterControlPlugin,
    ));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(tick));
    app.insert_resource(Time::<Fixed>::from_duration(tick));
    app.add_systems(
        FixedPostUpdate,
        (
            crate::player_projection::restore_authoritative_movement_intent
                .before(AdventureSimulatorPhysicsSet::ApplyCharacterMotor),
            crate::player_projection::update_skeleton_locomotion.after(AhoySystems::MoveCharacters),
        ),
    );
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(30.0, 1.0, 30.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    let start = Vec3::new(0.0, 0.95, 0.0);
    if wall {
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.5, 3.0, 2.0),
            Transform::from_xyz(3.0, 1.5, 0.0),
        ));
    }
    let actor = app
        .world_mut()
        .spawn((
            Player::default(),
            TacticalCombatState::default(),
            TacticalCombatSide::Party,
            Transform::from_translation(start),
            Collider::cylinder(
                adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES,
                1.9,
            ),
            tactical_character_controller(),
            CharacterLook::default(),
            AuthoritativeMovementIntent::default(),
            AuthoritativePostureIntent::default(),
            QuickstepPush::default(),
            MovementPace::default(),
            CharacterControl {
                mode: ControlMode::Indirect,
                ..default()
            },
        ))
        .id();
    app.finish();
    app.cleanup();
    for _ in 0..5 {
        app.update();
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while !app
        .world()
        .resource::<crate::navigation::TacticalNavigation>()
        .ready
    {
        assert!(
            std::time::Instant::now() < deadline,
            "navigation mesh was not generated"
        );
        app.update();
        std::thread::yield_now();
    }
    let archipelago = app
        .world()
        .resource::<crate::navigation::TacticalNavigation>()
        .archipelago
        .unwrap();
    let archipelago = app
        .world()
        .get::<bevy_landmass::Archipelago3d>(archipelago)
        .unwrap();
    let feet = app.world().get::<Transform>(actor).unwrap().translation - Vec3::Y * 0.95;
    assert!(
        archipelago
            .sample_point(feet, &archipelago.get_agent_options().point_sample_distance)
            .is_ok(),
        "the generated mesh does not contain the starting feet position {feet:?}"
    );
    (app, actor)
}

fn walk_to(app: &mut App, actor: Entity, destination: Vec3) -> f32 {
    let start = app.world().get::<Transform>(actor).unwrap().translation;
    let control = *app.world().get::<CharacterControl>(actor).unwrap();
    let sequence = app
        .world()
        .get::<CharacterOrder>(actor)
        .unwrap()
        .sequence
        .next();
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Client(actor),
        message: ControlRequest {
            generation: control.generation,
            sequence,
            change: ControlChange::Order(CharacterGoal::MoveTo(NavigationDestination {
                position: Position::from_metres(destination).unwrap(),
            })),
        },
    });
    let mut max_deviation = 0.0_f32;
    for _ in 0..900 {
        app.update();
        let position = app.world().get::<Transform>(actor).unwrap().translation;
        let displacement = position.xz() - start.xz();
        let direction = (destination - start).xz().normalize();
        max_deviation =
            max_deviation.max((displacement.x * direction.y - displacement.y * direction.x).abs());
        if matches!(
            app.world().get::<CharacterOrder>(actor).unwrap().status,
            OrderStatus::Completed | OrderStatus::Failed(_)
        ) {
            break;
        }
    }
    let order = app.world().get::<CharacterOrder>(actor).unwrap();
    let position = app.world().get::<Transform>(actor).unwrap().translation;
    let remaining = position.xz().distance(destination.xz());
    let radius = app
        .world()
        .resource::<TacticalCombatConfig>()
        .ai
        .control
        .arrival_radius_metres
        .metres();
    assert_eq!(
        order.status,
        OrderStatus::Completed,
        "position {position:?}, remaining {remaining}, deviation {max_deviation}"
    );
    assert!(
        remaining <= radius,
        "position {position:?}, remaining {remaining}, radius {radius}"
    );
    // Let the motor brake after the order reports completion, then inspect the
    // actual resting position rather than only its completion frame.
    for _ in 0..30 {
        app.update();
    }
    let resting = app.world().get::<Transform>(actor).unwrap().translation;
    assert!(
        resting.xz().distance(destination.xz()) <= radius + 0.3,
        "resting position {resting:?} is too far from {destination:?}"
    );
    max_deviation
}

#[test]
fn clear_orders_reach_destinations_without_large_detours() {
    let (mut app, actor) = movement_scene(false);
    for destination in [
        Vec3::new(6.0, 0.0, 4.0),
        Vec3::new(3.0, 0.0, -3.0),
        Vec3::new(1.5, 0.0, -2.0),
    ] {
        let deviation = walk_to(&mut app, actor, destination);
        assert!(deviation < 0.75, "clear route deviated {deviation} metres");
    }
}

#[test]
fn movement_around_wall_reaches_the_destination() {
    let (mut app, actor) = movement_scene(true);
    walk_to(&mut app, actor, Vec3::new(6.0, 0.0, 0.0));
}

#[test]
fn newly_added_wall_rebuilds_the_mesh_before_travel() {
    let (mut app, actor) = movement_scene(false);
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.5, 3.0, 2.0),
        Transform::from_xyz(3.0, 1.5, 0.0),
    ));
    wait_for_rebuild(&mut app);
    walk_to(&mut app, actor, Vec3::new(6.0, 0.0, 0.0));
}

fn wait_for_rebuild(app: &mut App) {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut rebuilding = false;
    loop {
        app.update();
        let ready = app
            .world()
            .resource::<crate::navigation::TacticalNavigation>()
            .ready;
        rebuilding |= !ready;
        if rebuilding && ready {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "changed geometry did not rebuild the mesh"
        );
        std::thread::yield_now();
    }
}

#[test]
fn disconnected_upper_floor_is_not_selected_as_a_route() {
    let (mut app, actor) = movement_scene(false);
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(8.0, 0.5, 8.0),
        Transform::from_xyz(0.0, 2.75, 0.0),
    ));
    wait_for_rebuild(&mut app);
    let control = *app.world().get::<CharacterControl>(actor).unwrap();
    let sequence = app
        .world()
        .get::<CharacterOrder>(actor)
        .unwrap()
        .sequence
        .next();
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Client(actor),
        message: ControlRequest {
            generation: control.generation,
            sequence,
            change: ControlChange::Order(CharacterGoal::MoveTo(NavigationDestination {
                position: Position::from_metres(Vec3::new(0.0, 3.0, 0.0)).unwrap(),
            })),
        },
    });
    for _ in 0..30 {
        app.update();
        if matches!(
            app.world().get::<CharacterOrder>(actor).unwrap().status,
            OrderStatus::Failed(_)
        ) {
            break;
        }
    }
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().status,
        OrderStatus::Failed(ControlRejection::Unreachable)
    );
}

#[test]
fn moved_door_rebuilds_the_mesh_and_routes_around_its_new_position() {
    use adventuresim_building_generator::{
        OpeningAssemblyId,
        spatial_geometry::{LeafDimensions, SpatialDirection},
    };
    let (mut app, actor) = movement_scene(false);
    let size = Vec3::new(0.5, 3.0, 2.0);
    let door = app
        .world_mut()
        .spawn((
            SceneDoor {
                building_id: SceneBuildingId(1),
                opening_id: OpeningAssemblyId(1),
                size_metres: LeafDimensions::from_metres(size).unwrap(),
                doorway_centre_metres: Position::from_metres(Vec3::new(3.0, 1.5, 4.0)).unwrap(),
                tangent: SpatialDirection::from_normalized(Vec3::X).unwrap(),
                outward: SpatialDirection::from_normalized(Vec3::Z).unwrap(),
            },
            RigidBody::Kinematic,
            Collider::cuboid(size.x, size.y, size.z),
            Transform::from_xyz(3.0, 1.5, 4.0),
        ))
        .id();
    wait_for_rebuild(&mut app);
    app.world_mut()
        .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(door)
        .unwrap()
        .0
        .z = 0.0;
    wait_for_rebuild(&mut app);
    walk_to(&mut app, actor, Vec3::new(6.0, 0.0, 0.0));
}
