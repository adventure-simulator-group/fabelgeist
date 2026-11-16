//! Ordered goal changes and generation-checked direct actions.

use adventuresim_tactical_core::control::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

use crate::message::{DefendRequest, MeleeActionRequest, RangedActionRequest};

#[derive(Event, Clone, Copy, Debug, Serialize, Deserialize, MapEntities)]
pub struct ControlRequest {
    pub generation: ControlGeneration,
    pub sequence: OrderSequence,
    #[entities]
    pub change: ControlChange,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, MapEntities)]
pub enum ControlChange {
    SetMode(ControlMode),
    Order(#[entities] CharacterGoal),
    AppendWaypoint(NavigationDestination),
    Stop,
    Policy(BehaviorPolicy),
}

#[derive(Event, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ControlResponse {
    pub sequence: OrderSequence,
    pub control: CharacterControl,
    pub result: Result<(), ControlRejection>,
}

#[derive(Event, Clone, Copy, Debug, Serialize, Deserialize, MapEntities)]
pub struct DirectActionRequest {
    pub generation: ControlGeneration,
    #[entities]
    pub action: DirectAction,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, MapEntities)]
pub enum DirectAction {
    Melee(#[entities] MeleeActionRequest),
    Ranged(#[entities] RangedActionRequest),
    Defend(DefendRequest),
}

/// Client-side acknowledged authority. Suppress input while a switch is pending.
#[cfg(feature = "client")]
#[derive(Resource, Clone, Debug, Default)]
pub struct LocalControl {
    pub acknowledged: CharacterControl,
    pub sequence: OrderSequence,
    pub pending: Option<OrderSequence>,
    pub rejection: Option<ControlRejection>,
}

#[cfg(feature = "client")]
impl LocalControl {
    pub fn is_direct(&self) -> bool {
        self.pending.is_none() && self.acknowledged.mode == ControlMode::Direct
    }

    pub fn request(&mut self, change: ControlChange) -> ControlRequest {
        self.sequence = self.sequence.next();
        self.pending = Some(self.sequence);
        self.rejection = None;
        ControlRequest {
            generation: self.acknowledged.generation,
            sequence: self.sequence,
            change,
        }
    }
}

/// Wrap the action at the client boundary; the server never registers raw actions.
#[cfg(feature = "client")]
pub fn send_direct_action(commands: &mut Commands, action: DirectAction) {
    use bevy_replicon::prelude::ClientTriggerExt;
    commands.queue(move |world: &mut World| {
        let local = world.resource::<LocalControl>();
        if !local.is_direct() {
            return;
        }
        let generation = local.acknowledged.generation;
        world
            .commands()
            .client_trigger(DirectActionRequest { generation, action });
    });
}

#[cfg(feature = "client")]
pub trait DirectActionCommands {
    fn direct_action(&mut self, action: DirectAction);
}

#[cfg(feature = "client")]
impl DirectActionCommands for Commands<'_, '_> {
    fn direct_action(&mut self, action: DirectAction) {
        send_direct_action(self, action);
    }
}
