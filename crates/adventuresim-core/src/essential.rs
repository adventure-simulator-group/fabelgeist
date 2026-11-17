use crate::{
    attribute::PlayerAttributes, body::PlayerBody, equipment::PlayerEquipment,
    prelude::SimpleAttribute,
};

pub use fatigue_handoff::{
    FatigueHandoffError, FatigueIncapacitation, fatigue_calories_after_handoff,
};

mod fatigue_handoff;

const CALORIES_PER_ENDURANCE: f32 = 1000.0;
const FATIGUE_EXPONENT: i32 = 5;

#[blanket::blanket(derive(Ref, Rc, Arc, Mut, Box, Cow))]
#[ambassador::delegatable_trait]
pub trait PlayerEssentials {
    fn calories_used_today(&self) -> f32;
    fn focus_level(&self) -> f32;

    fn fatigue_by_parts(&self, attr: &impl PlayerAttributes, body: &impl PlayerBody) -> f32 {
        self.calories_used_today() / fatigue_capacity_kcal(attr, body)
    }

    /// Strategic skills account for daily fatigue and burden here. Combat instead
    /// applies their contribution through live incapacitation at resolution.
    fn physical_skill_condition_by_parts(
        &self,
        attr: &impl PlayerAttributes,
        body: &impl PlayerBody,
        equipment: &impl PlayerEquipment,
    ) -> f32 {
        self.fatigue_penalty_by_parts(attr, body)
            * equipment.encumbrance_penalty_by_parts(attr, body)
    }

    fn fatigue_penalty_by_parts(
        &self,
        attr: &impl PlayerAttributes,
        body: &impl PlayerBody,
    ) -> f32 {
        let fatigue = self.fatigue_by_parts(attr, body);
        (1.0 - fatigue.powi(FATIGUE_EXPONENT)).max(0.0)
    }
}

/// Native calorie coordinate used by durable fatigue accounting. Live combat
/// exposes its incapacitation fraction instead; neither is an extra penalty.
pub fn fatigue_capacity_kcal(attr: &impl PlayerAttributes, body: &impl PlayerBody) -> f32 {
    attr.attr_by_parts(SimpleAttribute::Endurance, body) * CALORIES_PER_ENDURANCE
}
