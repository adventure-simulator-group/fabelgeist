/// Context roster closure at an authoritative strategic instant.
pub(crate) fn deactivate_context_roster_at(
    ctx: &ReducerContext,
    context_id: &str,
    minute: StrategicMinute,
) {
    for mut row in context_members(ctx, context_id) {
        if !row.active {
            continue;
        }
        row.active = false;
        row.left_at = Some(minute.max(row.entered_at));
        row.revision = row.revision.saturating_add(1);
        ctx.db.character_context_membership().id().update(row);
    }
}

pub(crate) fn deactivate_context_roster(ctx: &ReducerContext, context_id: &str) {
    let minute = crate::time::refresh_clock(ctx).unwrap_or_default();
    deactivate_context_roster_at(ctx, context_id, minute);
}
