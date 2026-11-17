use super::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientId, control::ControlChange,
    replication::AdventureSimulatorReplicationPlugin,
};

#[derive(Resource, Default)]
struct ReceivedDialogueRequests(Vec<ReceivedDialogueRequest>);

struct ReceivedDialogueRequest {
    sender: ClientId,
    request: DialogueRequest,
}

fn scene() -> (App, Entity, DialogueStartResponse) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AdventureSimulatorReplicationPlugin,
        ClientDialoguePlugin,
    ));
    let actor = app.world_mut().spawn((ClientPlayer, CharacterId(7))).id();
    let response = {
        let mut local = app.world_mut().resource_mut::<LocalControl>();
        local.acknowledged.mode = ControlMode::Indirect;
        local.sequence = local.sequence.next();
        DialogueStartResponse {
            controlled_actor: actor,
            actor: CharacterId(7),
            resident: CharacterId(u64::MAX),
            generation: local.acknowledged.generation,
            sequence: local.sequence,
            result: Ok(DialogueSessionId::try_from("dialogue:7:scene:1:1".to_owned()).unwrap()),
        }
    };
    app.finish();
    app.cleanup();
    (app, actor, response)
}

#[test]
fn a_later_local_request_cancels_dialogue_and_rejects_a_late_confirmation() {
    let (mut app, _, response) = scene();
    app.world_mut().trigger(response.clone());
    assert!(app.world().resource::<LocalDialogue>().0.is_some());
    app.world_mut()
        .resource_mut::<LocalControl>()
        .request(ControlChange::Stop);
    app.update();
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
    app.world_mut().trigger(response);
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
}

#[test]
fn another_characters_confirmation_cannot_replace_the_current_conversation() {
    let (mut app, _, response) = scene();
    app.world_mut().trigger(response.clone());
    let original = app
        .world()
        .resource::<LocalDialogue>()
        .0
        .as_ref()
        .unwrap()
        .session
        .clone();
    let mut foreign = response;
    foreign.actor = CharacterId(8);
    foreign.result = Ok(DialogueSessionId::try_from("dialogue:8:scene:1:1".to_owned()).unwrap());
    app.world_mut().trigger(foreign);
    assert_eq!(
        app.world()
            .resource::<LocalDialogue>()
            .0
            .as_ref()
            .unwrap()
            .session,
        original
    );
}

#[test]
fn ending_conversation_stops_the_talk_goal_and_fences_a_late_confirmation() {
    let (mut app, _, response) = scene();
    app.world_mut().trigger(response.clone());
    app.world_mut().trigger(BrowserDialogueRequest {
        session_id: response.result.clone().unwrap(),
        action: DialogueAction::Close,
    });
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
    let local = app.world().resource::<LocalControl>();
    assert!(local.sequence > response.sequence);
    assert!(local.pending.is_some());
    app.world_mut().trigger(response);
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
}

#[test]
fn closing_an_older_session_cannot_close_a_newer_one() {
    let (mut app, _, response) = scene();
    app.world_mut().trigger(response.clone());
    let old = app
        .world()
        .resource::<LocalDialogue>()
        .0
        .as_ref()
        .unwrap()
        .session
        .clone();
    let mut next = response;
    {
        let mut local = app.world_mut().resource_mut::<LocalControl>();
        local.sequence = local.sequence.next();
        next.sequence = local.sequence;
    }
    next.result = Ok(DialogueSessionId::try_from("dialogue:7:scene:1:2".to_owned()).unwrap());
    app.world_mut().trigger(next);
    let current = app
        .world()
        .resource::<LocalDialogue>()
        .0
        .as_ref()
        .unwrap()
        .session
        .clone();
    app.world_mut().trigger(DialogueEnded { session_id: old });
    assert_eq!(
        app.world()
            .resource::<LocalDialogue>()
            .0
            .as_ref()
            .unwrap()
            .session,
        current
    );
    app.world_mut().trigger(DialogueEnded {
        session_id: current,
    });
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
}

#[test]
fn losing_the_controlled_actor_cancels_dialogue_and_prevents_reopening() {
    let (mut app, actor, response) = scene();
    app.world_mut().trigger(response.clone());
    app.world_mut().despawn(actor);
    // Reprojection of the same durable character must not inherit the old
    // connection's confirmation, even without an empty-player frame.
    app.world_mut().spawn((ClientPlayer, CharacterId(7)));
    app.update();
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
    app.world_mut().trigger(response);
    assert!(app.world().resource::<LocalDialogue>().0.is_none());
}

