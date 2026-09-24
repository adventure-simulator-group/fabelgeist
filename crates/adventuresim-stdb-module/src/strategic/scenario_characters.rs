use super::*;

pub(super) fn ensure_scenario_character_at(
    ctx: &ReducerContext,
    character_id: u64,
    name: &str,
    settlement_id: &str,
) -> Result<(), String> {
    if let Some(character) = ctx.db.character().id().find(character_id) {
        return (character.current_settlement_id.as_deref() == Some(settlement_id))
            .then_some(())
            .ok_or_else(|| "Development scenario character is in the wrong settlement".into());
    }
    crate::character::insert_character_with_origin(
        ctx,
        name.into(),
        character_id,
        crate::character::CharacterCreationOptions {
            origin_settlement_id: Some(settlement_id),
            mode: crate::character::CharacterCreationMode::Player,
            create_solo_party: true,
            materialize_generated_carry: true,
            stable_seed: character_id,
            initial_time_minute: None,
            field_actor: false,
            npc_personality: None,
        },
        None,
        None,
    )
}
