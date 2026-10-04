//! Social request parsing and living, co-located participant admission.
use super::*;

pub(super) fn admit_social_action(
    ctx: &ReducerContext,
    actor_id: u64,
    target_id: u64,
    action_key: &str,
) -> Result<SocialActionKind, String> {
    let action = action_key
        .parse::<SocialActionKind>()
        .map_err(|error| error.to_string())?;
    let is_self = actor_id == target_id;
    if is_self != (action == SocialActionKind::Reflect) {
        return Err("Reflect is self-only; other social actions require a companion".into());
    }
    let actor = ctx
        .db
        .character()
        .id()
        .find(actor_id)
        .ok_or("Actor not found")?;
    let target = ctx
        .db
        .character()
        .id()
        .find(target_id)
        .ok_or("Target not found")?;
    validate_social_pair(ctx, &actor, &target, is_self)?;
    Ok(action)
}
