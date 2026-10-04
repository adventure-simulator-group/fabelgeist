//! Effective-dated father admission for canonical courtship.

use super::*;
use adventuresim_core::identity::CharacterId;
mod admission;
pub(super) use admission::FatherAdmissionError;
use admission::require_frontier;

pub(super) fn father_of_at(
    ctx: &ReducerContext,
    child_id: CharacterId,
    minute: StrategicMinute,
) -> Result<Option<CharacterId>, FatherAdmissionError> {
    let father = ctx.db.character_kinship().iter().find_map(|edge| {
        (CharacterId::from(edge.subject_id) == child_id
            && edge.kind == KinshipKind::Parent
            && edge.established_minute <= minute)
            .then(|| {
                ctx.db
                    .character_personality()
                    .character_id()
                    .find(edge.related_id)
                    .filter(|personality| personality.sex == Sex::Male)
                    .map(|_| CharacterId::from(edge.related_id))
            })
            .flatten()
    });
    let Some(father) = father else {
        return Ok(None);
    };
    require_frontier(child_id, father, minute, canonical_now(ctx, father))?;
    Ok(character_alive_at(ctx, father, minute).then_some(father))
}
