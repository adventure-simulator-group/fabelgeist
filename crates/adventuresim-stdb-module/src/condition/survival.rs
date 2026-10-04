//! Project survival impairment after morale, pain, blood and fatigue evaluation.

use super::*;
use adventuresim_core::identity::CharacterId;

pub(super) struct SurvivalProjection {
    pub(super) needs: CharacterNeeds,
    pub(super) exposure: CharacterExposure,
    pub(super) water_capacity: u32,
    pub(super) hunger: f32,
    pub(super) thirst: f32,
    pub(super) thermal: f32,
}

impl SurvivalProjection {
    pub(super) fn load(
        ctx: &ReducerContext,
        character_id: CharacterId,
    ) -> Result<Self, StrategicConditionError> {
        let needs = ctx
            .db
            .character_needs()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Needs,
            })?;
        let water_capacity = water_capacity_ml(ctx, character_id);
        let hunger = hunger_incapacitation(needs.food_balance_kcal, STRATEGIC_TRAVEL_KCAL_PER_DAY);
        let thirst =
            thirst_incapacitation(needs.water_balance_ml, STRATEGIC_TRAVEL_WATER_ML_PER_DAY);
        let exposure = ctx
            .db
            .character_exposure()
            .character_id()
            .find(u64::from(character_id))
            .ok_or(StrategicConditionError::Missing {
                character: character_id,
                component: ConditionComponent::Exposure,
            })?;
        let thermal = adventuresim_core::survival::thermal_incapacitation(exposure.thermal_strain);
        Ok(Self {
            needs,
            exposure,
            water_capacity,
            hunger,
            thirst,
            thermal,
        })
    }
}
