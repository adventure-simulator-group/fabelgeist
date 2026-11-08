use super::*;

pub(super) fn key_code(input: &str) -> Option<KeyCode> {
    Some(match input {
        "q" => KeyCode::KeyQ,
        "e" => KeyCode::KeyE,
        "f" => KeyCode::KeyF,
        "x" => KeyCode::KeyX,
        "tab" => KeyCode::Tab,
        "r" => KeyCode::KeyR,
        "g" => KeyCode::KeyG,
        "y" => KeyCode::KeyY,
        "h" => KeyCode::KeyH,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "5" => KeyCode::Digit5,
        "t" => KeyCode::KeyT,
        "`" => KeyCode::Backquote,
        "v" => KeyCode::KeyV,
        "b" => KeyCode::KeyB,
        "z" => KeyCode::KeyZ,
        "c" => KeyCode::KeyC,
        _ => return None,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_grab_input(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(Entity, &GlobalTransform), With<ClientPlayer>>,
    item_owners: Query<(Entity, &ItemOf, Option<&EquipSlot>)>,
    topologies: Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>,
    properties: Query<&ItemProperties>,
    action_states: Query<&EquipmentActionState>,
    world_targets: WorldGrabTargets,
    mut session: ResMut<GrabSession>,
) {
    session.invalid_flash_remaining =
        (session.invalid_flash_remaining - time.delta_secs()).max(0.0);
    let (actor, actor_transform) = *player;
    if session.active.is_none() && !mouse.pressed(MouseButton::Right) {
        session.active = if mouse.just_pressed(MouseButton::Left) {
            Some(EquipmentHand::Right)
        } else if mouse.just_pressed(MouseButton::Middle) {
            Some(EquipmentHand::Left)
        } else {
            None
        };
    }
    let Some(hand) = session.active else { return };
    let held = held_item(actor, hand, &item_owners);
    session.selection = world_grab_selection(
        session.selection,
        held.is_some(),
        world_targets.pointed(actor_transform),
    );
    let held_preview = HeldPreviewItem::from(held.and_then(|entity| properties.get(entity).ok()));
    slot_selection::keyboard_slots(&mut session, &keys, held_preview, |location| {
        ordered_preview_at_location(actor, location, &topologies)
    });
    let released = match hand {
        EquipmentHand::Right => mouse.just_released(MouseButton::Left),
        EquipmentHand::Left => mouse.just_released(MouseButton::Middle),
    };
    if !released {
        return;
    }
    let Ok(action_state) = action_states.get(actor) else {
        warn!(
            ?actor,
            "Equipment action not sent: authoritative revision is unavailable"
        );
        (session.active, session.selection, session.repeated_input) = (None, None, None);
        return;
    };
    let action = match session.selection {
        Some(GrabSelection::Slot { location, depth }) => EquipmentAction::Slot {
            location,
            depth,
            expected_destination: ordered_preview_at_location(actor, location, &topologies)
                .get(depth as usize)
                .map(|target| target.addressed_entity()),
        },
        Some(GrabSelection::Hand(destination)) => EquipmentAction::Hand {
            destination,
            expected_destination: held_item(actor, destination, &item_owners),
        },
        Some(GrabSelection::SceneItem(item)) => EquipmentAction::Pickup { item },
        Some(GrabSelection::Door(door)) => EquipmentAction::OpenDoor { door },
        Some(GrabSelection::Window(window)) => EquipmentAction::ToggleWindow { window },
        None if held.is_some() => EquipmentAction::Drop,
        None => {
            session.active = None;
            session.repeated_input = None;
            return;
        }
    };
    let sequence = session.next_sequence;
    session.next_sequence = sequence.next();
    commands.client_trigger(EquipmentActionRequest {
        actor,
        sequence,
        expected_revision: action_state.revision,
        hand,
        expected_hand_item: held,
        action,
    });
    (session.active, session.selection, session.repeated_input) = (None, None, None);
}
