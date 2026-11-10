use adventuresim_core::combat::ContactTimingOffset;
use adventuresim_core::elapsed::AnimationElapsed;
#[cfg(test)]
use std::time::Duration;

pub(crate) use clock::{CombatDuration, CombatInstant};

use adventuresim_tactical_core::prelude::{BodyPart, MeleeAttackKey, melee_interaction_range};
use bevy::prelude::*;

use super::{
    MeleeIntentFacts, MeleeIntentRejection, RangedIntentFacts, RangedIntentRejection,
    TacticalCombatSide,
};

pub(crate) use ranged::RangedAttackAuthority;

mod clock;
mod ranged;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ReportedPrecision(f32);

pub(crate) struct TransformedMeleeAttack {
    pub(crate) attack_key: MeleeAttackKey,
    pub(crate) retained_power: f32,
}

impl ReportedPrecision {
    /// Accepts every finite client report exactly as supplied. Precision is a
    /// deliberately trusted animation boundary and is not geometrically
    /// reconstructed or range-clamped by the headless server.
    pub(crate) fn new(value: f32) -> Option<Self> {
        value.is_finite().then_some(Self(value))
    }

    pub(crate) fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ValidatedMeleeAttack {
    attacker: Entity,
    target: Entity,
    pub(super) body_part: BodyPart,
    reported_precision: ReportedPrecision,
    attacker_position: Vec3,
    target_position: Vec3,
    attacker_yaw: f32,
    target_yaw: f32,
}

impl ValidatedMeleeAttack {
    pub(super) fn attacker(&self) -> Entity {
        self.attacker
    }
    pub(super) fn target(&self) -> Entity {
        self.target
    }
    pub(super) fn attacker_position(&self) -> Vec3 {
        self.attacker_position
    }
    pub(super) fn target_position(&self) -> Vec3 {
        self.target_position
    }
    pub(super) fn reported_precision(&self) -> ReportedPrecision {
        self.reported_precision
    }
    pub(super) fn attacker_yaw(&self) -> f32 {
        self.attacker_yaw
    }
    pub(super) fn target_yaw(&self) -> f32 {
        self.target_yaw
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AuthorizedMeleeAttack {
    attack_key: MeleeAttackKey,
    attack: ValidatedMeleeAttack,
    power_multiplier: f32,
}

impl AuthorizedMeleeAttack {
    pub(super) fn attack_key(&self) -> MeleeAttackKey {
        self.attack_key
    }
    pub(super) fn attacker(&self) -> Entity {
        self.attack.attacker
    }
    pub(super) fn target(&self) -> Entity {
        self.attack.target
    }
    pub(super) fn body_part(&self) -> BodyPart {
        self.attack.body_part
    }
    pub(super) fn reported_precision(&self) -> ReportedPrecision {
        self.attack.reported_precision
    }
    pub(super) fn power_multiplier(&self) -> f32 {
        self.power_multiplier
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ValidatedRangedImpact {
    Miss,
    Hit {
        target: Entity,
        body_part: BodyPart,
        target_position: Vec3,
        target_yaw: f32,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ValidatedRangedShot {
    attacker: Entity,
    attacker_side: TacticalCombatSide,
    attacker_position: Vec3,
    attacker_yaw: f32,
    reported_precision: ReportedPrecision,
    impact: ValidatedRangedImpact,
}

impl ValidatedRangedShot {
    pub(super) fn attacker(&self) -> Entity {
        self.attacker
    }
    pub(super) fn attacker_position(&self) -> Vec3 {
        self.attacker_position
    }
    pub(super) fn impact(&self) -> ValidatedRangedImpact {
        self.impact
    }
    pub(super) fn attacker_yaw(&self) -> f32 {
        self.attacker_yaw
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AuthorizedRangedShot(ValidatedRangedShot);

impl AuthorizedRangedShot {
    pub(super) fn attacker(&self) -> Entity {
        self.0.attacker
    }
    pub(super) fn attacker_side(&self) -> TacticalCombatSide {
        self.0.attacker_side
    }
    pub(super) fn reported_precision(&self) -> ReportedPrecision {
        self.0.reported_precision
    }
    pub(super) fn impact(&self) -> ValidatedRangedImpact {
        self.0.impact
    }
}

#[derive(Debug, Clone)]
struct ObservedMeleeWindup {
    attack_key: MeleeAttackKey,
    target: Option<Entity>,
    body_part: Option<BodyPart>,
    ready_at: CombatInstant,
    expires_at: CombatInstant,
}

#[derive(Debug, Clone)]
struct ActiveMeleeAttack {
    target: Option<Entity>,
    started_at: CombatInstant,
    reported_precision: ReportedPrecision,
    power_multiplier: f32,
    scheduled_measure_metres: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub(crate) struct MeleeAttackAuthority {
    windup: Option<ObservedMeleeWindup>,
    active: Option<ActiveMeleeAttack>,
    cooldown_until: CombatInstant,
    consecutive_intercepts: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ReciprocalAttackOpportunity {
    pub input_reflex: f32,
    pub precision: ReportedPrecision,
    pub own_contact_after_incoming_seconds: ContactTimingOffset,
    pub own_windup_seconds: AnimationElapsed,
    pub decision_sample: f32,
    pub consecutive_intercepts: u8,
}

impl MeleeAttackAuthority {
    #[expect(
        clippy::too_many_arguments,
        reason = "an observed attack records each independent authoritative timing and targeting fact"
    )]
    pub(crate) fn observe(
        &mut self,
        attack_key: MeleeAttackKey,
        target: Option<Entity>,
        body_part: Option<BodyPart>,
        now: CombatInstant,
        windup: CombatDuration,
        network_allowance: CombatDuration,
        scheduled_measure_metres: f32,
        reported_precision: ReportedPrecision,
    ) {
        let ready_at = now + windup;
        self.windup = Some(ObservedMeleeWindup {
            attack_key,
            target,
            body_part,
            ready_at,
            expires_at: ready_at + network_allowance,
        });
        self.active = Some(ActiveMeleeAttack {
            target,
            started_at: now,
            reported_precision,
            power_multiplier: 1.0,
            scheduled_measure_metres,
        });
    }

    pub(crate) fn attack_key(&self) -> Option<MeleeAttackKey> {
        self.windup.as_ref().map(|windup| windup.attack_key)
    }

    pub(super) fn started_at(&self) -> Option<CombatInstant> {
        self.active.as_ref().map(|attack| attack.started_at)
    }

    pub(crate) fn scheduled_measure_metres(&self) -> Option<f32> {
        self.active
            .as_ref()
            .map(|attack| attack.scheduled_measure_metres)
    }

    pub(crate) fn complete_miss(&mut self) -> Option<MeleeAttackKey> {
        self.active = None;
        self.windup.take().map(|windup| windup.attack_key)
    }

    pub(crate) fn commit_attack_to_defense(&mut self) -> Option<MeleeAttackKey> {
        self.consecutive_intercepts = self.consecutive_intercepts.saturating_add(1);
        self.complete_miss()
    }

    pub(crate) fn preserve_attack_for_trade(&mut self) {
        self.consecutive_intercepts = self.consecutive_intercepts.saturating_sub(1);
    }

    pub(crate) fn transform_attack_for_offhand_defense(
        &mut self,
        effectiveness: f32,
    ) -> Option<TransformedMeleeAttack> {
        let active = self.active.as_mut()?;
        let attack_key = self.windup.as_ref()?.attack_key;
        // Keeping the sword committed while the off hand intercepts costs
        // trunk rotation and grip support. A complete buckler interception
        // retains sixty percent of the already-worked strike; weaker contacts
        // disturb it proportionally less.
        let retained = 1.0 - 0.4 * effectiveness.clamp(0.0, 1.0);
        active.power_multiplier *= retained;
        Some(TransformedMeleeAttack {
            attack_key,
            retained_power: active.power_multiplier,
        })
    }

    pub(super) fn authorize_attack(
        &mut self,
        attack: ValidatedMeleeAttack,
        now: CombatInstant,
        cooldown: CombatDuration,
    ) -> Option<AuthorizedMeleeAttack> {
        let attack_key = self.attack_key()?;
        let power_multiplier = self.authorize(attack.target, attack.body_part, now, cooldown)?;
        Some(AuthorizedMeleeAttack {
            attack_key,
            attack,
            power_multiplier,
        })
    }

    pub(crate) fn permits(&self, target: Entity, body_part: BodyPart, now: CombatInstant) -> bool {
        self.windup.as_ref().is_some_and(|windup| {
            windup.target.is_none_or(|observed| observed == target)
                && windup
                    .body_part
                    .is_none_or(|observed| observed == body_part)
                && now >= windup.ready_at
                && now <= windup.expires_at
                && now >= self.cooldown_until
        })
    }

    pub(super) fn reciprocal_attack_opportunity(
        &self,
        incoming_attacker: Entity,
        incoming_started_at: CombatInstant,
        now: CombatInstant,
        reflex_window: CombatDuration,
    ) -> Option<ReciprocalAttackOpportunity> {
        let active = self.active.as_ref()?;
        let windup = self.windup.as_ref()?;
        if active.target != Some(incoming_attacker) || active.started_at <= incoming_started_at {
            return None;
        }
        let delay = active.started_at.elapsed_since(incoming_started_at);
        let window = reflex_window.as_secs_f32();
        (delay.as_secs_f32() <= window).then(|| {
            let decision_sample = fabelgeist_determinism::Seed::derive(
                &windup.attack_key.to_le_bytes(),
                fabelgeist_determinism::StreamId::new("combat.reciprocal-attack"),
                &[&active.started_at.native_nanos().to_le_bytes()],
            )
            .rng()
            .inclusive_unit_f32();
            ReciprocalAttackOpportunity {
                input_reflex: (1.0 - delay.as_secs_f32() / window).clamp(0.0, 1.0),
                precision: active.reported_precision,
                own_contact_after_incoming_seconds: ContactTimingOffset::from(
                    windup.ready_at.elapsed_since(now),
                ),
                own_windup_seconds: AnimationElapsed::from(
                    windup.ready_at.elapsed_since(active.started_at),
                ),
                decision_sample,
                consecutive_intercepts: self.consecutive_intercepts,
            }
        })
    }
    fn authorize(
        &mut self,
        target: Entity,
        body_part: BodyPart,
        now: CombatInstant,
        cooldown: CombatDuration,
    ) -> Option<f32> {
        let authorized = self.windup.as_ref().and_then(|windup| {
            (windup.target.is_none_or(|observed| observed == target)
                && windup
                    .body_part
                    .is_none_or(|observed| observed == body_part)
                && now >= windup.ready_at
                && now <= windup.expires_at
                && now >= self.cooldown_until)
                .then(|| {
                    let active = self
                        .active
                        .as_ref()
                        .expect("an observed windup has active attack metadata");
                    active.power_multiplier
                })
        });
        if authorized.is_some() {
            self.windup = None;
            self.active = None;
            self.cooldown_until = now + cooldown;
        }
        authorized
    }
}

#[cfg(test)]
#[expect(
    clippy::items_after_test_module,
    reason = "authority boundary tests stay next to the state machines they specify"
)]
mod tests {
    use super::*;

    const SECOND: CombatDuration = CombatDuration::from_duration(Duration::from_secs(1));

    #[test]
    fn precision_accepts_all_finite_reports_without_clamping() {
        assert_eq!(ReportedPrecision::new(-1.0).unwrap().get(), -1.0);
        assert_eq!(ReportedPrecision::new(99.0).unwrap().get(), 99.0);
        assert!(ReportedPrecision::new(f32::NAN).is_none());
        assert!(ReportedPrecision::new(f32::INFINITY).is_none());
    }

    #[test]
    fn authority_boundaries_are_inclusive_and_duration_typed() {
        let target = Entity::from_bits(7);
        let start = CombatInstant::from_duration(Duration::ZERO);
        let mut authority = MeleeAttackAuthority::default();
        authority.observe(
            MeleeAttackKey::from(7),
            Some(target),
            Some(BodyPart::Chest),
            start,
            SECOND,
            SECOND,
            1.0,
            ReportedPrecision::new(0.75).unwrap(),
        );
        assert!(!authority.permits(target, BodyPart::Chest, start));
        assert!(!authority.permits(target, BodyPart::Head, start + SECOND));
        assert!(authority.permits(target, BodyPart::Chest, start + SECOND));
        assert!(authority.permits(target, BodyPart::Chest, start + SECOND + SECOND));
    }

    #[test]
    fn client_miss_terminates_the_correlated_windup() {
        let target = Entity::from_bits(7);
        let mut authority = MeleeAttackAuthority::default();
        authority.observe(
            MeleeAttackKey::from(42),
            Some(target),
            Some(BodyPart::Head),
            CombatInstant::from_duration(Duration::ZERO),
            SECOND,
            SECOND,
            1.0,
            ReportedPrecision::new(0.75).unwrap(),
        );
        assert_eq!(authority.complete_miss(), Some(MeleeAttackKey::from(42)));
        assert_eq!(authority.attack_key(), None);
    }

    #[test]
    fn only_later_reciprocal_attacks_supply_parry_reflex_and_precision() {
        let incoming_attacker = Entity::from_bits(7);
        let other_target = Entity::from_bits(9);
        let incoming_started_at = CombatInstant::from_duration(Duration::ZERO);
        let later_started_at = CombatInstant::from_duration(Duration::from_millis(100));
        let precision = ReportedPrecision::new(0.25).unwrap();
        let mut authority = MeleeAttackAuthority::default();
        authority.observe(
            MeleeAttackKey::from(8),
            Some(incoming_attacker),
            Some(BodyPart::Chest),
            later_started_at,
            SECOND,
            SECOND,
            1.0,
            precision,
        );

        let opportunity = authority
            .reciprocal_attack_opportunity(
                incoming_attacker,
                incoming_started_at,
                later_started_at + SECOND,
                CombatDuration::from_duration(Duration::from_millis(500)),
            )
            .unwrap();
        assert_eq!(opportunity.input_reflex, 0.8);
        assert_eq!(opportunity.precision, precision);
        assert!(
            authority
                .reciprocal_attack_opportunity(
                    incoming_attacker,
                    later_started_at,
                    later_started_at + SECOND,
                    SECOND,
                )
                .is_none()
        );
        assert!(
            authority
                .reciprocal_attack_opportunity(
                    other_target,
                    incoming_started_at,
                    later_started_at + SECOND,
                    SECOND,
                )
                .is_none()
        );
    }

    #[test]
    fn committing_a_reciprocal_attack_to_parry_consumes_its_contact() {
        let incoming_attacker = Entity::from_bits(7);
        let start = CombatInstant::from_duration(Duration::from_millis(100));
        let mut authority = MeleeAttackAuthority::default();
        authority.observe(
            MeleeAttackKey::from(19),
            Some(incoming_attacker),
            Some(BodyPart::Chest),
            start,
            SECOND,
            SECOND,
            1.0,
            ReportedPrecision::new(0.75).unwrap(),
        );
        assert!(
            authority
                .reciprocal_attack_opportunity(
                    incoming_attacker,
                    CombatInstant::from_duration(Duration::ZERO),
                    start + SECOND,
                    SECOND,
                )
                .is_some()
        );
        assert_eq!(
            authority.commit_attack_to_defense(),
            Some(MeleeAttackKey::from(19))
        );
        assert!(!authority.permits(incoming_attacker, BodyPart::Chest, start + SECOND));
        assert!(
            authority
                .reciprocal_attack_opportunity(
                    incoming_attacker,
                    CombatInstant::from_duration(Duration::ZERO),
                    start + SECOND,
                    SECOND,
                )
                .is_none()
        );
    }

    #[test]
    fn offhand_intercept_preserves_but_depowers_committed_attack() {
        let attacker = Entity::from_bits(3);
        let target = Entity::from_bits(7);
        let start = CombatInstant::from_duration(Duration::ZERO);
        let mut authority = MeleeAttackAuthority::default();
        authority.observe(
            MeleeAttackKey::from(23),
            Some(target),
            Some(BodyPart::Chest),
            start,
            SECOND,
            SECOND,
            1.0,
            ReportedPrecision::new(0.75).unwrap(),
        );
        let TransformedMeleeAttack {
            attack_key: key,
            retained_power: retained,
        } = authority.transform_attack_for_offhand_defense(1.0).unwrap();
        assert_eq!(key, MeleeAttackKey::from(23));
        assert_eq!(retained, 0.6);
        let authorized = authority
            .authorize_attack(
                ValidatedMeleeAttack {
                    attacker,
                    target,
                    body_part: BodyPart::Chest,
                    reported_precision: ReportedPrecision::new(0.75).unwrap(),
                    attacker_position: Vec3::ZERO,
                    target_position: Vec3::X,
                    attacker_yaw: 0.0,
                    target_yaw: 0.0,
                },
                start + SECOND,
                SECOND,
            )
            .unwrap();
        assert_eq!(authorized.power_multiplier(), 0.6);
    }
}

pub(super) fn validate_ranged_intent(
    facts: RangedIntentFacts,
) -> Result<ValidatedRangedShot, RangedIntentRejection> {
    if facts.target == Some(facts.attacker) {
        return Err(RangedIntentRejection::SelfTarget);
    }
    let Some(attacker_side) = facts.attacker_side else {
        return Err(RangedIntentRejection::MissingSide);
    };
    let Some(attacker_incapacitated) = facts.attacker_incapacitated else {
        return Err(RangedIntentRejection::MissingCombatState);
    };
    if attacker_incapacitated {
        return Err(RangedIntentRejection::Incapacitated);
    }
    if !facts.weapon_is_ranged || !facts.weapon_range.is_finite() || facts.weapon_range <= 0.0 {
        return Err(RangedIntentRejection::NotRanged);
    }
    if facts.target.is_some() {
        let Some(target_side) = facts.target_side else {
            return Err(RangedIntentRejection::MissingSide);
        };
        let Some(target_incapacitated) = facts.target_incapacitated else {
            return Err(RangedIntentRejection::MissingCombatState);
        };
        if attacker_side == target_side {
            return Err(RangedIntentRejection::FriendlyTarget);
        }
        if target_incapacitated {
            return Err(RangedIntentRejection::Incapacitated);
        }
        if !facts.separation.is_some_and(|distance| {
            distance.is_finite() && distance <= facts.weapon_range + facts.range_latency_tolerance
        }) {
            return Err(RangedIntentRejection::OutOfRange);
        }
        if facts.target_in_aim_cone != Some(true) {
            return Err(RangedIntentRejection::OutsideAimCone);
        }
    }
    if !facts.authority_permits {
        return Err(RangedIntentRejection::Windup);
    }
    let impact = match facts.target {
        None => ValidatedRangedImpact::Miss,
        Some(target) => ValidatedRangedImpact::Hit {
            target,
            body_part: facts.body_part,
            target_position: facts.target_position.expect("validated target position"),
            target_yaw: facts.target_yaw.expect("validated target yaw"),
        },
    };
    Ok(ValidatedRangedShot {
        attacker: facts.attacker,
        attacker_side,
        attacker_position: facts.attacker_position,
        attacker_yaw: facts.attacker_yaw,
        reported_precision: facts.reported_precision,
        impact,
    })
}

pub(super) fn validate_melee_intent_cheap(
    facts: MeleeIntentFacts,
) -> Result<ValidatedMeleeAttack, MeleeIntentRejection> {
    if facts.attacker == facts.target {
        return Err(MeleeIntentRejection::SelfTarget);
    }
    let (Some(attacker_side), Some(target_side)) = (facts.attacker_side, facts.target_side) else {
        return Err(MeleeIntentRejection::MissingSide);
    };
    if attacker_side == target_side {
        return Err(MeleeIntentRejection::FriendlyTarget);
    }
    let (Some(attacker_incapacitated), Some(target_incapacitated)) =
        (facts.attacker_incapacitated, facts.target_incapacitated)
    else {
        return Err(MeleeIntentRejection::MissingCombatState);
    };
    if attacker_incapacitated || target_incapacitated {
        return Err(MeleeIntentRejection::Incapacitated);
    }
    if !facts.attack_capability.is_available() {
        return Err(MeleeIntentRejection::DisabledWeaponArm);
    }
    if !facts.arm_reach.is_finite()
        || facts.arm_reach <= 0.0
        || !facts.weapon_reach.is_finite()
        || facts.weapon_reach < 0.0
    {
        return Err(MeleeIntentRejection::Unarmed);
    }
    if !facts.separation.is_finite()
        || facts.separation
            > melee_interaction_range(facts.arm_reach, facts.weapon_reach)
                + facts.range_latency_tolerance
    {
        return Err(MeleeIntentRejection::OutOfRange);
    }
    if !facts.authority_permits {
        return Err(MeleeIntentRejection::Windup);
    }
    Ok(ValidatedMeleeAttack {
        attacker: facts.attacker,
        target: facts.target,
        body_part: facts.body_part,
        reported_precision: facts.reported_precision,
        attacker_position: facts.attacker_position,
        target_position: facts.target_position,
        attacker_yaw: facts.attacker_yaw,
        target_yaw: facts.target_yaw,
    })
}
