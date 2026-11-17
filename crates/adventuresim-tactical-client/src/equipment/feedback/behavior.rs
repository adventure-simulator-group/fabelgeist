//! Acknowledgement and replication order through the real client observers.

use adventuresim_tactical_netcode::replication::AdventureSimulatorReplicationPlugin;

use super::*;

#[test]
fn committed_feedback_waits_for_replication_and_ignores_old_actor_results() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AdventureSimulatorReplicationPlugin))
        .init_resource::<GrabSession>();
    install(&mut app);
    let actor = app
        .world_mut()
        .spawn((ClientPlayer, EquipmentActionState::default()))
        .id();
    let foreign = app.world_mut().spawn_empty().id();
    let request = EquipmentActionRequest {
        actor,
        sequence: EquipmentSequence::FIRST,
        expected_revision: EquipmentRevision::INITIAL,
        hand: EquipmentHand::Right,
        expected_hand_item: None,
        action: EquipmentAction::Drop,
    };
    app.world_mut()
        .resource_scope(|world, mut feedback: Mut<EquipmentFeedback>| {
            feedback.submit(&mut world.commands(), request);
        });
    app.world_mut().flush();
    let revision = EquipmentRevision::INITIAL.next().next();
    for acknowledged_actor in [foreign, actor] {
        app.world_mut().trigger(EquipmentActionResponse {
            actor: acknowledged_actor,
            sequence: request.sequence,
            outcome: EquipmentActionOutcome::Committed { revision },
        });
        app.update();
        assert!(app.world().resource::<EquipmentFeedback>().is_pending());
    }
    app.world_mut()
        .get_mut::<EquipmentActionState>(actor)
        .unwrap()
        .revision = revision.next();
    app.update();
    assert!(!app.world().resource::<EquipmentFeedback>().is_pending());
    let next_request = EquipmentActionRequest {
        sequence: request.sequence.next(),
        ..request
    };
    app.world_mut()
        .resource_scope(|world, mut feedback: Mut<EquipmentFeedback>| {
            feedback.submit(&mut world.commands(), next_request);
        });
    app.world_mut().flush();
    app.world_mut().trigger(EquipmentActionResponse {
        actor,
        sequence: request.sequence,
        outcome: EquipmentActionOutcome::Rejected(
            adventuresim_tactical_netcode::equipment::EquipmentRejection::Stale,
        ),
    });
    assert!(app.world().resource::<EquipmentFeedback>().is_pending());
    app.world_mut().despawn(actor);
    app.update();
    assert!(!app.world().resource::<EquipmentFeedback>().is_pending());
}

#[test]
fn matching_rejection_releases_input_without_changing_equipment_revision() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AdventureSimulatorReplicationPlugin))
        .init_resource::<GrabSession>();
    install(&mut app);
    let actor = app
        .world_mut()
        .spawn((ClientPlayer, EquipmentActionState::default()))
        .id();
    let request = EquipmentActionRequest {
        actor,
        sequence: EquipmentSequence::FIRST,
        expected_revision: EquipmentRevision::INITIAL,
        hand: EquipmentHand::Left,
        expected_hand_item: None,
        action: EquipmentAction::Drop,
    };
    app.world_mut()
        .resource_scope(|world, mut feedback: Mut<EquipmentFeedback>| {
            feedback.submit(&mut world.commands(), request);
        });
    app.world_mut().flush();
    app.world_mut().trigger(EquipmentActionResponse {
        actor,
        sequence: request.sequence,
        outcome: EquipmentActionOutcome::Rejected(
            adventuresim_tactical_netcode::equipment::EquipmentRejection::SceneCustodyUnavailable,
        ),
    });
    assert!(!app.world().resource::<EquipmentFeedback>().is_pending());
    assert_eq!(
        app.world()
            .get::<EquipmentActionState>(actor)
            .unwrap()
            .revision,
        EquipmentRevision::INITIAL
    );
}
