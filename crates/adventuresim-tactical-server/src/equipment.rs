//! Native equipment intents commit through the durable inventory authority.

use adventuresim_core::item_catalog;
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::{ClientId, FromClient, SendTargets, ServerTriggerExt, ToClients},
    equipment::{EquipmentActionOutcome, EquipmentActionResponse, EquipmentRejection},
    prelude::{EquipmentAction, EquipmentActionRequest},
};
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use attachment::attachment_topology;
use commits::{EquipmentCommandHost, EquipmentCommits, finish_equipment_commits};
use environment::EquipmentEnvironment;
pub(crate) use lifecycle::{purge_equipment_lifecycle, reconnect_equipment_lifecycle};
use plan::{plan_hand, plan_slot};
use topology::*;

mod attachment;
mod commits;
mod environment;
mod fit;
mod lifecycle;
mod plan;
mod topology;

#[cfg(test)]
mod behavior;

const MAX_PENDING_PER_ACTOR: usize = 4;

type ItemView<'a> = (
    Entity,
    &'a ItemProperties,
    Option<&'a ItemOf>,
    &'a EquipmentTopology,
    Option<&'a EquipSlot>,
    Has<TacticalSceneItem>,
);

pub(crate) struct TacticalEquipmentPlugin;

#[derive(Resource, Default)]
pub(crate) struct PendingEquipmentActions(VecDeque<(Entity, EquipmentActionRequest)>);

#[derive(Resource, Default)]
pub(crate) struct LastEquipmentSequence(HashMap<Entity, EquipmentSequence>);

#[derive(bevy::ecs::system::SystemParam)]
struct EquipmentInventory<'w, 's> {
    items: Query<'w, 's, ItemView<'static>>,
    rows: Query<'w, 's, &'static TacticalInventoryItemId>,
}

impl Plugin for TacticalEquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingEquipmentActions>()
            .init_resource::<LastEquipmentSequence>()
            .init_resource::<EquipmentCommits>()
            .add_observer(queue_equipment_action)
            .add_systems(
                Update,
                (finish_equipment_commits, process_equipment_actions)
                    .chain()
                    .after(crate::player_projection::PlayerProjectionSet::Spawn)
                    .after(crate::combat::CombatSet::Condition),
            );
    }
}

fn queue_equipment_action(
    request: On<FromClient<EquipmentActionRequest>>,
    mut pending: ResMut<PendingEquipmentActions>,
    mut commands: Commands,
) {
    let Some(controlled) = request.client_id.entity() else {
        return;
    };
    let rejection = if request.actor != controlled {
        Some(EquipmentRejection::Uncontrolled)
    } else if !can_enqueue(&pending.0, controlled) {
        Some(EquipmentRejection::Busy)
    } else {
        None
    };
    if let Some(reason) = rejection {
        respond(
            &mut commands,
            controlled,
            request.sequence,
            EquipmentActionOutcome::Rejected(reason),
        );
        return;
    }
    pending.0.push_back((controlled, **request));
}

fn can_enqueue(queue: &VecDeque<(Entity, EquipmentActionRequest)>, actor: Entity) -> bool {
    queue.iter().filter(|(queued, _)| *queued == actor).count() < MAX_PENDING_PER_ACTOR
}

