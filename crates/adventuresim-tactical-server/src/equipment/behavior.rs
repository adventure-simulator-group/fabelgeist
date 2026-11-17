//! Observe real network admission and dispatch without a durable host connection.

use adventuresim_tactical_netcode::bevy_replicon::{
    prelude::*,
    test_app::{ServerTestAppExt, TestClientEntity},
};
use bevy::state::app::StatesPlugin;

use super::*;

#[derive(Resource, Default)]
struct Responses(Vec<EquipmentActionResponse>);

struct EquipmentTest {
    server: App,
    client: App,
}

fn app() -> (EquipmentTest, Entity, Entity) {
    let mut app = App::new();
    let mut client = App::new();
    for peer in [&mut app, &mut client] {
        peer.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            RepliconPlugins.set(ServerPlugin::new(PostUpdate)),
        ))
        .replicate::<CharacterId>()
        .add_mapped_client_event::<EquipmentActionRequest>(Channel::Ordered)
        .add_mapped_server_event::<EquipmentActionResponse>(Channel::Ordered);
    }
    app.add_plugins(TacticalEquipmentPlugin);
    client.init_resource::<Responses>().add_observer(
        |event: On<EquipmentActionResponse>, mut responses: ResMut<Responses>| {
            responses.0.push((*event).clone());
        },
    );
    app.finish();
    client.finish();
    app.connect_client(&mut client);
    let actor = **client.world().resource::<TestClientEntity>();
    app.world_mut().entity_mut(actor).insert(Replicated);
    app.world_mut().entity_mut(actor).insert((
        Player {
            name: "Equipment actor".into(),
        },
        CharacterId(1),
        TacticalCombatState::default(),
        Limbs::default(),
        EquipmentActionState::default(),
        Transform::from_xyz(5.0, 2.0, 3.0),
    ));
    let item = app
        .world_mut()
        .spawn((
            ItemOf(actor),
            ItemProperties {
                id: "torch".into(),
                weight: 1.0,
            },
            TacticalInventoryItemId(1),
            hand_topology(EquipmentHand::Right),
            EquipSlot::HoldingRight,
            Transform::default(),
        ))
        .id();
    app.update();
    app.exchange_with_client(&mut client);
    client.update();
    (
        EquipmentTest {
            server: app,
            client,
        },
        actor,
        item,
    )
}

fn request(actor: Entity, item: Entity, sequence: u32) -> EquipmentActionRequest {
    EquipmentActionRequest {
        actor,
        sequence: EquipmentSequence::from_encoded(sequence),
        expected_revision: EquipmentRevision::INITIAL,
        hand: EquipmentHand::Right,
        expected_hand_item: Some(item),
        action: EquipmentAction::Hand {
            destination: EquipmentHand::Left,
            expected_destination: None,
        },
    }
}

fn exchange(app: &mut EquipmentTest) {
    app.server.update();
    app.server.exchange_with_client(&mut app.client);
    app.client.update();
}

fn send(app: &mut EquipmentTest, client: Entity, request: EquipmentActionRequest) {
    app.server.world_mut().trigger(FromClient {
        client_id: ClientId::Client(client),
        message: request,
    });
    exchange(app);
}

fn rejected(app: &EquipmentTest, expected: EquipmentRejection) {
    let response = app.client.world().resource::<Responses>().0.last().unwrap();
    assert!(
        matches!(&response.outcome, EquipmentActionOutcome::Rejected(reason) if *reason == expected),
        "{:?}",
        response.outcome
    );
}

