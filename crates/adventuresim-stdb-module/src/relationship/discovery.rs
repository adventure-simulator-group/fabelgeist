//! Secret-courtship exposure and observer discovery chronology.

use super::*;
use crate::time::CharacterClockError;
use adventuresim_core::courtship::{CourtshipDiscoveryTrial, DiscoveryDayOutcome};
use adventuresim_core::identity::CharacterId;
/// Resolve the public-risk side of an informal relationship once per observer
/// and day.  The receipt makes it independent of time-advance chunking; only
/// living adult parents and siblings co-located with either partner observe.
fn settle_secret_courtship_discovery_for_pair(
    ctx: &ReducerContext,
    first_id: CharacterId,
    second_id: CharacterId,
    day: StrategicDayIndex,
) -> Result<DiscoveryDayOutcome, CharacterClockError> {
    let (first, second) = (first_id.min(second_id), first_id.max(second_id));
    let courtship_id = format!("courtship:{first}:{second}");
    let Some(courtship) = ctx.db.courtship().id().find(&courtship_id) else {
        return Ok(DiscoveryDayOutcome::Evaluated);
    };
    if courtship.kind != CourtshipKind::Informal || courtship.status != CourtshipStatus::Active {
        return Ok(DiscoveryDayOutcome::Evaluated);
    }
    let first_frontier = canonical_now(ctx, first)?;
    let second_frontier = canonical_now(ctx, second)?;
    if first_frontier.day_index() < day || second_frontier.day_index() < day {
        return Ok(DiscoveryDayOutcome::DeferredForFrontier);
    }
    let mut observers: Vec<_> = ctx
        .db
        .courtship_observer_baseline()
        .courtship_id()
        .filter(&courtship_id)
        .collect();
    observers.sort_by_key(|baseline| baseline.observer_id);
    let attempted_minute = day.start().max(courtship.started_minute);
    for baseline in &observers {
        // Death is an effective-dated end to observer eligibility. A dead
        // observer neither rolls nor prevents the remaining living cohort
        // from resolving this and later relationship days.
        if !character_alive_at(ctx, baseline.observer_id.into(), attempted_minute) {
            continue;
        }
        if canonical_now(ctx, (baseline.observer_id).into())?.day_index() < day {
            return Ok(DiscoveryDayOutcome::DeferredForFrontier);
        }
    }
    for baseline in observers {
        let observer_id = CharacterId::from(baseline.observer_id);
        if !character_alive_at(ctx, observer_id, attempted_minute) {
            continue;
        }
        let id = format!("discovery:{courtship_id}:{observer_id}:{day}");
        if ctx.db.courtship_discovery().id().find(&id).is_some() {
            continue;
        }
        let insight = baseline.observer_insight;
        let deception = courtship.weaker_deception_baseline;
        let entropy = CourtshipDiscoveryTrial::new(first, second, observer_id, day)
            .rng()
            .unit_f32();
        let discovery_chance = ((insight - deception) * 0.08 + 0.15).clamp(0.02, 0.85);
        let succeeded = entropy < discovery_chance;
        ctx.db.courtship_discovery().insert(CourtshipDiscovery {
            id,
            courtship_id: courtship_id.clone(),
            observer_id: u64::from(observer_id),
            day: u64::from(day),
            attempted_minute,
            succeeded,
            observer_insight: insight,
            weaker_deception: deception,
        });
        if succeeded {
            if let Some(mut active) = ctx.db.courtship().id().find(&courtship_id) {
                active.status = CourtshipStatus::Exposed;
                ctx.db.courtship().id().update(active);
            }
            // The immutable discovery receipt remains effective on the
            // relationship day. Affinity, however, is a mutable soft edge
            // evaluated by `current_affinity` at the observer's current
            // frontier. Anchor the penalty at that same frontier so a delayed
            // settlement cannot decay the value once before subtraction and
            // then a second time from a backdated anchor.
            let anchor_minute = canonical_now(ctx, observer_id).unwrap_or(attempted_minute);
            for participant_id in [first, second] {
                let affinity_id = adventuresim_core::courtship::CharacterAffinityKey::new(
                    observer_id,
                    participant_id,
                )
                .to_string();
                let row = CharacterAffinity {
                    id: affinity_id.clone(),
                    subject_id: u64::from(observer_id),
                    actor_id: u64::from(participant_id),
                    anchor: (crate::social::current_affinity(ctx, observer_id, participant_id)
                        - 8.0)
                        .clamp(-100.0, 100.0),
                    anchor_minute,
                };
                if ctx
                    .db
                    .character_affinity()
                    .id()
                    .find(&affinity_id)
                    .is_some()
                {
                    ctx.db.character_affinity().id().update(row);
                } else {
                    ctx.db.character_affinity().insert(row);
                }
            }
            break;
        }
    }
    Ok(DiscoveryDayOutcome::Evaluated)
}

/// Advance all active secret relationships involving this character through
/// the current relationship day. This is independent of whom the character
/// happened to socialize with: every eligible family observer gets one
/// receipt per day until the first successful exposure.
pub(crate) fn settle_secret_courtship_discovery_for_character(
    ctx: &ReducerContext,
    character_id: CharacterId,
    minute: StrategicMinute,
) -> Result<(), CharacterClockError> {
    let current_day = minute.day_index();
    let mut courtship_ids: Vec<_> = ctx
        .db
        .courtship()
        .iter()
        .filter(|row| {
            row.kind == CourtshipKind::Informal
                && row.status == CourtshipStatus::Active
                && (CharacterId::from(row.first_character_id) == character_id
                    || CharacterId::from(row.second_character_id) == character_id
                    || ctx
                        .db
                        .courtship_observer_baseline()
                        .observer_id()
                        .filter(u64::from(character_id))
                        .any(|baseline| baseline.courtship_id == row.id))
                && row.started_minute <= minute
        })
        .map(|row| row.id)
        .collect();
    courtship_ids.sort();
    for courtship_id in courtship_ids {
        while let Some(courtship) = ctx.db.courtship().id().find(&courtship_id) {
            if courtship.status != CourtshipStatus::Active
                || StrategicDayIndex::new(courtship.next_discovery_day) > current_day
            {
                break;
            }
            let day = StrategicDayIndex::new(courtship.next_discovery_day);
            let evaluated = settle_secret_courtship_discovery_for_pair(
                ctx,
                courtship.first_character_id.into(),
                courtship.second_character_id.into(),
                day,
            )?;
            if evaluated == DiscoveryDayOutcome::DeferredForFrontier {
                break;
            }
            let Some(mut updated) = ctx.db.courtship().id().find(&courtship_id) else {
                break;
            };
            if updated.status != CourtshipStatus::Active {
                break;
            }
            updated.next_discovery_day = u64::from(day.following());
            ctx.db.courtship().id().update(updated);
        }
    }
    Ok(())
}
