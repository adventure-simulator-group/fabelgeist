//! Refresh persisted capability bands and their social-presence boundaries.

use super::*;

pub(crate) fn refresh_character_capability(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<CharacterCapabilities, CapabilityEvaluationError> {
    let capabilities = evaluate_character(ctx, character_id)?;
    let mut row = CharacterCapability::from((u64::from(character_id), capabilities));
    let condition = ctx
        .db
        .character_strategic_condition()
        .character_id()
        .find(u64::from(character_id));
    let combatant = load_combatant(
        ctx,
        character_id,
        condition.as_ref().map_or(0.0, |row| row.incapacitation),
        condition.as_ref().map_or(0.0, |row| row.pain),
        condition.as_ref().map_or(0.0, |row| row.blood_loss),
    )?;
    row.autoresolve_combat_power =
        adventuresim_core::autoresolve::autoresolve_combat_power(&combatant);
    if let Some(existing) = ctx
        .db
        .character_capability()
        .character_id()
        .find(u64::from(character_id))
    {
        // Capability reads are currently refreshed lazily by the web layer. Avoid
        // emitting a table update when the derived value has not changed: that
        // update invalidates the SSE UI, which otherwise refreshes the same
        // capability again and creates a feedback loop.
        if existing != row {
            let old_band = existing.physiology.round().clamp(0.0, 5.0) as u8;
            let new_band = row.physiology.round().clamp(0.0, 5.0) as u8;
            if old_band != new_band {
                crate::social::close_physiology_presence(ctx, character_id);
            }
            ctx.db.character_capability().character_id().update(row);
            if old_band != new_band {
                crate::social::reset_familiarity_after_join(ctx, (u64::from(character_id)).into());
            }
        }
    } else {
        ctx.db.character_capability().insert(row);
    }
    Ok(capabilities)
}
