use super::*;

#[derive(Resource, Default)]
struct Arrivals(Vec<ReachedTalkTarget>);

fn scene() -> (App, Entity, Entity) {
    let (mut app, actor) = super::super::movement_tests::movement_scene(false);
    app.init_resource::<Arrivals>().add_systems(
        PostUpdate,
        |mut events: MessageReader<ReachedTalkTarget>, mut arrivals: ResMut<Arrivals>| {
            arrivals.0.extend(events.read().copied());
        },
    );
    let target = app
        .world_mut()
        .spawn((
            Player {
                name: "Merchant".into(),
            },
            ResidentActor,
            TacticalCombatSide::Neutral,
            TacticalCombatState::default(),
            crate::player_projection::player_collider(),
            tactical_character_controller(),
            Transform::from_xyz(6.0, 0.95, 0.0),
            CharacterLook::default(),
            AuthoritativeMovementIntent::default(),
            AuthoritativePostureIntent::default(),
            QuickstepPush::default(),
            MovementPace::default(),
        ))
        .id();
    for _ in 0..3 {
        app.update();
    }
    (app, actor, target)
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
fn approach_tracks_a_moving_resident_and_emits_one_fenced_arrival_without_combat() {
    let (mut app, actor, target) = scene();
    request(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Talk { target }),
    );
    for _ in 0..10 {
        app.update();
    }
    app.world_mut()
        .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
        .unwrap()
        .0
        .z = 2.0;
    for _ in 0..900 {
        app.update();
        assert_eq!(app.world().get::<AgentState>(actor).unwrap().target(), None);
        if app.world().get::<CharacterOrder>(actor).unwrap().status == OrderStatus::Completed {
            break;
        }
    }
    let order = *app.world().get::<CharacterOrder>(actor).unwrap();
    assert_eq!(order.status, OrderStatus::Completed);
    assert_eq!(order.goal, CharacterGoal::Talk { target });
    let position = app
        .world()
        .get::<adventuresim_tactical_core::avian3d::prelude::Position>(actor)
        .unwrap()
        .0;
    let destination = app
        .world()
        .get::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
        .unwrap()
        .0;
    assert_eq!(
        destination.z, 2.0,
        "The resident's physical body must actually move"
    );
    assert!(
        position.x > 1.0,
        "the physical motor must actually approach"
    );
    assert!(
        position.distance(destination)
            <= app
                .world()
                .resource::<TacticalCombatConfig>()
                .ai
                .control
                .talk_distance_metres
                .metres()
    );
    for _ in 0..30 {
        app.update();
    }
    let arrivals = &app.world().resource::<Arrivals>().0;
    assert_eq!(arrivals.len(), 1);
    let arrival = arrivals[0];
    assert_eq!(arrival.actor, actor);
    assert_eq!(arrival.target, target);
    assert_eq!(arrival.sequence, order.sequence);
    assert_eq!(
        arrival.generation,
        app.world()
            .get::<CharacterControl>(actor)
            .unwrap()
            .generation
    );
}

#[test]
fn stopped_or_replaced_approaches_cannot_emit_an_arrival() {
    for replacement in [
        ControlChange::Stop,
        ControlChange::SetMode(ControlMode::Direct),
        ControlChange::Order(CharacterGoal::MoveTo(NavigationDestination {
            position: Position::from_metres(Vec3::ZERO).unwrap(),
        })),
    ] {
        let (mut app, actor, target) = scene();
        request(
            &mut app,
            actor,
            ControlChange::Order(CharacterGoal::Talk { target }),
        );
        app.update();
        request(&mut app, actor, replacement);
        for _ in 0..30 {
            app.update();
        }
        assert!(app.world().resource::<Arrivals>().0.is_empty());
        assert!(app.world().get::<TalkApproach>(actor).is_none());
    }
}

#[test]
fn lost_or_newly_hostile_targets_fail_the_approach_without_attacking() {
    for removed in [false, true] {
        let (mut app, actor, target) = scene();
        request(
            &mut app,
            actor,
            ControlChange::Order(CharacterGoal::Talk { target }),
        );
        app.update();
        if removed {
            app.world_mut().despawn(target);
        } else {
            app.world_mut()
                .entity_mut(target)
                .insert(TacticalCombatSide::Enemy);
        }
        app.update();
        assert_eq!(
            app.world().get::<CharacterOrder>(actor).unwrap().status,
            OrderStatus::Failed(ControlRejection::InvalidTarget)
        );
        assert_eq!(app.world().get::<AgentState>(actor).unwrap().target(), None);
        assert!(app.world().resource::<Arrivals>().0.is_empty());
    }
}

#[test]
fn a_counter_allows_head_height_contact_but_a_wall_does_not() {
    for height in [1.1, 3.0] {
        let (mut app, actor, target) = scene();
        app.world_mut()
            .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
            .unwrap()
            .0
            .x = 1.6;
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.2, height, 3.0),
            Transform::from_xyz(0.8, height * 0.5, 0.0),
        ));
        for _ in 0..3 {
            app.update();
        }
        request(
            &mut app,
            actor,
            ControlChange::Order(CharacterGoal::Talk { target }),
        );
        app.update();
        assert_eq!(
            app.world().resource::<Arrivals>().0.len(),
            usize::from(height < 1.9)
        );
    }
}
