//! What a bot can observe: opponents, its current target, and the offensive
//! facts that decide how it can attack that target.

use bevy::ecs::{entity::MapEntities, system::SystemParam};

use super::*;

/// A strike past half of its windup is treated as already landing; guarding
/// against it no longer delays the bot's own initiative.
const INCOMING_STRIKE_PHASE_LIMIT: f32 = 0.5;

type BodyQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        &'static SkeletonState,
    ),
>;

type OpponentQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        Has<CombatantYielded>,
    ),
    With<Player>,
>;

/// The opponent a bot is currently engaging. The `AcquireTargetNode` takes it
/// from the bot's behaviour decision and drops it when no opponent remains.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, MapEntities)]
#[reflect(Component)]
pub struct BotTarget(#[entities] pub Entity);

/// A bot's view of its current target for one decision.
pub(super) struct Engagement<'a> {
    pub(super) target: Entity,
    pub(super) distance_metres: f32,
    pub(super) facts: OffensiveFacts,
    pub(super) target_skeleton: &'a SkeletonState,
    actor_skeleton: &'a SkeletonState,
}

/// A melee strike the bot can start right now.
pub(super) struct StrikePlan {
    /// Time the lunge needs to arrive; the windup lasts at least this long.
    pub(super) arrival: AnimationElapsed,
    pub(super) recovery: AnimationElapsed,
    pub(super) strike_family: StrikeFamily,
}

#[derive(SystemParam)]
pub(super) struct BotSenses<'w, 's> {
    viewer: TacticalPlayerViewer<'w, 's>,
    bodies: BodyQuery<'w, 's>,
    opponents: OpponentQuery<'w, 's>,
    targets: Query<'w, 's, &'static BotTarget>,
    decisions: Query<'w, 's, &'static crate::control::AgentState>,
    dimensions: Query<'w, 's, &'static CharacterDimensions>,
    colliders: Query<'w, 's, &'static Collider>,
    config: Res<'w, TacticalCombatConfig>,
}

impl BotSenses<'_, '_> {
    pub(super) fn offense_config(&self) -> &AiOffenseConfig {
        &self.config.ai.ordinary.offense
    }

    pub(super) fn is_incapacitated(&self, entity: Entity) -> bool {
        self.bodies
            .get(entity)
            .is_ok_and(|(_, _, state, _)| state.is_incapacitated())
    }

    pub(super) fn skeleton(&self, entity: Entity) -> Option<&SkeletonState> {
        self.bodies
            .get(entity)
            .ok()
            .map(|(_, _, _, skeleton)| skeleton)
    }

    /// The nearest opponent that can still fight. Equal distances resolve by
    /// entity identity so the choice is deterministic.
    pub(super) fn nearest_opponent(&self, actor: Entity) -> Option<Entity> {
        let (transform, side, _, _) = self.bodies.get(actor).ok()?;
        self.opponents
            .iter()
            .filter(|(candidate, _, candidate_side, state, yielded)| {
                *candidate != actor
                    && *candidate_side != side
                    && !state.is_incapacitated()
                    && !*yielded
            })
            .min_by(|(a, a_transform, ..), (b, b_transform, ..)| {
                compare_target(transform, a_transform, *a, b_transform, *b)
            })
            .map(|(candidate, ..)| candidate)
    }

    /// The opponent the bot's behaviour decision (`ControlSet::Decide`)
    /// chose to engage.
    pub(super) fn chosen_target(&self, actor: Entity) -> Option<Entity> {
        self.decisions.get(actor).ok()?.target()
    }

    pub(super) fn target(&self, actor: Entity) -> Option<Entity> {
        self.targets.get(actor).ok().map(|target| target.0)
    }

    /// Instinct shortens a bot's initiative delay. Bots without authored
    /// attributes assess like an ordinary combatant.
    pub(super) fn assessment_instinct(&self, actor: Entity) -> f32 {
        const ORDINARY_INSTINCT: f32 = 3.0;
        self.viewer.get(actor).map_or(ORDINARY_INSTINCT, |view| {
            view.raw_single_body_part_attr(SimpleAttribute::Instinct)
        })
    }

