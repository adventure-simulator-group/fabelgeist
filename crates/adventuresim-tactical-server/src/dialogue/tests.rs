use super::*;
use crate::player_projection::{AuthoritativeMovementIntent, AuthoritativePostureIntent};
use adventuresim_tactical_core::control::{
    CharacterControl, CharacterGoal, CharacterOrder, ControlMode, ControlRejection, OrderStatus,
};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::control::{ControlChange, ControlRequest};

fn scene(target_x_metres: f32) -> (App, Entity, Entity) {
    let (mut app, actor) = crate::control::movement_tests::movement_scene(false);
    app.world_mut().entity_mut(actor).insert(CharacterId(1));
    let target = app
        .world_mut()
        .spawn((
            Player {
                name: "Merchant".into(),
            },
            CharacterId(2),
            crate::residents::ResidentActor,
            TacticalCombatSide::Neutral,
            TacticalCombatState::default(),
            crate::player_projection::player_collider(),
            tactical_character_controller(),
            Transform::from_xyz(target_x_metres, 0.95, 0.0),
            CharacterLook::default(),
            AuthoritativeMovementIntent::default(),
            AuthoritativePostureIntent::default(),
            QuickstepPush::default(),
            MovementPace::default(),
        ))
        .id();
    app.insert_resource(crate::local_time::tests::contact_visits(
        CharacterId(1),
        CharacterId(2),
    ));
    for _ in 0..3 {
        app.update();
    }
    (app, actor, target)
}

fn order(app: &mut App, actor: Entity, change: ControlChange) {
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

fn take_calls(app: &mut App) -> Vec<DialogueCall> {
    std::mem::take(&mut app.world_mut().resource_mut::<DialogueOutbox>().0)
}

fn approach(app: &mut App, actor: Entity, target: Entity) -> DialogueCall {
    order(
        app,
        actor,
        ControlChange::Order(CharacterGoal::Talk { target }),
    );
    for _ in 0..900 {
        app.update();
        let calls = take_calls(app);
        if let Some(start) = calls
            .into_iter()
            .find(|call| matches!(call, DialogueCall::Start { .. }))
        {
            return start;
        }
    }
    panic!("Approach never established a physically validated start request");
}

fn acknowledge(app: &mut App, call: DialogueCall) {
    app.world()
        .resource::<DialogueMailbox>()
        .0
        .lock()
        .unwrap()
        .push(DialogueCompletion {
            call,
            result: Ok(()),
        });
    app.update();
}

fn choice(app: &mut App, actor: Entity, session_id: DialogueSessionId, action: DialogueAction) {
    let reached = app.world().resource::<SceneDialogues>().sessions[&actor].reached;
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Client(actor),
        message: DialogueRequest {
            generation: reached.generation,
            sequence: reached.sequence,
            session_id,
            action,
        },
    });
}

fn topic() -> DialogueAction {
    DialogueAction::Topic {
        topic_id: "profession".into(),
        action_id: "topic-one".into(),
        expected_revision: 0,
    }
}

#[test]
fn only_physical_arrival_requests_a_start_and_only_acknowledgement_opens_it() {
    let (mut app, actor, target) = scene(6.0);
    order(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Talk { target }),
    );
    for _ in 0..3 {
        app.update();
    }
    assert!(
        take_calls(&mut app).is_empty(),
        "A distant NPC must not start a conversation"
    );
    let mut start = None;
    for _ in 0..900 {
        app.update();
        if let Some(call) = take_calls(&mut app).into_iter().next() {
            start = Some(call);
            break;
        }
    }
    let start = start.expect("The physical motor never reached the resident");
    assert!(matches!(start, DialogueCall::Start { .. }));
    assert!(matches!(
        app.world().resource::<SceneDialogues>().sessions[&actor].phase,
        DialoguePhase::Starting
    ));
    assert!(app.world().get::<Transform>(actor).unwrap().translation.x > 1.0);
    acknowledge(&mut app, start);
    assert!(matches!(
        app.world().resource::<SceneDialogues>().sessions[&actor].phase,
        DialoguePhase::Open { pending: None }
    ));
}