#[test]
fn admission_failures_and_replays_preserve_native_equipment_and_body() {
    let (mut app, actor, item) = app();
    let topology = app
        .server
        .world()
        .get::<EquipmentTopology>(item)
        .unwrap()
        .clone();
    let body = *app.server.world().get::<Transform>(actor).unwrap();
    let mut intent = request(actor, item, 1);
    send(&mut app, actor, intent);
    rejected(&app, EquipmentRejection::Unavailable);
    intent.sequence = intent.sequence.next();
    intent.expected_revision = EquipmentRevision::INITIAL.next();
    send(&mut app, actor, intent);
    rejected(&app, EquipmentRejection::Stale);
    intent.sequence = intent.sequence.next();
    intent.expected_revision = EquipmentRevision::INITIAL;
    app.server
        .world_mut()
        .get_mut::<TacticalCombatState>(actor)
        .unwrap()
        .incapacitation = 1.0;
    send(&mut app, actor, intent);
    rejected(&app, EquipmentRejection::Unable);
    app.server
        .world_mut()
        .get_mut::<TacticalCombatState>(actor)
        .unwrap()
        .incapacitation = 0.0;
    send(&mut app, actor, intent);
    rejected(&app, EquipmentRejection::Replayed);
    intent.sequence = intent.sequence.next();
    app.server
        .world_mut()
        .get_mut::<Limbs>(actor)
        .unwrap()
        .right_arm = 0.0;
    send(&mut app, actor, intent);
    rejected(&app, EquipmentRejection::Unable);
    app.server
        .world_mut()
        .get_mut::<Limbs>(actor)
        .unwrap()
        .right_arm = 1.0;
    for action in [EquipmentAction::Drop, EquipmentAction::Pickup { item }] {
        intent.sequence = intent.sequence.next();
        intent.action = action;
        send(&mut app, actor, intent);
        rejected(&app, EquipmentRejection::SceneCustodyUnavailable);
    }
    assert_eq!(
        app.server.world().get::<EquipmentTopology>(item),
        Some(&topology)
    );
    assert_eq!(app.server.world().get::<ItemOf>(item).unwrap().0, actor);
    assert_eq!(
        app.server.world().get::<EquipSlot>(item),
        Some(&EquipSlot::HoldingRight)
    );
    assert_eq!(app.server.world().get::<Transform>(actor), Some(&body));
    assert_eq!(
        app.server
            .world()
            .get::<EquipmentActionState>(actor)
            .unwrap()
            .revision,
        EquipmentRevision::INITIAL
    );
}

#[test]
fn condition_recalculation_precedes_equipment_admission() {
    let (mut app, actor, item) = app();
    app.server.add_systems(
        Update,
        (move |mut states: Query<&mut TacticalCombatState>| {
            states.get_mut(actor).unwrap().incapacitation = 1.0;
        })
        .in_set(crate::combat::CombatSet::Condition),
    );
    send(&mut app, actor, request(actor, item, 1));
    rejected(&app, EquipmentRejection::Unable);
    assert_eq!(
        app.server.world().get::<EquipSlot>(item),
        Some(&EquipSlot::HoldingRight)
    );
}

#[test]
fn foreign_control_and_queue_overflow_return_owning_client_rejections() {
    let (mut app, actor, item) = app();
    let foreign = app.server.world_mut().spawn_empty().id();
    let mut intent = request(foreign, item, 1);
    app.server.world_mut().trigger(FromClient {
        client_id: ClientId::Client(actor),
        message: intent,
    });
    exchange(&mut app);
    rejected(&app, EquipmentRejection::Uncontrolled);
    assert!(
        app.server
            .world()
            .resource::<PendingEquipmentActions>()
            .0
            .is_empty()
    );
    intent.actor = actor;
    for sequence in 1..=MAX_PENDING_PER_ACTOR as u32 + 1 {
        intent.sequence = EquipmentSequence::from_encoded(sequence);
        app.server.world_mut().trigger(FromClient {
            client_id: ClientId::Client(actor),
            message: intent,
        });
    }
    assert_eq!(
        app.server
            .world()
            .resource::<PendingEquipmentActions>()
            .0
            .len(),
        MAX_PENDING_PER_ACTOR
    );
    exchange(&mut app);
    let responses = &app.client.world().resource::<Responses>().0;
    assert!(responses.iter().any(|response| matches!(
        response.outcome,
        EquipmentActionOutcome::Rejected(EquipmentRejection::Busy)
    )));
    assert!(responses.iter().all(
        |response| app.client.world().get::<CharacterId>(response.actor) == Some(&CharacterId(1))
    ));
}

#[test]
fn reconnect_purges_old_intents_and_starts_new_connection_chronology() {
    let old = Entity::from_bits(7);
    let new = Entity::from_bits(8);
    let intent = request(old, Entity::from_bits(9), 4);
    let mut pending = PendingEquipmentActions(VecDeque::from([(old, intent)]));
    let mut sequences =
        LastEquipmentSequence(HashMap::from([(old, EquipmentSequence::from_encoded(3))]));
    reconnect_equipment_lifecycle(old, new, &mut pending, &mut sequences);
    assert!(pending.0.is_empty());
    assert!(!sequences.0.contains_key(&old));
    assert!(!sequences.0.contains_key(&new));
}
