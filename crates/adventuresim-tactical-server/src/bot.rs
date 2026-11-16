//! Server-controlled combatants.
//!
//! Each bot runs one authored behaviour tree, attached as a [`kisya_edbt`]
//! `Behaviour` when it spawns. The trees live in `content/tactical/ai/` (see
//! [`trees`]) and combine library nodes with the game-specific nodes in
//! [`nodes`].

use adventuresim_core::{
    combat::RecoveryTime,
    elapsed::{AnimationElapsed, ElapsedResult},
};
use adventuresim_tactical_core::animation::AnimationValueResult;

use adventuresim_core::{
    combat::{
        has_distal_striking_surface, melee_attack_capability, preferred_melee_striking_measure,
    },
    fixture_path::resolve_fixture_path,
    item_references::ARROW_ID,
    tactical_fixture::TacticalEnemyFixture,
};
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::FromClient,
    message::{DefendRequest, MeleeActionRequest},
};
use bevy::prelude::*;
use fabelgeist_determinism::DeterministicRng;
use fabelgeist_determinism::Seed;
use kisya_edbt::prelude::*;
use std::{cmp::Ordering, path::PathBuf};

use crate::{
    Args,
    combat::{
        CombatDuration, CombatSet, DefendIntent, MeleeAttackStartedIntent, RangedAttackIntent,
        RangedAttackStartedIntent, ReportedPrecision, TacticalCombatSide,
    },
    player_projection::{AuthoritativeMovementIntent, begin_get_up_transition_configured},
};

pub use defense::{DefenseChances, ReactiveDefenseAi};
use defense::{
    on_attack_started, on_targeted_attack_started, on_targeted_ranged_attack_started,
    tick_bot_reactions,
};
pub use offense::InitiativeLost;
use offense::long_weapon_windup_movement;
#[cfg(test)]
use offense::ranged_weapon_needs_ammo_lookup;
use offense::{OffensiveFacts, compare_target, offensive_facts};
pub use senses::BotTarget;
pub use trees::BotTreeError;
#[cfg(feature = "debug")]
pub(crate) use trees::start_restored_bots;
pub(crate) use trees::{BotTrees, PASSIVE_TREE, STANDARD_COMBAT_TREE};

mod defense;
mod nodes;
mod offense;
mod senses;
pub(crate) mod trees;

/// Marks a server-controlled bot filling in for a temporary (non-connected)
/// mission character.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MissionEnemy;

#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
pub(crate) struct CombatantYielded;

#[derive(Resource)]
pub(crate) struct CombatRandom(DeterministicRng);

#[derive(Event, Clone, Copy, Debug)]
pub(crate) struct BotContinuationDecisionEvent {
    pub(crate) combatant: Entity,
    pub(crate) decision: BotContinuationDecision,
}

/// Runs server-controlled combatants. Requires `AssetPlugin`, which stores the
/// behaviour trees.
pub struct BotPlugin;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BotContinuationDecision {
    Withdraw,
    Yield,
}

impl Default for CombatRandom {
    fn default() -> Self {
        Self::seeded(rand::random())
    }
}

impl CombatRandom {
    pub(crate) fn seeded(seed: Seed) -> Self {
        Self(fabelgeist_determinism::StreamId::new("combat.tactical").rng(seed, &[]))
    }

    pub(crate) fn unit_f32(&mut self) -> f32 {
        self.0.unit_f32()
    }

    pub(crate) fn unit_f64(&mut self) -> f64 {
        self.0.unit_f64()
    }

    pub(crate) fn range_f32(&mut self, minimum: f32, maximum: f32) -> f32 {
        self.0.range_f32(minimum, maximum)
    }

    pub(crate) fn coin_flip(&mut self) -> bool {
        self.0.boolean()
    }
}

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TacticalCombatConfig>()
            .init_resource::<CombatRandom>()
            .init_resource::<trees::BotTrees>()
            .add_plugins((BehaviourPlugins, nodes::BotNodesPlugin))
            .register_type::<BotTarget>()
            .register_type::<InitiativeLost>()
            .add_observer(on_attack_started)
            .add_observer(on_targeted_attack_started)
            .add_observer(on_targeted_ranged_attack_started)
            .add_observer(offense::on_attack_committed_to_defense)
            .add_systems(Startup, trees::preload_bot_trees)
            .add_systems(Update, tick_bot_reactions.after(CombatSet::Condition));
        #[cfg(feature = "iteration")]
        app.add_observer(trace_continuation_decision);
    }
}

pub(crate) fn resolve_scene_fixture(selector: &str) -> Result<PathBuf, String> {
    Ok(resolve_fixture_path(
        selector,
        "assets/tactical-scenes",
        "json",
    ))
}

pub(crate) fn load_enemy_fixture(selector: &str) -> Result<TacticalEnemyFixture, String> {
    let path = resolve_fixture_path(selector, "assets/tactical-enemies", "yaml");
    TacticalEnemyFixture::load(&path)
}

pub(crate) fn apply_enemy_fixture(mut args: Args) -> Args {
    if let Some(fixture) = &args.enemy_fixture {
        args.required_enemy_kills = fixture.enemy_count();
    }
    args
}

#[cfg(feature = "iteration")]
fn trace_continuation_decision(event: On<BotContinuationDecisionEvent>) {
    trace!(combatant = ?event.combatant, decision = ?event.decision, "bot_continuation_decision");
}

#[cfg(test)]
mod tests;
