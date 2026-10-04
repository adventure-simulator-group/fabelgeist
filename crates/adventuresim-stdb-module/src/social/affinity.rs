//! Directed affinity reads and anchored writes at the authoritative personal clock.

use super::*;

pub(crate) fn current_affinity(
    ctx: &ReducerContext,
    subject_id: adventuresim_core::identity::CharacterId,
    actor_id: adventuresim_core::identity::CharacterId,
) -> f32 {
    let now = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(subject_id))
        .map_or(StrategicMinute::ZERO, |v| v.minutes);
    ctx.db
        .character_affinity()
        .id()
        .find(
            adventuresim_core::courtship::CharacterAffinityKey::new(subject_id, actor_id)
                .to_string(),
        )
        .map_or(0.0, |row| {
            settle_affinity(row.anchor, now.elapsed_since(row.anchor_minute))
        })
}

pub(crate) fn put_affinity(
    ctx: &ReducerContext,
    subject_id: adventuresim_core::identity::CharacterId,
    actor_id: adventuresim_core::identity::CharacterId,
    value: f32,
) {
    let anchor_minute = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(subject_id))
        .map_or(StrategicMinute::ZERO, |v| v.minutes);
    put_affinity_at(ctx, subject_id, actor_id, value, anchor_minute);
}

pub(crate) fn put_affinity_at(
    ctx: &ReducerContext,
    subject_id: adventuresim_core::identity::CharacterId,
    actor_id: adventuresim_core::identity::CharacterId,
    value: f32,
    anchor_minute: StrategicMinute,
) {
    let id =
        adventuresim_core::courtship::CharacterAffinityKey::new(subject_id, actor_id).to_string();
    let row = CharacterAffinity {
        id: id.clone(),
        subject_id: u64::from(subject_id),
        actor_id: u64::from(actor_id),
        anchor: value.clamp(AFFINITY_MIN, AFFINITY_MAX),
        anchor_minute,
    };
    if ctx.db.character_affinity().id().find(&id).is_some() {
        ctx.db.character_affinity().id().update(row);
    } else {
        ctx.db.character_affinity().insert(row);
    }
}

/// Project a directional affinity at an effective relationship minute.
///
/// A row whose anchor is newer than the requested minute cannot be
/// reconstructed from compact soft state, so callers fail closed instead of
/// letting a future opinion authorize a backdated exclusive relationship.
pub(crate) fn affinity_at(
    ctx: &ReducerContext,
    subject_id: adventuresim_core::identity::CharacterId,
    actor_id: adventuresim_core::identity::CharacterId,
    minute: StrategicMinute,
) -> Option<f32> {
    let Some(row) = ctx.db.character_affinity().id().find(
        adventuresim_core::courtship::CharacterAffinityKey::new(subject_id, actor_id).to_string(),
    ) else {
        return Some(0.0);
    };
    (row.anchor_minute <= minute).then(|| {
        adventuresim_core::social::settle_affinity(
            row.anchor,
            minute.elapsed_since(row.anchor_minute),
        )
    })
}