fn process_equipment_actions(
    mut commands: Commands,
    mut pending: ResMut<PendingEquipmentActions>,
    mut sequences: ResMut<LastEquipmentSequence>,
    mut actors: Query<
        (
            &CharacterId,
            &TacticalCombatState,
            &Limbs,
            &mut EquipmentActionState,
        ),
        With<Player>,
    >,
    inventory: EquipmentInventory,
    mut environment: EquipmentEnvironment,
    mut host: EquipmentCommandHost,
) {
    let items = &inventory.items;
    let rows = &inventory.rows;
    let Some((controlled, request)) = pending.0.pop_front() else {
        return;
    };
    let outcome = (|| {
        if request.actor != controlled {
            return Err(EquipmentRejection::Uncontrolled);
        }
        let (character, combat, limbs, mut state) = actors
            .get_mut(controlled)
            .map_err(|_| EquipmentRejection::Unavailable)?;
        if !request
            .sequence
            .is_admissible_after(sequences.0.get(&controlled).copied())
        {
            return Err(EquipmentRejection::Replayed);
        }
        // Consume admitted network intent once, even when its contents fail.
        // A replay must never become valid after inventory or ability changes.
        sequences.0.insert(controlled, request.sequence);
        if host.is_busy(controlled) {
            return Err(EquipmentRejection::Busy);
        }
        if request.expected_revision != state.revision
            || hand_item(controlled, request.hand, items) != request.expected_hand_item
        {
            return Err(EquipmentRejection::Stale);
        }
        if !can_use_hand(combat, limbs, request.hand)
            || matches!(request.action, EquipmentAction::Hand { destination, .. }
                if !can_use_hand(combat, limbs, destination))
        {
            return Err(EquipmentRejection::Unable);
        }
        if let Some(accepted) = environment_action(request, &mut environment) {
            if !accepted {
                return Err(EquipmentRejection::InvalidPlacement);
            }
            state.revision = state.revision.next();
            return Ok(EquipmentActionOutcome::Committed {
                revision: state.revision,
            });
        }
        let changes = match request.action {
            EquipmentAction::Slot {
                location,
                depth,
                expected_destination,
            } => {
                if ordered_at_location(controlled, location, items)
                    .get(usize::from(depth))
                    .map(ReachableTarget::expected_entity)
                    != expected_destination
                {
                    return Err(EquipmentRejection::Stale);
                }
                plan_slot(controlled, request.hand, location, depth, items, rows)?
            }
            EquipmentAction::Hand {
                destination,
                expected_destination,
            } => {
                if hand_item(controlled, destination, items) != expected_destination {
                    return Err(EquipmentRejection::Stale);
                }
                plan_hand(controlled, request.hand, destination, items, rows)?
            }
            EquipmentAction::Drop | EquipmentAction::Pickup { .. } => {
                return Err(EquipmentRejection::SceneCustodyUnavailable);
            }
            EquipmentAction::OpenDoor { .. } | EquipmentAction::ToggleWindow { .. } => {
                return Err(EquipmentRejection::InvalidPlacement);
            }
        };
        host.submit(controlled, *character, request.sequence, changes)?;
        Ok(EquipmentActionOutcome::Pending)
    })()
    .unwrap_or_else(EquipmentActionOutcome::Rejected);
    respond(&mut commands, controlled, request.sequence, outcome);
}

fn can_use_hand(combat: &TacticalCombatState, limbs: &Limbs, hand: EquipmentHand) -> bool {
    let health = match hand {
        EquipmentHand::Left => limbs.left_arm,
        EquipmentHand::Right => limbs.right_arm,
    };
    combat.incapacitation.is_finite()
        && !combat.is_incapacitated()
        && health.is_finite()
        && health > 0.0
}

fn environment_action(
    request: EquipmentActionRequest,
    environment: &mut EquipmentEnvironment,
) -> Option<bool> {
    match request.action {
        EquipmentAction::OpenDoor { door } => Some(
            request.expected_hand_item.is_none()
                && environment.doors.try_open_from_inside(request.actor, door),
        ),
        EquipmentAction::ToggleWindow { window } => Some(
            request.expected_hand_item.is_none()
                && environment
                    .windows
                    .try_toggle_from_inside(request.actor, window),
        ),
        _ => None,
    }
}

fn respond(
    commands: &mut Commands,
    actor: Entity,
    sequence: EquipmentSequence,
    outcome: EquipmentActionOutcome,
) {
    commands.server_trigger(ToClients {
        targets: SendTargets::Single(ClientId::Client(actor)),
        message: EquipmentActionResponse {
            actor,
            sequence,
            outcome,
        },
    });
}
