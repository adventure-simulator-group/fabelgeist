//! Real network intents and replicated equipment, using the registered Bevy host.

use std::collections::BTreeMap;

use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::ClientTriggerExt,
    equipment::{EquipmentActionOutcome, EquipmentActionResponse, EquipmentRejection},
    message::{EquipmentAction, EquipmentActionRequest},
};
use bevy::prelude::*;

use super::{Acceptance, Args};

type OwnedItems<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ItemOf,
        &'static EquipmentTopology,
        Option<&'static EquipSlot>,
        &'static TacticalInventoryMass,
        &'static TacticalItemQuantity,
    ),
>;

#[derive(Resource, Default)]
pub(super) struct EquipmentChecks {
    stage: Stage,
    actor: Option<Entity>,
    original: BTreeMap<Entity, ItemSnapshot>,
    pending: Option<EquipmentActionRequest>,
    acknowledgement: Option<EquipmentActionOutcome>,
    sequence: EquipmentSequence,
}

#[derive(Clone, Debug, PartialEq)]
struct ItemSnapshot {
    topology: EquipmentTopology,
    hand: Option<EquipSlot>,
    mass: TacticalInventoryMass,
    quantity: TacticalItemQuantity,
}

#[derive(Clone, Copy, Default)]
enum Stage {
    #[default]
    Swap,
    Restore,
    Replay,
    Stale,
    Drop,
    Foreign,
    Done,
}

impl EquipmentChecks {
    fn send(&mut self, commands: &mut Commands, request: EquipmentActionRequest) {
        self.pending = Some(request);
        self.acknowledgement = None;
        commands.client_trigger(request);
    }

    fn fail(&self, reason: &str, exit: &mut MessageWriter<AppExit>) {
        eprintln!("Native equipment acceptance failed: {reason}");
        exit.write(AppExit::error());
    }
}

pub(super) fn on_response(
    response: On<EquipmentActionResponse>,
    mut checks: ResMut<EquipmentChecks>,
) {
    if checks.pending.as_ref().is_some_and(|request| {
        request.sequence == response.sequence && checks.actor == Some(response.actor)
    }) && !matches!(response.outcome, EquipmentActionOutcome::Pending)
    {
        checks.acknowledgement = Some(response.outcome.clone());
    }
}

pub(super) fn step(
    args: Res<Args>,
    mut acceptance: ResMut<Acceptance>,
    mut checks: ResMut<EquipmentChecks>,
    actors: Query<(Entity, &CharacterId, &EquipmentActionState), With<Player>>,
    items: OwnedItems,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !args.equipment
        || matches!(checks.stage, Stage::Done)
        || (args.reconnect && acceptance.capabilities < 2)
    {
        return;
    }
    let Some((actor, _, state)) = actors.iter().find(|(_, id, _)| id.0 == args.character) else {
        return;
    };
    let snapshot = capture(actor, &items);
    if checks.actor.is_none() {
        if !acceptance
            .stable_since
            .is_some_and(|since| since.elapsed() >= super::STABLE_PRESENCE_HOLD)
        {
            return;
        }
        if snapshot.is_empty() || !snapshot.values().any(|item| item.hand.is_some()) {
            return;
        }
        checks.actor = Some(actor);
        checks.original = snapshot.clone();
    }
    if checks.actor != Some(actor) {
        checks.fail("the controlled actor changed during a command", &mut exit);
        return;
    }
    if checks.pending.is_some() {
        if !finish_stage(&mut checks, &snapshot, state, &mut exit) {
            return;
        }
        if matches!(checks.stage, Stage::Done) {
            acceptance.equipment_done = true;
            println!("EQUIPMENT_ACCEPTED swaps=2 replay=1 stale=1 drop=1 foreign=1");
            return;
        }
    }
    let source = match checks.stage {
        Stage::Swap | Stage::Restore => {
            if snapshot
                .values()
                .any(|item| item.hand == Some(EquipSlot::HoldingRight))
            {
                EquipmentHand::Right
            } else {
                EquipmentHand::Left
            }
        }
        _ => EquipmentHand::Right,
    };
    let mut request = EquipmentActionRequest {
        actor,
        sequence: checks.sequence,
        expected_revision: state.revision,
        hand: source,
        expected_hand_item: hand_item(&snapshot, source),
        action: EquipmentAction::Hand {
            destination: other(source),
            expected_destination: hand_item(&snapshot, other(source)),
        },
    };
    match checks.stage {
        Stage::Replay => {
            request.sequence =
                EquipmentSequence::from_encoded(checks.sequence.encoded().wrapping_sub(1))
        }
        Stage::Stale => request.expected_revision = state.revision.next(),
        Stage::Drop => request.action = EquipmentAction::Drop,
        Stage::Foreign => {
            let Some((foreign, _, _)) = actors.iter().find(|(_, id, _)| id.0 != args.character)
            else {
                return;
            };
            request.actor = foreign;
        }
        _ => {}
    }
    if !matches!(checks.stage, Stage::Replay) {
        checks.sequence = checks.sequence.next();
    }
    checks.send(&mut commands, request);
}

