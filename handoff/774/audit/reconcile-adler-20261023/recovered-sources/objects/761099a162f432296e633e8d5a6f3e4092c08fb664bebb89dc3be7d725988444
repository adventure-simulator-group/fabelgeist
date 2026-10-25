//! Strategic condition refresh and allied combatant admission for pure resolution.

use adventuresim_core::autoresolve::Combatant;
use spacetimedb::ReducerContext;

pub(super) fn load_allied_combatants(
    ctx: &ReducerContext,
    member_ids: &[u64],
) -> crate::StrategicOperationResult<Vec<Combatant>> {
    member_ids
        .iter()
        .map(|member_id| {
            let condition =
                crate::condition::refresh_character_strategic_condition(ctx, *member_id)?;
            crate::capability::load_combatant(
                ctx,
                *member_id,
                condition.incapacitation,
                condition.pain,
                condition.blood_loss,
            )
        })
        .collect()
}
