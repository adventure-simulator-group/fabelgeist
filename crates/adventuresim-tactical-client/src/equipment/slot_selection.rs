//! Keyboard and HUD resolve a binding to an eligible anatomical location.
use super::{
    GrabSelection, GrabSession, INVALID_FLASH_SECS, PreviewTarget, eligible_slot_depth, key_code,
    outermost_occupied_depth,
};
use adventuresim_core::equipment::{INPUT_ADDRESS_MAPPINGS, InputAddressMapping};
use adventuresim_core::item_catalog::EquipmentLocation;
use bevy::prelude::{ButtonInput, KeyCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MappedSlot {
    pub location: EquipmentLocation,
    pub depth: u16,
    location_index: usize,
}

fn eligible_mapped_slot(
    mapping: &InputAddressMapping,
    held_item_id: Option<&str>,
    start: usize,
    mut layers_at: impl FnMut(EquipmentLocation) -> Vec<PreviewTarget>,
) -> Option<MappedSlot> {
    (0..mapping.locations.len()).find_map(|offset| {
        let location_index = (start + offset) % mapping.locations.len();
        let location = mapping.locations[location_index];
        let layers = layers_at(location);
        let depth = match held_item_id {
            Some(item_id) => eligible_slot_depth(item_id, location, &layers),
            None => outermost_occupied_depth(&layers),
        }?;
        Some(MappedSlot {
            location,
            depth: u16::try_from(depth).ok()?,
            location_index,
        })
    })
}

pub(super) fn keyboard_slots(
    session: &mut GrabSession,
    keys: &ButtonInput<KeyCode>,
    held_item_id: Option<&str>,
    mut layers_at: impl FnMut(EquipmentLocation) -> Vec<PreviewTarget>,
) {
    for mapping in INPUT_ADDRESS_MAPPINGS {
        if !key_code(mapping.input).is_some_and(|key| keys.just_pressed(key)) {
            continue;
        }
        let start = session
            .repeated_input
            .filter(|(input, _)| *input == mapping.input)
            .map_or(0, |(_, index)| index + 1);
        if let Some(slot) = eligible_mapped_slot(mapping, held_item_id, start, &mut layers_at) {
            apply_slot(session, mapping, slot);
        } else {
            session.invalid_flash_remaining = INVALID_FLASH_SECS;
        }
    }
}

pub(super) fn hud_slot(
    session: &GrabSession,
    mapping: &InputAddressMapping,
    held_item_id: Option<&str>,
    layers_at: impl FnMut(EquipmentLocation) -> Vec<PreviewTarget>,
) -> Option<MappedSlot> {
    let start = match session.selection {
        Some(GrabSelection::Slot { location, .. }) => mapping
            .locations
            .iter()
            .position(|candidate| *candidate == location)
            .unwrap_or(0),
        _ => 0,
    };
    eligible_mapped_slot(mapping, held_item_id, start, layers_at)
}

pub(super) fn apply_slot(
    session: &mut GrabSession,
    mapping: &InputAddressMapping,
    slot: MappedSlot,
) {
    session.repeated_input = Some((mapping.input, slot.location_index));
    session.selection = Some(GrabSelection::Slot {
        location: slot.location,
        depth: slot.depth,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_tactical_netcode::prelude::EquipmentHand;

    #[test]
    fn fresh_gauntlet_grab_reaches_each_hand_through_keyboard_and_hud() {
        for (key, input, location) in [
            (KeyCode::Backquote, "`", EquipmentLocation::LeftHand),
            (KeyCode::Digit5, "5", EquipmentLocation::RightHand),
        ] {
            let mapping = INPUT_ADDRESS_MAPPINGS
                .iter()
                .find(|mapping| mapping.input == input)
                .unwrap();
            let mut keys = ButtonInput::default();
            keys.press(key);
            let mut session = GrabSession {
                active: Some(EquipmentHand::Right),
                ..Default::default()
            };
            let hud = hud_slot(&session, mapping, Some("mitten_gauntlet"), |_| vec![]).unwrap();
            assert_eq!(
                hud.location, location,
                "fresh HUD binding must reach the authored hand"
            );
            keyboard_slots(&mut session, &keys, Some("mitten_gauntlet"), |_| vec![]);
            assert_eq!(
                session.selection,
                Some(GrabSelection::Slot { location, depth: 0 })
            );
            assert_eq!(session.invalid_flash_remaining, 0.0);
            keys.clear();
            keys.release(key);
            keys.press(key);
            keyboard_slots(&mut session, &keys, Some("mitten_gauntlet"), |_| vec![]);
            assert_eq!(
                session.selection,
                Some(GrabSelection::Slot { location, depth: 0 }),
                "repeating the input must not become stuck on the ineligible arm"
            );
            let mut hud_session = GrabSession::default();
            apply_slot(&mut hud_session, mapping, hud);
            assert_eq!(hud_session.selection, session.selection);
        }
    }
    #[test]
    fn client_keyboard_preview_and_hud_preserve_depth_across_reversed_entity_allocation() {
        use adventuresim_core::item_catalog;
        use adventuresim_tactical_core::prelude::*;
        use bevy::{ecs::system::RunSystemOnce, prelude::*};
        for allocation in [
            ["rerebrace", "couter", "vambrace"],
            ["vambrace", "couter", "rerebrace"],
        ] {
            let mut world = World::new();
            let actor = world.spawn_empty().id();
            for item in allocation {
                let placement = &item_catalog::definition(item)
                    .unwrap()
                    .equipment
                    .as_ref()
                    .unwrap()
                    .placements[0];
                world.spawn((
                    ItemOf(actor),
                    ItemProperties {
                        id: item.into(),
                        weight: 1.0,
                    },
                    EquipmentTopology {
                        placement_id: Some(placement.id.clone()),
                        occupancies: placement
                            .occupancy
                            .iter()
                            .enumerate()
                            .map(|(index, requirement)| EquipmentTopologyOccupancy {
                                occupancy_id: format!("{item}:{index}"),
                                anchor: TacticalEquipmentAnchor::CharacterLocation(
                                    requirement.location,
                                ),
                                channel: requirement.channel,
                                order: requirement.order,
                                requirement_index: index as u16,
                                capacity_index: 0,
                            })
                            .collect(),
                    },
                ));
            }
            let preview = world
                .run_system_once(
                    move |items: Query<(Entity, &ItemOf, &EquipmentTopology, &ItemProperties)>| {
                        super::super::ordered_preview_at_location(
                            actor,
                            EquipmentLocation::LeftArm,
                            &items,
                        )
                    },
                )
                .unwrap();
            let hud = world
                .run_system_once(
                    move |items: Query<(
                        Entity,
                        &ItemOf,
                        Option<&EquipSlot>,
                        &ItemProperties,
                        &EquipmentTopology,
                    )>| {
                        super::super::hud_layers(actor, EquipmentLocation::LeftArm, &items)
                    },
                )
                .unwrap();
            let item_ids = |targets: Vec<PreviewTarget>| {
                targets
                    .into_iter()
                    .map(|target| {
                        world
                            .get::<ItemProperties>(target.entity)
                            .unwrap()
                            .id
                            .clone()
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(item_ids(preview), ["rerebrace", "couter", "vambrace"]);
            assert_eq!(item_ids(hud), ["rerebrace", "couter", "vambrace"]);
        }
    }
}