fn finish_stage(
    checks: &mut EquipmentChecks,
    current: &BTreeMap<Entity, ItemSnapshot>,
    state: &EquipmentActionState,
    exit: &mut MessageWriter<AppExit>,
) -> bool {
    let Some(outcome) = checks.acknowledgement.as_ref() else {
        return false;
    };
    let accepted = match (&checks.stage, outcome) {
        (Stage::Swap | Stage::Restore, EquipmentActionOutcome::Committed { revision }) => {
            if !state.revision.has_reached(*revision) {
                return false;
            }
            let expected = if matches!(checks.stage, Stage::Swap) {
                swapped(&checks.original)
            } else {
                checks.original.clone()
            };
            current == &expected
        }
        (Stage::Replay, EquipmentActionOutcome::Rejected(EquipmentRejection::Replayed))
        | (Stage::Stale, EquipmentActionOutcome::Rejected(EquipmentRejection::Stale))
        | (
            Stage::Drop,
            EquipmentActionOutcome::Rejected(EquipmentRejection::SceneCustodyUnavailable),
        )
        | (Stage::Foreign, EquipmentActionOutcome::Rejected(EquipmentRejection::Uncontrolled)) => {
            current == &checks.original
        }
        _ => false,
    };
    if !accepted {
        checks.fail(
            &format!("unexpected acknowledgement or inventory mutation: {outcome:?}"),
            exit,
        );
        return false;
    }
    checks.stage = match checks.stage {
        Stage::Swap => Stage::Restore,
        Stage::Restore => Stage::Replay,
        Stage::Replay => Stage::Stale,
        Stage::Stale => Stage::Drop,
        Stage::Drop => Stage::Foreign,
        Stage::Foreign | Stage::Done => Stage::Done,
    };
    checks.pending = None;
    checks.acknowledgement = None;
    true
}

fn capture(actor: Entity, items: &OwnedItems) -> BTreeMap<Entity, ItemSnapshot> {
    items
        .iter()
        .filter(|(_, owner, ..)| owner.0 == actor)
        .map(|(entity, _, topology, hand, mass, quantity)| {
            (
                entity,
                ItemSnapshot {
                    topology: topology.clone(),
                    hand: hand.copied(),
                    mass: *mass,
                    quantity: *quantity,
                },
            )
        })
        .collect()
}

fn swapped(original: &BTreeMap<Entity, ItemSnapshot>) -> BTreeMap<Entity, ItemSnapshot> {
    let mut result = original.clone();
    for item in result.values_mut().filter(|item| item.hand.is_some()) {
        item.hand = item.hand.map(|hand| match hand {
            EquipSlot::HoldingRight => EquipSlot::HoldingLeft,
            EquipSlot::HoldingLeft => EquipSlot::HoldingRight,
            other => other,
        });
        for occupancy in &mut item.topology.occupancies {
            if occupancy.channel == EquipmentChannel::Held {
                occupancy.anchor = match occupancy.anchor {
                    TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::RightHand) => {
                        TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftHand)
                    }
                    TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::LeftHand) => {
                        TacticalEquipmentAnchor::CharacterLocation(EquipmentLocation::RightHand)
                    }
                    ref other => other.clone(),
                };
            }
        }
    }
    result
}

fn hand_item(items: &BTreeMap<Entity, ItemSnapshot>, hand: EquipmentHand) -> Option<Entity> {
    items
        .iter()
        .find_map(|(entity, item)| (item.hand == Some(EquipSlot::from(hand))).then_some(*entity))
}

fn other(hand: EquipmentHand) -> EquipmentHand {
    match hand {
        EquipmentHand::Left => EquipmentHand::Right,
        EquipmentHand::Right => EquipmentHand::Left,
    }
}
