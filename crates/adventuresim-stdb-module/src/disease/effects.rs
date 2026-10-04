//! Apply admitted episode impairments to strategic attributes.

use super::*;

pub(crate) fn effective_attributes(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    mut attributes: crate::CharacterAttributes,
) -> Result<crate::CharacterAttributes, EpisodeDecodeError> {
    let now = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map_or(StrategicMinute::ZERO, |t| t.minutes);
    let (penalty, _, _, _) = disease::combined_state(
        &character_episodes(ctx, character_id)?,
        now,
        attributes.immunity,
    );
    attributes.endurance = (attributes.endurance - penalty.endurance).max(0.0);
    attributes.immunity = (attributes.immunity - penalty.immunity).max(0.0);
    attributes.gut = (attributes.gut - penalty.gut).max(0.0);
    attributes.intelligence = (attributes.intelligence - penalty.intelligence).max(0.0);
    attributes.instinct = (attributes.instinct - penalty.instinct).max(0.0);
    for value in [
        &mut attributes.left_arm_agility,
        &mut attributes.right_arm_agility,
        &mut attributes.left_leg_agility,
        &mut attributes.right_leg_agility,
    ] {
        *value = (*value - penalty.limb_agility).max(0.0)
    }
    Ok(attributes)
}
