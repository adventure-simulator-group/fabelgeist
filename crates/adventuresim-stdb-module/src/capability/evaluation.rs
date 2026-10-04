//! Derive capabilities and combat projection from admitted strategic state.

use super::*;

pub(crate) fn evaluate_character(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<CharacterCapabilities, CapabilityEvaluationError> {
    let CapabilityInputs {
        attributes,
        skills,
        body,
        essentials,
    } = CapabilityInputs::load(ctx, character_id)?;
    let equipment = StrategicEquipment::load(ctx, character_id);
    Ok(evaluate_capabilities(
        &attributes,
        &body,
        &essentials,
        &equipment,
        &skills,
    ))
}

pub(crate) fn load_combatant(
    ctx: &ReducerContext,
    character_id: CharacterId,
    strategic_incapacitation: f32,
    strategic_pain: f32,
    strategic_blood_loss: f32,
) -> Result<Combatant, CapabilityEvaluationError> {
    let CapabilityInputs {
        attributes,
        skills,
        body: limbs,
        essentials: stats,
    } = CapabilityInputs::load(ctx, character_id)?;
    let condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(CapabilityEvaluationError::Missing {
            character: character_id,
            component: CapabilityComponent::Condition,
        })?;
    let fatigue = fatigue_incapacitation(stats.fatigue_by_parts(&attributes, &limbs));
    let equipment = StrategicEquipment::load(ctx, character_id);
    let combat_equipment = equipment.combat_equipment();
    let (starting_incapacitation, starting_blood_fraction) = derive_combat_starting_condition(
        strategic_incapacitation,
        strategic_pain,
        strategic_blood_loss,
        condition.current_blood_ml,
        condition.maximum_blood_ml,
    );

    Ok(Combatant::from_strategic_state(CombatantStrategicState {
        fatigue,
        id: u64::from(character_id),
        attributes: attributes.into(),
        body: CombatBody {
            health: [
                limbs.left_arm_health,
                limbs.right_arm_health,
                limbs.left_leg_health,
                limbs.right_leg_health,
                limbs.chest_health,
                limbs.stomach_health,
                limbs.head_health,
            ],
            weight_kg: condition.body_weight_kg,
            primary_side: BodySide::Right,
        },
        essentials: CombatEssentials {
            calories_used_today: stats.calories_used,
            focus_level: stats.focus,
        },
        equipment: combat_equipment,
        skills: CombatSkills {
            polearm_hours: skills.polearm_hours,
            axe_hours: skills.axe_hours,
            bludgeon_hours: skills.bludgeon_hours,
            sword_hours: skills.sword_hours,
            knife_hours: skills.knife_hours,
            dodge_hours: skills.dodge_hours,
            block_hours: skills.block_hours,
            bow_hours: skills.bow_hours,
            crossbow_hours: skills.crossbow_hours,
            firearm_hours: skills.firearm_hours,
            throw_hours: skills.throw_hours,
            will_hours: skills.will_hours,
            insight_hours: skills.insight_hours,
            charm_hours: skills.charm_hours,
            command_hours: skills.command_hours,
            deception_hours: skills.deception_hours,
            physiology_hours: skills.physiology_hours,
            religion_hours: skills.religion_hours.total_direct(),
            stealth_hours: skills.stealth_hours,
            balance_hours: skills.balance_hours,
            bestiary_hours: skills.bestiary_hours,
            surgery_hours: skills.surgery_hours,
            tailoring_hours: skills.tailoring_hours,
            smithing_hours: skills.smithing_hours,
        },
        starting_incapacitation,
        starting_blood_fraction,
    }))
}
