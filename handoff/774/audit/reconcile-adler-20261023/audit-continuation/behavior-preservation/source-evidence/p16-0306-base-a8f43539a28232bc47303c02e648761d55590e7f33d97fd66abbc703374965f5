/// First segment of an interval cut by a relationship lifecycle event.
fn first_lifecycle_segment(
    ctx: &ReducerContext,
    character_id: u64,
    start: StrategicMinute,
    requested_minutes: u64,
) -> Option<u64> {
    let end = start.saturating_add_minutes(requested_minutes);
    crate::relationship::next_lifecycle_boundary(ctx, character_id, start, end)
        .map(|boundary| boundary.elapsed_since(start))
}
