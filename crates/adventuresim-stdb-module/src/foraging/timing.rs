//! Character-clock access and terminal boundaries for a foraging attempt.

use super::*;

pub(super) fn character_minute(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<StrategicMinute, String> {
    ctx.db
        .character_time()
        .character_id()
        .find(character_id)
        .map(|time| time.minutes)
        .ok_or("Character time record not found".into())
}

pub(super) fn forage_terminal_minute(
    ctx: &ReducerContext,
    character_id: u64,
    current_minute: StrategicMinute,
    duration: u64,
) -> Result<Option<StrategicMinute>, String> {
    let injury = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        duration,
        crate::surgery::InjuryRecoveryMinutes::NONE,
    )?;
    let (disease_safe, disease_terminal) = crate::disease::preview_disease_terminal_boundary(
        ctx,
        character_id,
        injury.elapsed,
        false,
    )?;
    let safe = injury.elapsed.min(disease_safe);
    if safe < duration || injury.terminal || disease_terminal {
        Ok(Some(current_minute.checked_add_minutes(safe).ok_or(
            "Foraging terminal time exceeds the strategic clock",
        )?))
    } else {
        Ok(None)
    }
}
