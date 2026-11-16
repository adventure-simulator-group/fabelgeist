//! Orders describe goals; direct input and autonomous behavior share execution.
//! All state in this module belongs to the transient tactical simulation.

use adventuresim_building_generator::spatial_geometry::Position;
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

use crate::scene_coordinates::Scene;

pub use config::{ControlConfig, ControlConfigError};

mod config;

/// Changes whenever authority switches between direct and indirect input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct ControlGeneration(u64);

/// Orders are monotonically numbered within one control generation.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect,
)]
pub struct OrderSequence(u64);

#[derive(
    Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect,
)]
#[reflect(Component)]
pub struct CharacterControl {
    pub mode: ControlMode,
    pub generation: ControlGeneration,
}

/// A finite scene position identifies the requested elevation and floor.
/// The server associates it with physical support at admission.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect, MapEntities)]
pub struct NavigationDestination {
    pub position: Position<Scene>,
}

/// Future destinations, executed in order after the current movement goal.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct WaypointQueue {
    destinations: Vec<NavigationDestination>,
}

#[derive(
    Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect, MapEntities,
)]
#[reflect(Component)]
pub struct CharacterOrder {
    pub sequence: OrderSequence,
    #[entities]
    pub goal: CharacterGoal,
    pub status: OrderStatus,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ControlMode {
    #[default]
    Direct,
    Indirect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Reflect, MapEntities)]
pub enum CharacterGoal {
    #[default]
    Idle,
    MoveTo(#[entities] NavigationDestination),
    Attack {
        #[entities]
        target: Entity,
    },
    Hold {
        anchor: Position<Scene>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RetaliationPolicy {
    Disabled,
    #[default]
    SelfDefense,
}

#[derive(
    Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect,
)]
#[reflect(Component)]
pub struct BehaviorPolicy {
    pub retaliation: RetaliationPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum OrderStatus {
    #[default]
    Idle,
    Executing,
    Defending,
    Completed,
    Failed(ControlRejection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Reflect, thiserror::Error)]
pub enum ControlRejection {
    #[error("character is not controlled by this connection")]
    Unauthorized,
    #[error("control generation has changed")]
    StaleGeneration,
    #[error("order has already been superseded")]
    StaleOrder,
    #[error("orders require indirect control")]
    WrongMode,
    #[error("target is unavailable")]
    InvalidTarget,
    #[error("destination has no reachable support")]
    InvalidDestination,
    #[error("no route reaches the destination")]
    Unreachable,
    #[error("navigation mesh is unavailable")]
    NavigationUnavailable,
    #[error("route is obstructed")]
    Blocked,
    #[error("character cannot make progress")]
    NoProgress,
    #[error("waypoint queue is full")]
    WaypointQueueFull,
}

impl WaypointQueue {
    /// Bound replicated order state and work requested by a client.
    pub const MAX_DESTINATIONS: usize = 32;

    pub fn destinations(&self) -> &[NavigationDestination] {
        &self.destinations
    }

    pub fn append(&mut self, destination: NavigationDestination) -> Result<(), ControlRejection> {
        if self.destinations.len() >= Self::MAX_DESTINATIONS {
            return Err(ControlRejection::WaypointQueueFull);
        }
        self.destinations.push(destination);
        Ok(())
    }

    pub fn pop_front(&mut self) -> Option<NavigationDestination> {
        if self.destinations.is_empty() {
            None
        } else {
            Some(self.destinations.remove(0))
        }
    }

    pub fn clear(&mut self) {
        self.destinations.clear();
    }
}

impl ControlGeneration {
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl OrderSequence {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl CharacterControl {
    pub fn accepts_direct(self, generation: ControlGeneration) -> bool {
        self.mode == ControlMode::Direct && self.generation == generation
    }

    pub fn switch(&mut self, mode: ControlMode) {
        self.mode = mode;
        self.generation = self.generation.next();
    }
}

impl Default for CharacterOrder {
    fn default() -> Self {
        Self {
            sequence: OrderSequence::default(),
            goal: CharacterGoal::Idle,
            status: OrderStatus::Idle,
        }
    }
}
