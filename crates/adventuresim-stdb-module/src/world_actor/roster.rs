/// Context roster closure at an authoritative strategic instant.
pub(crate) fn deactivate_context_roster_at(
    ctx: &ReducerContext,
    context_id: &str,
    minute: StrategicMinute,
) {
    for row in context_members(ctx, context_id) {
        close_context_membership_at(ctx, row, minute);
    }
}

/// Close an interval once; later recovery or cleanup cannot rewrite its end.
pub(crate) fn close_context_membership_at(
    ctx: &ReducerContext,
    mut row: CharacterContextMembership,
    minute: StrategicMinute,
) {
    if row.is_open() {
        row.left_at = Some(minute.max(row.entered_at));
        row.revision = row.revision.saturating_add(1);
        ctx.db.character_context_membership().id().update(row);
    }
}

pub(crate) fn deactivate_context_roster(ctx: &ReducerContext, context_id: &str) {
    let minute = crate::time::refresh_clock(ctx).unwrap_or_default();
    deactivate_context_roster_at(ctx, context_id, minute);
}
