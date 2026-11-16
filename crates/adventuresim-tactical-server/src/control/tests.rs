//! Behavioral acceptance scenarios exercise the production controller and physics.

use super::*;
use adventuresim_tactical_netcode::replication::AdventureSimulatorReplicationPlugin;
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        PhysicsPlugins::default(),
        AdventureSimulatorReplicationPlugin,
        CharacterControlPlugin,
    ));
    app.init_resource::<TacticalCombatConfig>();
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.add_systems(
        Update,
        crate::bot::maintain_guard_stance.in_set(ControlSet::Execute),
    );
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(30.0, 1.0, 30.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    app.finish();
    app.cleanup();
    app
}

fn wait_for_navigation(app: &mut App) {
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
}

#[test]
fn indirect_guard_tracks_engagement_and_returns_to_travel() {
    let mut app = app();
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.95, -3.0),
    );
    app.world_mut().entity_mut(actor).insert((
        crate::bot::RaisedGuardAi,
        crate::bot::OffensiveCombatAi::default(),
    ));
    app.update();
    assert_eq!(
        app.world()
            .get::<SkeletonState>(actor)
            .unwrap()
            .weapon_guard(),
        WeaponGuardState::Lowered
    );
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Attack { target }),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<SkeletonState>(actor)
            .unwrap()
            .weapon_guard(),
        WeaponGuardState::Raised
    );
    app.world_mut().entity_mut(target).despawn();
    app.update();
    assert_eq!(
        app.world()
            .get::<SkeletonState>(actor)
            .unwrap()
            .weapon_guard(),
        WeaponGuardState::Lowered
    );
}

#[test]
fn combat_pursuit_turns_toward_the_route_before_advancing() {
    let mut app = app();
    wait_for_navigation(&mut app);
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(4.0, 0.95, 0.0),
    );
    app.update();
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Attack { target }),
    );
    app.world_mut()
        .get_mut::<AuthoritativeMovementIntent>(actor)
        .unwrap()
        .0 = Some(Vec2::Y);
    app.update();
    let look = app.world().get::<CharacterLook>(actor).unwrap();
    assert!((Quat::from_rotation_y(look.yaw) * Vec3::NEG_Z).dot(Vec3::X) > 0.99);
    assert_eq!(
        app.world()
            .get::<AuthoritativeMovementIntent>(actor)
            .unwrap()
            .0,
        None
    );
    app.world_mut()
        .get_mut::<Transform>(actor)
        .unwrap()
        .rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    app.world_mut()
        .get_mut::<AuthoritativeMovementIntent>(actor)
        .unwrap()
        .0 = Some(Vec2::Y);
    app.update();
    assert_eq!(
        app.world()
            .get::<AuthoritativeMovementIntent>(actor)
            .unwrap()
            .0,
        Some(Vec2::Y)
    );
}

#[test]
fn pursuit_uses_target_height_to_find_its_support() {
    let mut app = app();
    wait_for_navigation(&mut app);
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(4.0, 1.9, 0.0),
    );
    app.world_mut()
        .get_mut::<CharacterDimensions>(target)
        .unwrap()
        .body_height_metres = 3.8;
    app.update();
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Attack { target }),
    );
    app.world_mut()
        .get_mut::<AuthoritativeMovementIntent>(actor)
        .unwrap()
        .0 = Some(Vec2::Y);
    app.update();
    let look = app.world().get::<CharacterLook>(actor).unwrap();
    assert!((Quat::from_rotation_y(look.yaw) * Vec3::NEG_Z).dot(Vec3::X) > 0.99);
}

fn character(app: &mut App, side: TacticalCombatSide, position: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            Player::default(),
            TacticalCombatState::default(),
            side,
            Transform::from_translation(position),
            CharacterLook::default(),
            AuthoritativeMovementIntent::default(),
            AuthoritativePostureIntent::default(),
            input::AccumulatedInput::default(),
            CharacterControl {
                mode: ControlMode::Indirect,
                ..default()
            },
        ))
        .id()
}

fn request(app: &mut App, actor: Entity, change: ControlChange) {
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
            change,
        },
    });
}

#[test]
fn explicit_attack_keeps_its_target_when_a_nearer_enemy_appears() {
    let mut app = app();
    let actor = character(&mut app, TacticalCombatSide::Party, Vec3::Y);
    let selected = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 1.0, -5.0),
    );
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Attack { target: selected }),
    );
    character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 1.0, -1.0),
    );
    app.update();
    assert_eq!(
        app.world().get::<AgentState>(actor).unwrap().target(),
        Some(selected)
    );
}

