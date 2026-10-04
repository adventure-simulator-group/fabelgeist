/// First segment of an interval cut by a relationship lifecycle event.
fn first_lifecycle_segment(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    start: StrategicMinute,
    requested_minutes: u64,
) -> Option<u64> {
    let end = start.saturating_add_minutes(requested_minutes);
    crate::relationship::next_lifecycle_boundary(ctx, (character_id).into(), start, end)
        .map(|boundary| boundary.elapsed_since(start))
}