    /// Facts about the current target. Returns `None` when the bot has no
    /// target or its facts cannot be admitted (which is logged).
    pub(super) fn engagement(&self, actor: Entity) -> Option<Engagement<'_>> {
        let target = self.target(actor)?;
        let (transform, _, state, actor_skeleton) = self.bodies.get(actor).ok()?;
        let (target_transform, _, _, target_skeleton) = self.bodies.get(target).ok()?;
        let facts = offensive_facts(
            actor,
            target,
            transform,
            target_transform,
            state,
            &self.viewer,
            &self.dimensions,
            &self.colliders,
            &self.config,
            self.offense_config(),
        )
        .inspect_err(|error| warn!(?actor, %error, "cannot admit bot attack facts"))
        .ok()?;
        Some(Engagement {
            target,
            distance_metres: transform
                .translation
                .xz()
                .distance(target_transform.translation.xz()),
            facts,
            target_skeleton,
            actor_skeleton,
        })
    }

    /// Yaw that faces `target` from `actor`, if they are apart.
    pub(super) fn yaw_toward(&self, actor: Entity, target: Entity) -> Option<f32> {
        let (transform, ..) = self.bodies.get(actor).ok()?;
        let (target_transform, ..) = self.bodies.get(target).ok()?;
        let offset = target_transform.translation.xz() - transform.translation.xz();
        (offset.length_squared() > 0.0).then(|| (-offset.x).atan2(-offset.y))
    }
}

impl Engagement<'_> {
    pub(super) fn melee_target_reachable(&self) -> bool {
        matches!(
            self.facts.melee_lunge_delay,
            Some(MeleeLungeArrival::NotNeeded | MeleeLungeArrival::ArrivesIn(_))
        )
    }

    /// The strike the bot can start now: its weapon allows one, the lunge
    /// arrives, and the skeleton has a strike family available.
    pub(super) fn strike_plan(&self) -> Option<StrikePlan> {
        if !self.can_start_melee_strike() {
            return None;
        }
        let arrival = match self.facts.melee_lunge_delay? {
            MeleeLungeArrival::ArrivesIn(arrival) => arrival,
            MeleeLungeArrival::NotNeeded => AnimationElapsed::ZERO,
            MeleeLungeArrival::CannotArrive => return None,
        };
        let Some(RecoveryTime::After(recovery)) = self.facts.melee_recovery_seconds else {
            return None;
        };
        Some(StrikePlan {
            arrival,
            recovery,
            strike_family: self
                .actor_skeleton
                .available_strike_family(self.facts.strike_family)?,
        })
    }

    /// Movement during a melee windup: a long weapon backs off while the
    /// target is inside its preferred measure, otherwise the bot holds.
    pub(super) fn windup_movement(&self, config: &AiOffenseConfig) -> Option<Vec2> {
        long_weapon_windup_movement(
            self.facts.weapon_reach,
            self.facts.preferred_melee_measure,
            self.distance_metres,
            config.long_weapon_measure_threshold_metres,
        )
    }

    /// Whether the bot's weapon allows a melee strike at the target, with a
    /// lunge if needed.
    pub(super) fn can_start_melee_strike(&self) -> bool {
        self.facts.weapon_is_melee
            && self.facts.melee_attack_available
            && matches!(
                self.facts.melee_recovery_seconds,
                Some(RecoveryTime::After(_))
            )
            && self.facts.dimensions.arm_reach_metres > 0.0
            && self.melee_target_reachable()
    }
}

/// Whether `skeleton` is in the first half of a committed strike.
pub(super) fn strike_is_incoming(skeleton: &SkeletonState) -> bool {
    skeleton.action_kind() == SkeletonAction::Attack
        && skeleton.action_phase().fraction() < INCOMING_STRIKE_PHASE_LIMIT
}