#[test]
fn direct_control_cannot_receive_orders_and_old_generations_are_rejected() {
    let mut app = app();
    let actor = character(&mut app, TacticalCombatSide::Party, Vec3::Y);
    let generation = app
        .world()
        .get::<CharacterControl>(actor)
        .unwrap()
        .generation;
    request(&mut app, actor, ControlChange::SetMode(ControlMode::Direct));
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Hold {
            anchor: Position::ORIGIN,
        }),
    );
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().goal,
        CharacterGoal::Idle
    );
    let sequence = app
        .world()
        .get::<CharacterOrder>(actor)
        .unwrap()
        .sequence
        .next();
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Client(actor),
        message: ControlRequest {
            generation,
            sequence,
            change: ControlChange::SetMode(ControlMode::Indirect),
        },
    });
    assert_eq!(
        app.world().get::<CharacterControl>(actor).unwrap().mode,
        ControlMode::Direct
    );
}

#[test]
fn perceived_aggression_temporarily_interrupts_but_retains_move_order() {
    let mut app = app();
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    let aggressor = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 0.95, -1.0),
    );
    app.world_mut().entity_mut(actor).insert((
        crate::bot::RaisedGuardAi,
        crate::bot::OffensiveCombatAi::default(),
    ));
    let destination = NavigationDestination {
        position: Position::from_metres(Vec3::new(4.0, 0.0, 0.0)).unwrap(),
    };
    app.update();
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::MoveTo(destination)),
    );
    app.world_mut().trigger(AcceptedAttack {
        attacker: aggressor,
        target: Some(actor),
        windup: AnimationElapsed::from_seconds(0.5).unwrap(),
    });
    app.update();
    let order = app.world().get::<CharacterOrder>(actor).unwrap();
    assert_eq!(order.goal, CharacterGoal::MoveTo(destination));
    assert_eq!(order.status, OrderStatus::Defending);
    assert_eq!(
        app.world()
            .get::<SkeletonState>(actor)
            .unwrap()
            .weapon_guard(),
        WeaponGuardState::Raised
    );
    assert!(matches!(
        app.world().get::<AgentState>(actor).unwrap().decision,
        BehaviorDecision::Engage {
            pursuit: PursuitPolicy::Forbidden,
            ..
        }
    ));
    app.world_mut().entity_mut(aggressor).despawn();
    app.update();
    assert!(matches!(
        app.world().get::<AgentState>(actor).unwrap().decision,
        BehaviorDecision::Travel(_)
    ));
    assert_eq!(
        app.world()
            .get::<SkeletonState>(actor)
            .unwrap()
            .weapon_guard(),
        WeaponGuardState::Lowered
    );
}

#[test]
fn wall_prevents_perception_and_target_admission() {
    let mut app = app();
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(-2.0, 1.0, 0.0),
    );
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(2.0, 1.0, 0.0),
    );
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.5, 3.0, 6.0),
        Transform::from_xyz(0.0, 1.5, 0.0),
    ));
    for _ in 0..3 {
        app.update();
    }
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Attack { target }),
    );
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().goal,
        CharacterGoal::Idle
    );
    app.world_mut().trigger(AcceptedAttack {
        attacker: target,
        target: Some(actor),
        windup: AnimationElapsed::from_seconds(0.5).unwrap(),
    });
    assert!(
        app.world()
            .get::<ThreatMemory>(actor)
            .unwrap()
            .threat
            .is_none()
    );
}

#[test]
fn stop_clears_pursuit_and_reaction_without_changing_health() {
    let mut app = app();
    let actor = character(&mut app, TacticalCombatSide::Party, Vec3::Y);
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 1.0, -1.0),
    );
    let health = app.world().get::<Limbs>(actor).unwrap().clone();
    app.world_mut().trigger(perception::ReceivedDamage {
        attacker: target,
        defender: actor,
    });
    request(&mut app, actor, ControlChange::Stop);
    assert!(
        app.world()
            .get::<ThreatMemory>(actor)
            .unwrap()
            .threat
            .is_none()
    );
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().goal,
        CharacterGoal::Idle
    );
    assert_eq!(*app.world().get::<Limbs>(actor).unwrap(), health);
}

