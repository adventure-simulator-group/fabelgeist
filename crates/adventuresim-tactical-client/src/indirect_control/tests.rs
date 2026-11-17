use super::*;

#[test]
fn restored_orders_advance_the_client_counter_without_losing_a_newer_pending_request() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<LocalControl>()
        .add_systems(Update, synchronize_authority);
    let retained = OrderSequence::default().next().next();
    let actor = app
        .world_mut()
        .spawn((
            ClientPlayer,
            CharacterControl {
                mode: ControlMode::Indirect,
                ..default()
            },
            CharacterOrder {
                sequence: retained,
                ..default()
            },
        ))
        .id();
    app.update();
    let request = app
        .world_mut()
        .resource_mut::<LocalControl>()
        .request(ControlChange::Stop);
    assert!(
        request.sequence > retained,
        "A restored controller must accept the next click"
    );
    app.world_mut()
        .get_mut::<CharacterOrder>(actor)
        .unwrap()
        .status = OrderStatus::Executing;
    app.update();
    let local = app.world().resource::<LocalControl>();
    assert_eq!(local.sequence, request.sequence);
    assert_eq!(local.pending, Some(request.sequence));
}
