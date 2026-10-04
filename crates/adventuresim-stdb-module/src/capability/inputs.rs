//! Ordered strategic state admission shared by capability and combat projection.

use super::*;

pub(super) struct CapabilityInputs {
    pub(super) attributes: CharacterAttributes,
    pub(super) skills: CharacterSkills,
    pub(super) body: CharacterLimbs,
    pub(super) essentials: CharacterStats,
}

impl CapabilityInputs {
    pub(super) fn load(
        ctx: &ReducerContext,
        character: CharacterId,
    ) -> Result<Self, CapabilityEvaluationError> {
        let attributes = ctx
            .db
            .character_attributes()
            .character_id()
            .find(u64::from(character))
            .ok_or(CapabilityEvaluationError::Missing {
                character,
                component: CapabilityComponent::Attributes,
            })?;
        let attributes = crate::disease::effective_attributes(ctx, character, attributes)?;
        let skills = ctx
            .db
            .character_skills()
            .character_id()
            .find(u64::from(character))
            .ok_or(CapabilityEvaluationError::Missing {
                character,
                component: CapabilityComponent::Skills,
            })?;
        let body = ctx
            .db
            .character_limbs()
            .character_id()
            .find(u64::from(character))
            .ok_or(CapabilityEvaluationError::Missing {
                character,
                component: CapabilityComponent::Limbs,
            })?;
        let essentials = ctx
            .db
            .character_stats()
            .character_id()
            .find(u64::from(character))
            .ok_or(CapabilityEvaluationError::Missing {
                character,
                component: CapabilityComponent::Stats,
            })?;
        Ok(Self {
            attributes,
            skills,
            body,
            essentials,
        })
    }
}