#[test]
fn confirmed_actor_mapping_and_browser_choices_cross_the_registered_native_protocol() {
    use adventuresim_tactical_netcode::bevy_replicon::{
        prelude::*,
        test_app::{ServerTestAppExt, TestClientEntity},
    };
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    let mut server = App::new();
    let mut client = App::new();
    for app in [&mut server, &mut client] {
        app.add_plugins((MinimalPlugins, AdventureSimulatorReplicationPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / f64::from(adventuresim_tactical_netcode::TACTICAL_TICK_HZ),
            )));
    }
    server
        .init_resource::<ReceivedDialogueRequests>()
        .add_observer(
            |request: On<FromClient<DialogueRequest>>,
             mut received: ResMut<ReceivedDialogueRequests>| {
                received.0.push(ReceivedDialogueRequest {
                    sender: request.client_id,
                    request: request.message.clone(),
                });
            },
        );
    client.add_plugins(ClientDialoguePlugin);
    // Force different entity spaces so an unmapped server ID cannot pass.
    for _ in 0..8 {
        client.world_mut().spawn_empty();
    }
    for app in [&mut server, &mut client] {
        app.finish();
        app.cleanup();
    }
    server.connect_client(&mut client);
    let server_actor = **client.world().resource::<TestClientEntity>();
    server
        .world_mut()
        .entity_mut(server_actor)
        .insert((Replicated, CharacterId(7)));
    for _ in 0..3 {
        server.update();
        server.exchange_with_client(&mut client);
        client.update();
    }
    let client_actor = client
        .world_mut()
        .query::<(Entity, &CharacterId)>()
        .iter(client.world())
        .find_map(|(entity, id)| (id.0 == 7).then_some(entity))
        .unwrap();
    assert_ne!(server_actor, client_actor);
    client
        .world_mut()
        .entity_mut(client_actor)
        .insert(ClientPlayer);
    let (generation, sequence) = {
        let mut local = client.world_mut().resource_mut::<LocalControl>();
        local.acknowledged.mode = ControlMode::Indirect;
        local.sequence = local.sequence.next();
        (local.acknowledged.generation, local.sequence)
    };
    let session = DialogueSessionId::try_from("dialogue:7:scene:1:1".to_owned()).unwrap();
    server.world_mut().server_trigger(ToClients {
        targets: SendTargets::Single(ClientId::Client(server_actor)),
        message: DialogueStartResponse {
            controlled_actor: server_actor,
            actor: CharacterId(7),
            resident: CharacterId(u64::MAX),
            generation,
            sequence,
            result: Ok(session.clone()),
        },
    });
    for _ in 0..3 {
        server.update();
        server.exchange_with_client(&mut client);
        client.update();
    }
    assert_eq!(
        client
            .world()
            .resource::<LocalDialogue>()
            .0
            .as_ref()
            .unwrap()
            .actor,
        client_actor
    );
    let action = DialogueAction::Answer {
        prompt_row_id: format!("{}:prompt:0", session.as_str()).try_into().unwrap(),
        choice_ids: vec!["yes".into()],
        action_id: "answer-one".into(),
        expected_revision: 2,
    };
    client.world_mut().trigger(BrowserDialogueRequest {
        session_id: session.clone(),
        action: action.clone(),
    });
    client.update();
    server.exchange_with_client(&mut client);
    server.update();
    let received = server.world().resource::<ReceivedDialogueRequests>();
    let [received] = received.0.as_slice() else {
        panic!("The native host must receive exactly one typed browser choice");
    };
    assert_eq!(received.sender, ClientId::Client(server_actor));
    let request = &received.request;
    assert_eq!(request.session_id, session);
    assert_eq!(request.generation, generation);
    assert_eq!(request.sequence, sequence);
    assert_eq!(request.action, action);
    server.world_mut().server_trigger(ToClients {
        targets: SendTargets::Single(ClientId::Client(server_actor)),
        message: DialogueEnded {
            session_id: session,
        },
    });
    for _ in 0..3 {
        server.update();
        server.exchange_with_client(&mut client);
        client.update();
    }
    assert!(client.world().resource::<LocalDialogue>().0.is_none());
}
