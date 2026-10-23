
// Owns canonical personal-time policy and deterministic NPC clock advancement.
pub(crate) fn character_alive_at(
    ctx: &ReducerContext,
    character_id: u64,
    minute: StrategicMinute,
) -> bool {
    ctx.db.character().id().find(character_id).is_some()
        && ctx
            .db
            .character_birth()
            .character_id()
            .find(character_id)
            .is_some_and(|birth| minute.is_at_or_after_signed_birth(birth.birth_minute))
        && ctx
            .db
            .character_death()
            .character_id()
            .find(character_id)
            .is_none_or(|death| death.strategic_minute > minute)
}

pub fn canonical_now(ctx: &ReducerContext, character_id: u64) -> Result<StrategicMinute, String> {
    ctx.db
        .character_time()
        .character_id()
        .find(character_id)
        .map(|time| time.minutes)
        .ok_or_else(|| "Character time record not found".to_string())
}

/// Earliest relationship boundary which can change the meaning of an actor's
/// interval. Global materialization may already have written a future fact;
/// callers must still split the actor's personal interval at its effective
/// minute before applying leisure, household, or spouse consequences.
pub(crate) fn next_lifecycle_boundary(
    ctx: &ReducerContext,
    character_id: u64,
    start_minute: StrategicMinute,
    end_minute: StrategicMinute,
) -> Option<StrategicMinute> {
    let birthday = ctx
        .db
        .character_birth()
        .character_id()
        .find(character_id)
        .and_then(|birth| {
            start_minute.next_signed_birth_anniversary_before(birth.birth_minute, end_minute)
        });
    let wedding = ctx
        .db
        .exclusive_commitment()
        .iter()
        .filter(|row| {
            start_minute < row.effective_minute
                && row.effective_minute < end_minute
                && row.status == CommitmentStatus::Reserved
                && row.kind == CommitmentKind::Engagement
                && (row.first_character_id == character_id
                    || row.second_character_id == character_id)
        })
        .map(|row| row.effective_minute)
        .min();
    let birth = ctx
        .db
        .pregnancy()
        .iter()
        .filter(|row| {
            start_minute < row.due_minute
                && row.due_minute < end_minute
                && row.status == PregnancyStatus::Active
                && (row.mother_id == character_id || row.father_id == character_id)
        })
        .map(|row| row.due_minute)
        .min();
    let marriage = ctx
        .db
        .marriage()
        .iter()
        .filter(|row| {
            row.first_character_id == character_id || row.second_character_id == character_id
        })
        .flat_map(|row| [Some(row.married_minute), row.resolved_minute])
        .flatten()
        .filter(|minute| start_minute < *minute && *minute < end_minute)
        .min();
    let inheritance = ctx
        .db
        .estate_disposition()
        .chosen_heir_id()
        .filter(character_id)
        .filter(|row| {
            row.status == EstateDispositionStatus::Pending
                && start_minute < row.effective_minute
                && row.effective_minute < end_minute
        })
        .map(|row| row.effective_minute)
        .min();
    wedding
        .into_iter()
        .chain(birth)
        .chain(marriage)
        .chain(birthday)
        .chain(inheritance)
        .min()
}

pub fn initialize_npc_policy(
    ctx: &ReducerContext,
    character_id: u64,
    home_settlement_id: String,
    policy_seed: Seed,
) -> Result<(), String> {
    if ctx.db.character().id().find(character_id).is_none() {
        return Err("NPC policy requires a full Character".into());
    }
    crate::character::validate_full_character_components(ctx, character_id)?;
    if ctx
        .db
        .npc_policy()
        .character_id()
        .find(character_id)
        .is_none()
    {
        ctx.db.npc_policy().insert(NpcPolicy {
            character_id,
            home_settlement_id,
            policy_seed,
        });
    }
    Ok(())
}

/// A deliberately narrow advancement primitive. NPC policies use it to move
/// their ordinary CharacterTime and settle due canonical events atomically;
/// player travel, schedules, health, and account authority never run through
/// this path.
pub fn advance_npc_personal_time(
    ctx: &ReducerContext,
    character_id: u64,
    target_minute: StrategicMinute,
) -> Result<(), String> {
    if ctx
        .db
        .npc_policy()
        .character_id()
        .find(character_id)
        .is_none()
    {
        return Err("Character is not NPC-policy controlled".into());
    }
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or("NPC CharacterTime record not found")?;
    if target_minute < time.minutes {
        return Err("Canonical NPC time cannot be written retroactively".into());
    }
    if let Some(boundary) = next_lifecycle_boundary(ctx, character_id, time.minutes, target_minute)
    {
        advance_npc_personal_time(ctx, character_id, boundary)?;
        if ctx
            .db
            .npc_policy()
            .character_id()
            .find(character_id)
            .is_none()
        {
            // Adult promotion can transfer this clock to browser authority.
            // Leave it at the exact birthday instead of applying the rest of
            // an NPC-policy interval after that transfer.
            return Ok(());
        }
        return advance_npc_personal_time(ctx, character_id, target_minute);
    }
    // Delayed events are settled inside this transaction at the target
    // frontier. Any error rolls back both their effects and the clock, so an
    // NPC can never skip a due event by retaining an advanced date.
    time.minutes = target_minute;
    ctx.db.character_time().character_id().update(time);
    crate::time::settle_lifecycle_after_character_time_write(ctx, character_id, target_minute)
}

/// Internal-only scaffolding reducer.  Production NPC generation invokes the
/// same initializer; exposing no account-owned path prevents accidental NPC
/// time mutation through a player reducer.
#[reducer]
pub fn seed_npc_policy_for_development(
    ctx: &ReducerContext,
    character_id: u64,
    home_settlement_id: String,
    policy_seed: Seed,
) -> Result<(), String> {
    if ctx.sender() != ctx.database_identity() {
        return Err("Only database administration can seed NPC policy".into());
    }
    initialize_npc_policy(ctx, character_id, home_settlement_id, policy_seed)
}
