//! Server-owned control arbitration and autonomous order execution.

use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{control::*, prelude::*, scene_coordinates::Scene};
use adventuresim_tactical_netcode::{bevy_replicon::prelude::*, control::*};
use bevy::prelude::*;

use crate::{
    bot::CombatantBehaviorPackages,
    combat::{CombatInstant, CombatSet},
    navigation::NavigationWorld,
    player_projection::{AuthoritativeMovementIntent, AuthoritativePostureIntent},
};

use behavior::{follow_routes, prepare_behavior};
pub(crate) use conversation::{ReachedTalkTarget, TalkContactWorld, TalkParticipants};
use orders::{on_control_request, on_direct_action};
pub(crate) use perception::{AcceptedAttack, ThreatMemory, observe_attack, observe_damage};

mod behavior;
mod conversation;
mod navigation;
mod orders;
pub(crate) mod perception;

#[cfg(test)]
pub(crate) mod movement_tests;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ControlSet {
    Decide,
    Execute,
    Navigate,
}

#[derive(Component, Default)]
pub(crate) struct AgentState {
    pub(crate) decision: BehaviorDecision,
    last_progress: Option<ProgressObservation>,
}

struct ProgressObservation {
    position: Position<Scene>,
    at: CombatInstant,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum BehaviorDecision {
    #[default]
    Idle,
    Travel(NavigationDestination),
    Engage {
        target: Entity,
        pursuit: PursuitPolicy,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PursuitPolicy {
    Allowed,
    Forbidden,
}

/// Authored NPC packages can initiate combat; player defaults only retaliate.
#[derive(Component, Clone, Copy, Default)]
pub(crate) enum EngagementPolicy {
    #[default]
    Retaliate,
    SeekOpponents,
}

pub(crate) struct CharacterControlPlugin;

impl Plugin for CharacterControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::navigation::TacticalNavigationPlugin,
            crate::dialogue::TacticalDialoguePlugin,
        ))
        .add_message::<conversation::ReachedTalkTarget>()
        .register_required_components::<Player, AgentState>()
        .register_required_components::<Player, ThreatMemory>()
        .register_required_components::<Player, EngagementPolicy>()
        .configure_sets(
            Update,
            (
                ControlSet::Decide,
                ControlSet::Execute,
                ControlSet::Navigate,
            )
                .chain()
                .after(CombatSet::Condition),
        )
        .add_observer(crate::player_projection::apply_movement_action)
        .add_observer(on_control_request)
        .add_observer(on_direct_action)
        .add_observer(observe_attack)
        .add_observer(observe_damage)
        .add_systems(
            Update,
            navigation::sync_navigation_agents
                .after(ControlSet::Execute)
                .before(bevy_landmass::LandmassSystems::SyncExistence),
        )
        .configure_sets(
            Update,
            ControlSet::Navigate.after(bevy_landmass::LandmassSystems::Output),
        )
        .add_systems(Update, prepare_behavior.in_set(ControlSet::Decide))
        .add_systems(
            Update,
            conversation::prepare_approaches
                .in_set(ControlSet::Decide)
                .after(prepare_behavior),
        )
        .add_systems(Update, follow_routes.in_set(ControlSet::Navigate));
    }
}

impl AgentState {
    pub(crate) fn target(&self) -> Option<Entity> {
        match self.decision {
            BehaviorDecision::Engage { target, .. } => Some(target),
            _ => None,
        }
    }
}

pub(crate) fn indirect_packages(config: &TacticalCombatConfig) -> CombatantBehaviorPackages {
    CombatantBehaviorPackages::standard_combat(config)
}

pub(crate) fn control_filter(
    actors: &Query<Entity, With<Player>>,
    items: &Query<Entity, With<TacticalSceneItem>>,
) -> SpatialQueryFilter {
    SpatialQueryFilter::from_excluded_entities(actors.iter().chain(items.iter()))
}

#[cfg(test)]
mod tests;
