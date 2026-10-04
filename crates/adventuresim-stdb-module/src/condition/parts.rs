//! Ordered body, skill and stat admission for strategic condition checks.

use super::*;

pub(crate) fn mental_check(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    skill: Skill,
) -> Result<f32, StrategicConditionError> {
    let attributes = ctx
        .db
        .character_attributes()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Attributes,
        })?;
    let limbs = ctx
        .db
        .character_limbs()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Limbs,
        })?;
    let stats = ctx
        .db
        .character_stats()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Stats,
        })?;
    let skills = ctx
        .db
        .character_skills()
        .character_id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Skills,
        })?;
    let equipment = StrategicEquipment::load(ctx, character_id);
    Ok(skills.skill_check_by_parts(
        skill,
        &attributes,
        &limbs,
        &stats,
        &equipment,
        LimbWeights::all_equal(),
    ))
}

pub(super) fn load_character_parts(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<
    (
        CharacterAttributes,
        CharacterLimbs,
        CharacterStats,
        CharacterSkills,
    ),
    StrategicConditionError,
> {
    Ok((
        ctx.db
            .character_attributes()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Attributes,
            })?,
        ctx.db
            .character_limbs()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Limbs,
            })?,
        ctx.db
            .character_stats()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Stats,
            })?,
        ctx.db
            .character_skills()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Skills,
            })?,
    ))
}
