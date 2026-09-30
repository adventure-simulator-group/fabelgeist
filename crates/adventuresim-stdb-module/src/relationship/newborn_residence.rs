//! Resolve optional newborn housing after the birth itself has committed.

use super::*;

pub(super) fn attach_newborn_residence(
    ctx: &ReducerContext,
    child_id: u64,
    parent_ids: [u64; 2],
    due_minute: StrategicMinute,
    settlement_id: &str,
) {
    let holding_id = parent_ids
        .into_iter()
        .filter_map(|parent_id| {
            crate::residence::occupant_holding_id_at(ctx, parent_id, due_minute)
        })
        .find(|holding_id| {
            ctx.db
                .residence_holding()
                .id()
                .find(holding_id.to_owned())
                .is_some_and(|holding| {
                    crate::residence::holding_active_at(ctx, &holding.id, due_minute)
                        && holding.settlement_id == settlement_id
                })
        });
    if let Some(holding_id) = holding_id {
        // Housing is ancillary to an uncomplicated birth. If household or
        // occupancy authority changed, the child remains without this link.
        let _ = crate::residence::move_residence_occupant_effective(
            ctx,
            &holding_id,
            child_id,
            due_minute,
        );
    }
}
