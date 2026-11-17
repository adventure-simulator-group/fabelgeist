//! Validate combat targets before authorizing any attack work or damage.

use super::*;

pub(in crate::combat) fn validate_ranged_intent(
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
        if !attacker_side.is_hostile_to(target_side) {
            return Err(RangedIntentRejection::NonHostileTarget);
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
        Some(target) => {
            let (Some(target_position), Some(target_yaw)) =
                (facts.target_position, facts.target_yaw)
            else {
                return Err(RangedIntentRejection::MissingTargetTransform);
            };
            ValidatedRangedImpact::Hit {
                target,
                body_part: facts.body_part,
                target_position,
                target_yaw,
            }
        }
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

pub(in crate::combat) fn validate_melee_intent_cheap(
    facts: MeleeIntentFacts,
) -> Result<ValidatedMeleeAttack, MeleeIntentRejection> {
    if facts.attacker == facts.target {
        return Err(MeleeIntentRejection::SelfTarget);
    }
    let (Some(attacker_side), Some(target_side)) = (facts.attacker_side, facts.target_side) else {
        return Err(MeleeIntentRejection::MissingSide);
    };
    if !attacker_side.is_hostile_to(target_side) {
        return Err(MeleeIntentRejection::NonHostileTarget);
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

pub(in crate::combat) fn ranged_target_in_aim_cone(
    yaw: f32,
    attacker: Vec3,
    target: Vec3,
    half_angle_degrees: f32,
) -> bool {
    let offset = target.xz() - attacker.xz();
    let Some(direction) = offset.try_normalize() else {
        return false;
    };
    let forward = Vec2::new(-yaw.sin(), -yaw.cos());
    direction.dot(forward) >= half_angle_degrees.to_radians().cos()
}