#[test]
fn sustained_obstruction_reports_no_progress() {
    let mut app = app();
    wait_for_navigation(&mut app);
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs(1),
    ));
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(1));
    app.update();
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::MoveTo(NavigationDestination {
            position: Position::from_metres(Vec3::new(4.0, 0.0, 0.0)).unwrap(),
        })),
    );
    // No motor is installed in this fixture: valid steering cannot move its actor.
    for _ in 0..7 {
        app.update();
    }
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().status,
        OrderStatus::Failed(ControlRejection::NoProgress)
    );
}

#[test]
fn waypoints_execute_in_order_and_replacement_clears_the_queue() {
    let mut app = app();
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    app.update();
    let first = NavigationDestination {
        position: Position::from_metres(Vec3::new(2.0, 0.0, 0.0)).unwrap(),
    };
    let second = NavigationDestination {
        position: Position::from_metres(Vec3::new(4.0, 0.0, 0.0)).unwrap(),
    };
    let third = NavigationDestination {
        position: Position::from_metres(Vec3::new(6.0, 0.0, 0.0)).unwrap(),
    };
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::MoveTo(first)),
    );
    request(&mut app, actor, ControlChange::AppendWaypoint(second));
    request(&mut app, actor, ControlChange::AppendWaypoint(third));
    assert_eq!(
        app.world()
            .get::<WaypointQueue>(actor)
            .unwrap()
            .destinations(),
        &[second, third]
    );
    app.world_mut()
        .get_mut::<Transform>(actor)
        .unwrap()
        .translation = Vec3::new(2.0, 0.95, 0.0);
    app.update();
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().goal,
        CharacterGoal::MoveTo(second)
    );
    assert_eq!(
        app.world()
            .get::<WaypointQueue>(actor)
            .unwrap()
            .destinations(),
        &[third]
    );
    request(&mut app, actor, ControlChange::Stop);
    assert!(
        app.world()
            .get::<WaypointQueue>(actor)
            .unwrap()
            .destinations()
            .is_empty()
    );
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().goal,
        CharacterGoal::Idle
    );
}

#[test]
fn invalid_or_excess_waypoints_do_not_replace_the_active_order() {
    let mut app = app();
    let actor = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(0.0, 0.95, 0.0),
    );
    app.update();
    let destination = NavigationDestination {
        position: Position::from_metres(Vec3::new(2.0, 0.0, 0.0)).unwrap(),
    };
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::MoveTo(destination)),
    );
    let invalid = NavigationDestination {
        position: Position::from_metres(Vec3::new(100.0, 0.0, 0.0)).unwrap(),
    };
    request(&mut app, actor, ControlChange::AppendWaypoint(invalid));
    assert!(
        app.world()
            .get::<WaypointQueue>(actor)
            .unwrap()
            .destinations()
            .is_empty()
    );
    for _ in 0..WaypointQueue::MAX_DESTINATIONS {
        request(&mut app, actor, ControlChange::AppendWaypoint(destination));
    }
    let sequence = app.world().get::<CharacterOrder>(actor).unwrap().sequence;
    request(&mut app, actor, ControlChange::AppendWaypoint(destination));
    assert_eq!(
        app.world()
            .get::<WaypointQueue>(actor)
            .unwrap()
            .destinations()
            .len(),
        WaypointQueue::MAX_DESTINATIONS
    );
    let order = app.world().get::<CharacterOrder>(actor).unwrap();
    assert_eq!(order.sequence, sequence);
    assert_eq!(order.goal, CharacterGoal::MoveTo(destination));
}

#[test]
fn npc_and_player_orders_select_the_same_behavior() {
    let mut app = app();
    let player = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(-1.0, 1.0, 0.0),
    );
    let npc = character(
        &mut app,
        TacticalCombatSide::Party,
        Vec3::new(1.0, 1.0, 0.0),
    );
    let target = character(
        &mut app,
        TacticalCombatSide::Enemy,
        Vec3::new(0.0, 1.0, -4.0),
    );
    request(
        &mut app,
        player,
        ControlChange::Order(CharacterGoal::Attack { target }),
    );
    app.world_mut().get_mut::<CharacterOrder>(npc).unwrap().goal = CharacterGoal::Attack { target };
    app.update();
    assert_eq!(
        app.world().get::<AgentState>(player).unwrap().target(),
        app.world().get::<AgentState>(npc).unwrap().target()
    );
}