#[test]
fn cancelled_removed_hostile_or_departed_contacts_cannot_open_after_a_late_start() {
    #[derive(Debug)]
    enum Loss {
        Stop,
        Mode,
        Removed,
        Hostile,
        Visit,
        Distance,
    }
    for loss in [
        Loss::Stop,
        Loss::Mode,
        Loss::Removed,
        Loss::Hostile,
        Loss::Visit,
        Loss::Distance,
    ] {
        let (mut app, actor, target) = scene(1.6);
        let start = approach(&mut app, actor, target);
        let key = start.key().clone();
        match loss {
            Loss::Stop => order(&mut app, actor, ControlChange::Stop),
            Loss::Mode => order(&mut app, actor, ControlChange::SetMode(ControlMode::Direct)),
            Loss::Removed => {
                app.world_mut().despawn(target);
            }
            Loss::Hostile => {
                app.world_mut()
                    .entity_mut(target)
                    .insert(TacticalCombatSide::Enemy);
            }
            Loss::Visit => app
                .world_mut()
                .resource_mut::<LocalVisits>()
                .release(CharacterId(1)),
            Loss::Distance => {
                app.world_mut()
                    .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
                    .unwrap()
                    .0
                    .x = 8.0;
            }
        }
        acknowledge(&mut app, start);
        assert!(
            !app.world()
                .resource::<SceneDialogues>()
                .sessions
                .contains_key(&actor),
            "Lost-contact case {loss:?} retained a session; order={:?}, target={:?}",
            app.world().get::<CharacterOrder>(actor),
            app.world().get::<Transform>(target)
        );
        assert!(
            take_calls(&mut app)
                .iter()
                .any(|call| matches!(call, DialogueCall::Close { key: closed } if *closed == key))
        );
    }
}

#[test]
fn an_older_start_cannot_replace_a_new_approach_to_the_same_resident() {
    let (mut app, actor, target) = scene(1.6);
    let old = approach(&mut app, actor, target);
    let current = approach(&mut app, actor, target);
    assert!(old.key().session != current.key().session);
    acknowledge(&mut app, old.clone());
    assert!(matches!(
        app.world().resource::<SceneDialogues>().sessions[&actor].phase,
        DialoguePhase::Starting
    ));
    assert!(app.world().resource::<SceneDialogues>().sessions[&actor].key == *current.key());
    acknowledge(&mut app, current);
    assert!(matches!(
        app.world().resource::<SceneDialogues>().sessions[&actor].phase,
        DialoguePhase::Open { pending: None }
    ));
}

#[test]
fn choices_require_the_controlled_actor_current_session_and_present_contact() {
    let (mut app, actor, target) = scene(1.6);
    let start = approach(&mut app, actor, target);
    let key = start.key().clone();
    acknowledge(&mut app, start);
    let reached = app.world().resource::<SceneDialogues>().sessions[&actor].reached;
    app.world_mut().trigger(FromClient {
        client_id: ClientId::Client(target),
        message: DialogueRequest {
            generation: reached.generation,
            sequence: reached.sequence,
            session_id: key.session.clone(),
            action: topic(),
        },
    });
    choice(
        &mut app,
        actor,
        DialogueSessionId::try_from("dialogue:1:forged".to_owned()).unwrap(),
        topic(),
    );
    let foreign_prompt = adventuresim_core::dialogue_boundary::DialoguePromptRowId::try_from(
        "dialogue:1:elsewhere:prompt:choose:action".to_owned(),
    )
    .unwrap();
    choice(
        &mut app,
        actor,
        key.session.clone(),
        DialogueAction::Answer {
            prompt_row_id: foreign_prompt,
            choice_ids: vec!["yes".into()],
            action_id: "foreign-prompt".into(),
            expected_revision: 0,
        },
    );
    assert!(take_calls(&mut app).is_empty());
    choice(&mut app, actor, key.session.clone(), topic());
    choice(&mut app, actor, key.session.clone(), topic());
    let calls = take_calls(&mut app);
    assert_eq!(
        calls.len(),
        1,
        "A retry while the reducer is pending must not duplicate the call"
    );
    let call = calls.into_iter().next().unwrap();
    order(&mut app, actor, ControlChange::Stop);
    acknowledge(&mut app, call);
    assert!(
        !app.world()
            .resource::<SceneDialogues>()
            .sessions
            .contains_key(&actor)
    );

    let next = approach(&mut app, actor, target);
    let next_key = next.key().clone();
    acknowledge(&mut app, next);
    take_calls(&mut app);
    app.world_mut()
        .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
        .unwrap()
        .0
        .x = 8.0;
    choice(&mut app, actor, next_key.session, topic());
    assert!(
        take_calls(&mut app)
            .iter()
            .all(|call| matches!(call, DialogueCall::Close { .. }))
    );
    assert!(
        !app.world()
            .resource::<SceneDialogues>()
            .sessions
            .contains_key(&actor)
    );
}

