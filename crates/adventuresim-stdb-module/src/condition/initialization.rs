//! Infallible creation of missing strategic condition, needs, and exposure rows.

use super::*;

pub(crate) fn initialize_character_condition(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) {
    if ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .is_none()
    {
        let body_mass = BodyMassKg::DEFAULT;
        let maximum_blood_ml = body_mass.estimated_blood_milliliters();
        ctx.db.character_condition().insert(CharacterCondition {
            character_id: u64::from(character_id),
            body_weight_kg: body_mass.kilograms(),
            current_blood_ml: maximum_blood_ml,
            maximum_blood_ml,
            religion_id: None,
        });
    }
    if ctx
        .db
        .character_needs()
        .character_id()
        .find(u64::from(character_id))
        .is_none()
    {
        ctx.db.character_needs().insert(CharacterNeeds {
            character_id: u64::from(character_id),
            food_balance_kcal: STRATEGIC_TRAVEL_KCAL_PER_DAY,
            water_balance_ml: STRATEGIC_TRAVEL_WATER_ML_PER_DAY,
        });
    }
    if ctx
        .db
        .character_exposure()
        .character_id()
        .find(u64::from(character_id))
        .is_none()
    {
        ctx.db.character_exposure().insert(CharacterExposure {
            character_id: u64::from(character_id),
            wetness_bps: 0,
            thermal_strain: 0,
            frostbite_progress_minutes: 0,
        });
    }
}
