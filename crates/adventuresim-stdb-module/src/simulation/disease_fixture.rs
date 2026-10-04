//! Owned disposable illness seeding through ordinary lifecycle hooks.
use super::*;

/// Deterministic illness fixture for the claimed disposable simulator. It
/// supplies only an infection seed; diagnosis and treatment remain ordinary
/// player-facing actions and the simulator never subscribes to this private row.
#[reducer]
pub fn seed_simulation_disease(
    ctx: &ReducerContext,
    nonce: String,
    character_id: u64,
    disease_id: String,
) -> Result<(), String> {
    let run = owned_run(ctx, &nonce)?;
    let sim = ctx
        .db
        .simulation_character()
        .character_id()
        .find(character_id)
        .ok_or("Only configured simulation characters may use this fixture")?;
    if sim.run_id != run.id {
        return Err("Simulation character belongs to another run".into());
    }
    crate::require_living_character(ctx, character_id)?;
    if ctx
        .db
        .infection_episode()
        .character_id()
        .filter(character_id)
        .next()
        .is_some()
    {
        return Err("Simulation disease fixture may only be seeded once".into());
    }
    let disease = disease_id
        .parse::<adventuresim_core::disease::DiseaseId>()
        .map_err(|_| "Unknown simulation fixture disease".to_owned())?;
    ctx.db
        .infection_episode()
        .insert(crate::disease::InfectionEpisodeRow {
            id: 0,
            character_id,
            disease_id,
            contracted_at: StrategicMinute::ZERO,
            ruleset_version: adventuresim_core::physiology::PHYSIOLOGY_RULESET_VERSION,
            phenotype_key_version: adventuresim_core::physiology::PHENOTYPE_KEY_VERSION,
        });
    let requested = adventuresim_core::disease::definition(disease)
        .incubation_minutes
        .saturating_add(60);
    // Advance through the same disease interval hooks as ordinary gameplay so
    // the simulator observes symptom onset instead of receiving hidden fixture
    // knowledge from the private infection row.
    let injury_limit = crate::surgery::preview_injury_boundary(
        ctx,
        character_id,
        requested,
        crate::surgery::InjuryRecoveryMinutes::NONE,
    )?
    .elapsed;
    let (elapsed, terminal) =
        crate::disease::clip_elapsed_for_disease(ctx, character_id, injury_limit, false)?;
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or("Simulation character time not found")?;
    let settled = crate::surgery::settle_injuries(
        ctx,
        character_id,
        elapsed,
        crate::surgery::InjuryRecoveryMinutes::NONE,
    )?;
    let interval_end = time.minutes.saturating_add_minutes(settled.elapsed);
    time.minutes = interval_end;
    ctx.db.character_time().character_id().update(time);
    crate::disease::finish_disease_interval(ctx, character_id, terminal)?;
    settle_lifecycle_after_character_time_write(ctx, character_id, interval_end)?;
    if terminal.is_some() || !settled.alive {
        return Ok(());
    }
    crate::condition::refresh_character_strategic_condition(ctx, character_id)?;
    crate::capability::refresh_character_capability(ctx, character_id)?;
    Ok(())
}
