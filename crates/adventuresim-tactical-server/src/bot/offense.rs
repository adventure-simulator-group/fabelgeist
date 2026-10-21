use bevy::ecs::entity::MapEntities;
use bevy::ecs::system::SystemParam;

use super::*;
use fabelgeist_geometry::SignedLength;

pub(super) use facts::compare_target;
#[cfg(test)]
pub(super) use facts::ranged_weapon_needs_ammo_lookup;
use facts::{OffensiveFacts, offensive_facts};
use phase::{OffensiveCombatPhase, drive_offensive_phase};
use ranged::{RangedPursuit, drive_ranged_pursuit};
use tactics::*;

mod facts;
mod phase;
mod ranged;
mod tactics;

type OffensiveAiQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static Transform,
        &'static TacticalCombatSide,
        &'static mut CharacterLook,
        &'static mut AuthoritativeMovementIntent,
        &'static mut OffensiveCombatAi,
        &'static TacticalCombatState,
        &'static SkeletonState,
        Option<&'static CombatantYielded>,
    ),
>;

type OffensiveCandidateQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static Transform,
        &'static TacticalCombatSide,
        &'static TacticalCombatState,
        &'static SkeletonState,
        Option<&'static CombatantYielded>,
    ),
    With<Player>,
>;

#[derive(SystemParam)]
pub(super) struct OffensiveAiContext<'w, 's> {
    ai: OffensiveAiQuery<'w, 's>,
    colliders: Query<'w, 's, &'static Collider>,
    dimensions: Query<'w, 's, &'static CharacterDimensions>,
    combat_config: Res<'w, TacticalCombatConfig>,
}

/// Enables server-owned offensive control, preferring ranged fire while a
/// usable ranged weapon and arrows are available and otherwise using melee.
#[derive(Component, Debug, Reflect, MapEntities)]
#[reflect(Component)]
pub struct OffensiveCombatAi {
    #[entities]
    target: Option<Entity>,
    phase: OffensiveCombatPhase,
}

impl Default for OffensiveCombatAi {
    fn default() -> Self {
        Self {
            target: None,
            phase: OffensiveCombatPhase::Pursuing,
        }
    }
}

#[cfg(test)]
impl OffensiveCombatAi {
    pub(super) fn target(&self) -> Option<Entity> {
        self.target
    }

    pub(super) fn is_assessing(&self) -> bool {
        matches!(self.phase, OffensiveCombatPhase::Assessing(_))
    }

    pub(super) fn guard_committed_threat_for_test(&mut self, target: Entity) {
        self.target = Some(target);
        self.phase = OffensiveCombatPhase::GuardingCommittedThreat;
    }
}

