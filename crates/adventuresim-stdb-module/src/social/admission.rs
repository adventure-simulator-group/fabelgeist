//! Social request parsing and living, co-located participant admission.
use super::*;

pub(super) fn admit_social_action(
    ctx: &ReducerContext,
    actor_id: adventuresim_core::identity::CharacterId,
    target_id: adventuresim_core::identity::CharacterId,
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
        .find(u64::from(actor_id))
        .ok_or("Actor not found")?;
    let target = ctx
        .db
        .character()
        .id()
        .find(u64::from(target_id))
        .ok_or("Target not found")?;
    validate_social_pair(ctx, &actor, &target, is_self)?;
    Ok(action)
}