#[test]
fn a_conversation_yields_to_self_defense_and_resumes_as_a_new_session() {
    let (mut app, actor, target) = scene(1.6);
    let start = approach(&mut app, actor, target);
    let previous = start.key().session.clone();
    acknowledge(&mut app, start);
    let aggressor = app
        .world_mut()
        .spawn((
            Player::default(),
            CharacterId(3),
            TacticalCombatSide::Enemy,
            TacticalCombatState::default(),
            crate::player_projection::player_collider(),
            tactical_character_controller(),
            Transform::from_xyz(0.0, 0.95, -1.5),
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
    app.world_mut()
        .trigger(crate::control::perception::ReceivedDamage {
            attacker: aggressor,
            defender: actor,
        });
    app.update();
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().status,
        OrderStatus::Defending
    );
    assert_eq!(
        app.world()
            .get::<crate::control::AgentState>(actor)
            .unwrap()
            .target(),
        Some(aggressor)
    );
    assert!(
        !app.world()
            .resource::<SceneDialogues>()
            .sessions
            .contains_key(&actor)
    );
    assert!(
        take_calls(&mut app)
            .iter()
            .any(|call| matches!(call, DialogueCall::Close { .. }))
    );
    app.world_mut().despawn(aggressor);
    app.update();
    let dialogue = &app.world().resource::<SceneDialogues>().sessions[&actor];
    assert!(dialogue.key.session != previous);
    assert!(matches!(dialogue.phase, DialoguePhase::Starting));
}

#[test]
fn a_blocked_route_fails_without_requesting_dialogue_through_the_wall() {
    let (mut app, actor, target) = scene(6.0);
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(0.5, 3.0, 30.0),
        Transform::from_xyz(3.0, 1.5, 0.0),
    ));
    for _ in 0..3 {
        app.update();
    }
    order(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Talk { target }),
    );
    for _ in 0..900 {
        app.update();
        assert!(
            take_calls(&mut app)
                .iter()
                .all(|call| !matches!(call, DialogueCall::Start { .. }))
        );
        if matches!(
            app.world().get::<CharacterOrder>(actor).unwrap().status,
            OrderStatus::Failed(_)
        ) {
            break;
        }
    }
    assert!(matches!(
        app.world().get::<CharacterOrder>(actor).unwrap().status,
        OrderStatus::Failed(_)
    ));
    assert!(
        !app.world()
            .resource::<SceneDialogues>()
            .sessions
            .contains_key(&actor)
    );
    assert!(
        app.world()
            .get::<adventuresim_tactical_core::avian3d::prelude::Position>(actor)
            .unwrap()
            .0
            .x
            < 3.0
    );
}

#[test]
fn a_resident_escaping_the_pursuit_bound_never_starts_dialogue() {
    let (mut app, actor, target) = scene(6.0);
    order(
        &mut app,
        actor,
        ControlChange::Order(CharacterGoal::Talk { target }),
    );
    app.update();
    app.world_mut()
        .get_mut::<adventuresim_tactical_core::avian3d::prelude::Position>(target)
        .unwrap()
        .0
        .x = 20.0;
    app.update();
    assert_eq!(
        app.world().get::<CharacterOrder>(actor).unwrap().status,
        OrderStatus::Failed(ControlRejection::InvalidTarget)
    );
    assert!(take_calls(&mut app).is_empty());
}