pub(super) fn drive_offensive_combat_ai(
    mut cmd: Commands,
    time: Res<Time<()>>,
    viewer: TacticalPlayerViewer,
    candidates: OffensiveCandidateQuery<'_, '_>,
    context: OffensiveAiContext<'_, '_>,
    mut random: ResMut<CombatRandom>,
) {
    let OffensiveAiContext {
        mut ai,
        colliders,
        dimensions,
        combat_config,
    } = context;
    let config = &combat_config.ai.ordinary.offense;
    for (entity, transform, side, mut look, mut input, mut controller, state, skeleton, yielded) in
        &mut ai
    {
        if state.is_incapacitated() || yielded.is_some() {
            input.0 = None;
            continue;
        }
        let target = candidates
            .iter()
            .filter(
                |(candidate, _, candidate_side, candidate_state, _, candidate_yielded)| {
                    *candidate != entity
                        && **candidate_side != *side
                        && !candidate_state.is_incapacitated()
                        && candidate_yielded.is_none()
                },
            )
            .min_by(
                |(a, a_transform, _, _, _, _), (b, b_transform, _, _, _, _)| {
                    compare_target(transform, a_transform, *a, b_transform, *b)
                },
            )
            .map(|(target, _, _, _, _, _)| target);

        if target != controller.target {
            let instinct = viewer.get(entity).map_or(3.0, |view| {
                view.raw_single_body_part_attr(SimpleAttribute::Instinct)
            });
            let delay = initiative_delay_seconds(&mut random, instinct, config)
                .and_then(AnimationElapsed::native_duration);
            let delay = match delay {
                Ok(delay) => delay,
                Err(error) => {
                    warn!(?entity, %error, "cannot schedule bot initiative");
                    input.0 = None;
                    continue;
                }
            };
            controller.target = target;
            controller.phase = OffensiveCombatPhase::Assessing(Timer::new(delay, TimerMode::Once));
        }
        let Some(target) = target else {
            input.0 = None;
            continue;
        };
        let Ok((_, target_transform, _, _, target_skeleton, _)) = candidates.get(target) else {
            continue;
        };
        let offset = target_transform.translation.xz() - transform.translation.xz();
        let distance = match SignedLength::from_metres(offset.length()) {
            Ok(distance) => distance,
            Err(error) => {
                warn!(?entity, %error, "cannot admit bot target separation");
                input.0 = None;
                continue;
            }
        };
        if distance.metres() > 0.0 {
            look.yaw = (-offset.x).atan2(-offset.y);
        }
        let facts = match offensive_facts(
            entity,
            target,
            transform,
            target_transform,
            state,
            &viewer,
            &dimensions,
            &colliders,
            &combat_config,
            config,
        ) {
            Ok(facts) => facts,
            Err(error) => {
                warn!(?entity, %error, "cannot admit bot attack facts");
                input.0 = None;
                continue;
            }
        };
        let melee_target_reachable = matches!(
            facts.melee_lunge_delay,
            Some(MeleeLungeArrival::NotNeeded | MeleeLungeArrival::ArrivesIn(_))
        );

        let abort_windup = matches!(
            &controller.phase,
            OffensiveCombatPhase::MeleeWindup { .. }
                if !facts.weapon_is_melee
                    || !facts.melee_attack_available
                    || facts.dimensions.arm_reach_metres.metres() <= 0.0
                    || !melee_target_reachable
                    || !matches!(facts.melee_recovery_seconds, Some(RecoveryTime::After(_)))
        ) || matches!(
            &controller.phase,
            OffensiveCombatPhase::RangedWindup(_) if !facts.use_ranged || distance.metres() > facts.weapon_reach.metres()
        );
        if abort_windup {
            controller.phase = OffensiveCombatPhase::Pursuing;
        }
        if matches!(
            controller.phase,
            OffensiveCombatPhase::WithdrawingUnableToContinue(_)
        ) && facts.melee_attack_available
        {
            controller.phase = OffensiveCombatPhase::Pursuing;
        }

        if let Err(error) = drive_offensive_phase(
            &mut cmd,
            &time,
            entity,
            target,
            distance,
            &mut input,
            &mut controller,
            skeleton,
            target_skeleton,
            &facts,
            config,
            &mut random,
        ) {
            warn!(?entity, %error, "cannot schedule bot attack phase");
            input.0 = None;
        }
    }
}

pub(super) fn on_attack_committed_to_defense(
    event: On<crate::combat::MeleeAttackCommittedToDefense>,
    viewer: TacticalPlayerViewer,
    config: Res<TacticalCombatConfig>,
    mut random: ResMut<CombatRandom>,
    mut controllers: Query<&mut OffensiveCombatAi>,
) {
    let Ok(mut controller) = controllers.get_mut(event.defender) else {
        return;
    };
    let instinct = viewer.get(event.defender).map_or(3.0, |view| {
        view.raw_single_body_part_attr(SimpleAttribute::Instinct)
    });
    let delay = initiative_delay_seconds(&mut random, instinct, &config.ai.ordinary.offense)
        .and_then(AnimationElapsed::native_duration);
    match delay {
        Ok(delay) => {
            controller.phase = OffensiveCombatPhase::Assessing(Timer::new(delay, TimerMode::Once))
        }
        Err(error) => warn!(%error, "cannot schedule bot counterattack initiative"),
    }
}

#[cfg(test)]
mod tests {
    use super::{below_preferred_long_weapon_measure, committed_threat_recognition_probability};
    use adventuresim_tactical_core::animation::UnitInterval;
    use fabelgeist_geometry::SignedLength;

    #[test]
    fn threat_recognition_grows_with_visible_commitment_and_instinct() {
        let early = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.05).unwrap(),
            3.0,
        );
        let late = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.4).unwrap(),
            3.0,
        );
        let expert = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.4).unwrap(),
            5.0,
        );
        assert!(early < late);
        assert!(late < expert);
        assert!((0.0..=1.0).contains(&early));
        assert!((0.0..=1.0).contains(&expert));
    }

    #[test]
    fn polearm_retreats_below_authored_head_band_center_not_generic_seventy_percent() {
        let preferred =
            adventuresim_core::combat::preferred_melee_striking_measure(
                SignedLength::from_metres(2.0).unwrap(),
                SignedLength::from_metres(1.9).unwrap(),
                SignedLength::from_metres(0.16).unwrap(), true, 0.7,
            ).unwrap();
        assert!((preferred.metres() - 1.92).abs() < 1.0e-6);
        assert!(below_preferred_long_weapon_measure(
            SignedLength::from_metres(2.0).unwrap(), preferred,
            SignedLength::from_metres(2.6).unwrap(), SignedLength::from_metres(1.2).unwrap()
        ));
        assert!(!below_preferred_long_weapon_measure(
            SignedLength::from_metres(2.0).unwrap(), preferred,
            SignedLength::from_metres(2.8).unwrap(), SignedLength::from_metres(1.2).unwrap()
        ));
    }
}
