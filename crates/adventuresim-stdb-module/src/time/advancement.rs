// Owns travel, investigation, party synchronization, and neutral wait advancement policy.
include!("advancement/lifecycle.rs");

/// Record time spent travelling without applying recovery, activities, or
/// training. Travel time belongs only to the character's personal clock.
pub fn advance_character_time(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    minutes: u64,
) -> Result<bool, String> {
    crate::character::require_living_character(ctx, (character_id).into())?;
    initialize_character_time(ctx, (character_id).into())?;
    let mut character_time = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .ok_or_else(|| "Character time record not found".to_string())?;
    let starting_minute = character_time.minutes;
    if let Some(first) =
        first_lifecycle_segment(ctx, (character_id).into(), starting_minute, minutes)
    {
        if !advance_character_time(ctx, (character_id).into(), first)? {
            return Ok(false);
        }
        return advance_character_time(ctx, (character_id).into(), minutes.saturating_sub(first));
    }
    let injury_limit = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        minutes,
        InjuryRecoveryMinutes::NONE,
    )?
    .elapsed;
    let (elapsed, terminal) =
        crate::disease::clip_elapsed_for_disease(ctx, (character_id).into(), injury_limit, false)?;
    let settled = crate::surgery::settle_injuries(
        ctx,
        (character_id).into(),
        elapsed,
        InjuryRecoveryMinutes::NONE,
    )?;
    let elapsed = settled.elapsed;
    let interval_end = character_time.minutes.saturating_add_minutes(elapsed);
    character_time.minutes = interval_end;
    ctx.db
        .character_time()
        .character_id()
        .update(character_time);
    crate::condition::apply_weather_exposure(
        ctx,
        (character_id).into(),
        starting_minute,
        elapsed,
        true,
        ExposureShelter::Field(FieldShelter::Bivouac),
    )?;
    crate::organization::settle_membership_dues(ctx, (character_id).into())?;
    crate::social::settle_shared_party_time(ctx, (character_id).into());
    crate::disease::finish_disease_interval(ctx, (character_id).into(), terminal)?;
    settle_lifecycle_after_character_time_write(ctx, (character_id).into(), interval_end)?;
    advance_married_family_by(ctx, (character_id).into(), elapsed)?;
    if terminal.is_some() || !settled.alive {
        return Ok(false);
    }
    crate::condition::apply_travel_condition(
        ctx,
        (character_id).into(),
        starting_minute,
        elapsed,
        0,
    )?;
    crate::condition::refresh_character_strategic_condition(ctx, (character_id).into())?;
    crate::capability::refresh_character_capability(ctx, (character_id).into())?;
    Ok(true)
}

fn advance_character_time_in_plan(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    minutes: u64,
    plan: &crate::disease::PartyDiseaseIntervalPlan,
) -> Result<bool, String> {
    crate::character::require_living_character(ctx, (character_id).into())?;
    initialize_character_time(ctx, (character_id).into())?;
    let mut character_time = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .ok_or_else(|| "Character time record not found".to_string())?;
    let starting_minute = character_time.minutes;
    if let Some(first) =
        first_lifecycle_segment(ctx, (character_id).into(), starting_minute, minutes)
    {
        if !advance_character_time_in_plan(ctx, (character_id).into(), first, plan)? {
            return Ok(false);
        }
        return advance_character_time_in_plan(
            ctx,
            (character_id).into(),
            minutes.saturating_sub(first),
            plan,
        );
    }
    let injury_limit = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        minutes,
        InjuryRecoveryMinutes::NONE,
    )?
    .elapsed;
    let (elapsed, terminal) = crate::disease::clip_elapsed_for_disease_in_plan(
        ctx,
        (character_id).into(),
        injury_limit,
        false,
        plan,
    )?;
    let settled = crate::surgery::settle_injuries(
        ctx,
        (character_id).into(),
        elapsed,
        InjuryRecoveryMinutes::NONE,
    )?;
    let elapsed = settled.elapsed;
    character_time.minutes = character_time.minutes.saturating_add_minutes(elapsed);
    ctx.db
        .character_time()
        .character_id()
        .update(character_time);
    crate::organization::settle_membership_dues(ctx, (character_id).into())?;
    crate::social::settle_shared_party_time(ctx, (character_id).into());
    crate::disease::finish_disease_interval(ctx, (character_id).into(), terminal)?;
    settle_lifecycle_after_character_time_write(
        ctx,
        (character_id).into(),
        starting_minute.saturating_add_minutes(settled.elapsed),
    )?;
    advance_married_family_by(ctx, (character_id).into(), elapsed)?;
    if terminal.is_some() || !settled.alive {
        return Ok(false);
    }
    crate::condition::apply_travel_condition(
        ctx,
        (character_id).into(),
        starting_minute,
        elapsed,
        0,
    )?;
    crate::condition::refresh_character_strategic_condition(ctx, (character_id).into())?;
    crate::capability::refresh_character_capability(ctx, (character_id).into())?;
    Ok(true)
}

/// Actual strategic movement, split at exact dirt boundaries so filth and its
/// wound-risk multiplier are independent of caller chunking.
pub fn preview_travel_time(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    requested: u64,
) -> Result<u64, String> {
    let injury = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        requested,
        InjuryRecoveryMinutes::NONE,
    )?;
    crate::disease::preview_elapsed_for_disease(ctx, (character_id).into(), injury.elapsed, false)
}

pub fn preview_travel_time_in_plan(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    requested: u64,
    plan: &crate::disease::PartyDiseaseIntervalPlan,
) -> Result<u64, String> {
    let injury = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        requested,
        InjuryRecoveryMinutes::NONE,
    )?;
    crate::disease::preview_elapsed_for_disease_in_plan(
        ctx,
        (character_id).into(),
        injury.elapsed,
        false,
        plan,
    )
}

/// Commit a terminal injury or disease event that falls exactly on the
/// character's current strategic minute. This intentionally grants no elapsed
/// travel time, condition use, filth, or training.
pub fn settle_travel_boundary(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<bool, String> {
    let character = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .ok_or("Character not found")?;
    if !character.alive {
        return Ok(false);
    }
    advance_character_time(ctx, (character_id).into(), 0)
}

pub fn advance_travel_time(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    mut minutes: u64,
) -> Result<bool, String> {
    while minutes > 0 {
        let chunk = minutes.min(crate::filth::next_travel_dirt_boundary(
            ctx,
            (character_id).into(),
        ));
        let before = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(character_id))
            .map_or(StrategicMinute::ZERO, |t| t.minutes);
        let alive = advance_character_time(ctx, (character_id).into(), chunk)?;
        let after = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(character_id))
            .map_or(before, |t| t.minutes);
        let elapsed = after.elapsed_since(before);
        crate::filth::record_travel_elapsed(ctx, (character_id).into(), elapsed, after)?;
        if !alive || elapsed < chunk {
            return Ok(false);
        }
        minutes -= elapsed;
    }
    Ok(true)
}

pub fn advance_travel_time_in_plan(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    mut minutes: u64,
    plan: &crate::disease::PartyDiseaseIntervalPlan,
) -> Result<bool, String> {
    while minutes > 0 {
        let chunk = minutes.min(crate::filth::next_travel_dirt_boundary(
            ctx,
            (character_id).into(),
        ));
        let before = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(character_id))
            .map_or(StrategicMinute::ZERO, |t| t.minutes);
        let alive = advance_character_time_in_plan(ctx, (character_id).into(), chunk, plan)?;
        let after = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(character_id))
            .map_or(before, |t| t.minutes);
        let elapsed = after.elapsed_since(before);
        crate::filth::record_travel_elapsed(ctx, (character_id).into(), elapsed, after)?;
        if !alive || elapsed < chunk {
            return Ok(false);
        }
        minutes -= elapsed;
    }
    Ok(true)
}

/// Stationary but strenuous strategic time used by investigation actions.
/// Unlike neutral waiting this applies ordinary needs and the same fatigue
/// reservoir as travel, but it never records movement filth or terrain
/// exposure.
pub fn advance_investigation_time(
    ctx: &ReducerContext,
    character_id: u64,
    minutes: u64,
) -> Result<bool, String> {
    let mut remaining = minutes;
    while remaining > 0 {
        let safe = preview_travel_time(ctx, (character_id).into(), remaining)?;
        if safe == 0 {
            return settle_travel_boundary(ctx, (character_id).into());
        }
        let before = ctx
            .db
            .character_time()
            .character_id()
            .find(character_id)
            .map_or(StrategicMinute::ZERO, |t| t.minutes);
        let alive = advance_character_time(ctx, (character_id).into(), safe)?;
        let after = ctx
            .db
            .character_time()
            .character_id()
            .find(character_id)
            .map_or(before, |t| t.minutes);
        let elapsed = after.elapsed_since(before);
        if !alive || elapsed < safe {
            return Ok(false);
        }
        remaining -= elapsed;
    }
    Ok(true)
}

/// Bring a co-located party to one strategic minute before an atomic shared
/// activity. The furthest-advanced member is authoritative; lagging members
/// settle ordinary stationary time before the strenuous interval begins.
pub(crate) fn synchronize_party_activity_time(
    ctx: &ReducerContext,
    member_ids: &[adventuresim_core::identity::CharacterId],
    leader_id: adventuresim_core::identity::CharacterId,
) -> Result<Option<StrategicMinute>, String> {
    if !member_ids.contains(&adventuresim_core::identity::CharacterId::from(leader_id)) {
        return Err("Party leader is not a living activity participant".into());
    }
    if member_ids.iter().any(|member_id| {
        ctx.db
            .character()
            .id()
            .find(u64::from(*member_id))
            .is_none_or(|character| !character.alive)
    }) {
        if let Some(party_id) = ctx
            .db
            .character()
            .id()
            .find(u64::from(leader_id))
            .and_then(|character| character.party_id)
        {
            let _ = crate::strategic::normalize_and_elect_party_leader(ctx, &party_id);
        }
        return Ok(None);
    }
    for member_id in member_ids {
        initialize_character_time(ctx, (*member_id).into())?;
    }
    Ok(Some(
        ctx.db
            .character_time()
            .character_id()
            .find(u64::from(leader_id))
            .ok_or("Party leader has no subjective clock")?
            .minutes,
    ))
}

/// Validate co-location before a shared strategic action. Party members retain
/// independent subjective clocks; shared activity never ages one participant
/// merely to match another.
#[reducer]
pub fn synchronize_party_for_activity(ctx: &ReducerContext, leader_id: u64) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, (leader_id).into())
        .map_err(|error: crate::strategic::StrategicCharacterAuthorityError| error.to_string())?;
    let leader = crate::character::require_living_character(ctx, (leader_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    let party_id = leader
        .party_id
        .clone()
        .ok_or("Activity synchronization requires a party")?;
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(&party_id)
        .ok_or("Party not found")?;
    if party.leader_id != leader_id {
        return Err("Only the current party leader can synchronize party activity".into());
    }
    let member_ids = crate::strategic::living_party_member_ids(ctx, &party_id);
    if member_ids.is_empty()
        || !member_ids.contains(&adventuresim_core::identity::CharacterId::from(leader_id))
    {
        return Err("Party has no living leader for activity synchronization".into());
    }
    for member_id in &member_ids {
        let member = ctx
            .db
            .character()
            .id()
            .find(u64::from(*member_id))
            .ok_or("Party member not found")?;
        let together_at_settlement = leader.current_settlement_id.is_some()
            && member.current_settlement_id == leader.current_settlement_id
            && member.current_settlement_id == party.current_settlement_id;
        let together_at_party_case_site =
            party
                .current_case_site_id
                .as_ref()
                .is_some_and(|case_site_id| {
                    let leader_occupancy =
                        crate::investigation::current_character_case_site_occupancy(
                            ctx,
                            (leader_id).into(),
                        );
                    let member_occupancy =
                        crate::investigation::current_character_case_site_occupancy(
                            ctx,
                            (*member_id).into(),
                        );
                    leader_occupancy
                        .zip(member_occupancy)
                        .is_some_and(|(leader, member)| {
                            leader.case_site_id == *case_site_id
                                && member.case_site_id == *case_site_id
                        })
                });
        if member.party_id.as_deref() != Some(party_id.as_str())
            || !(together_at_settlement
                || together_at_party_case_site
                || crate::world_actor::characters_are_contextually_present(
                    ctx,
                    (leader_id).into(),
                    (*member_id).into(),
                ))
        {
            return Err("Party members must be co-located before activity synchronization".into());
        }
    }
    for member_id in &member_ids {
        initialize_character_time(ctx, (*member_id).into())
            .map_err(|error: crate::time::WorldClockError| error.to_string())?;
    }
    let _ = crate::strategic::normalize_and_elect_party_leader(ctx, &party_id);
    for member_id in crate::strategic::living_party_member_ids(ctx, &party_id) {
        let _ = crate::condition::refresh_character_strategic_condition(ctx, (member_id).into());
        let _ = crate::capability::refresh_character_capability(ctx, (member_id).into());
    }
    Ok(())
}

/// Neutral/location-appropriate personal time for waiting and procedures. It
/// advances disease, wounds, blood, and ordinary recovery without applying
/// travel fatigue or travel needs.
pub fn advance_character_wait_time(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    minutes: u64,
) -> Result<bool, String> {
    crate::character::require_living_character(ctx, (character_id).into())?;
    initialize_character_time(ctx, (character_id).into())?;
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character time record not found")?;
    let starting_minute = time.minutes;
    if let Some(first) =
        first_lifecycle_segment(ctx, (character_id).into(), starting_minute, minutes)
    {
        if !advance_character_wait_time(ctx, (character_id).into(), first)? {
            return Ok(false);
        }
        return advance_character_wait_time(
            ctx,
            (character_id).into(),
            minutes.saturating_sub(first),
        );
    }
    let injury_limit = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        minutes,
        InjuryRecoveryMinutes::new(minutes),
    )?
    .elapsed;
    let (elapsed, terminal) =
        crate::disease::clip_elapsed_for_disease(ctx, (character_id).into(), injury_limit, true)?;
    let settled = crate::surgery::settle_injuries(
        ctx,
        (character_id).into(),
        elapsed,
        InjuryRecoveryMinutes::new(elapsed),
    )?;
    time.minutes = time.minutes.saturating_add_minutes(settled.elapsed);
    ctx.db.character_time().character_id().update(time);
    advance_married_family_by(ctx, (character_id).into(), settled.elapsed)?;
    let at_settlement = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .is_some_and(|row| row.current_settlement_id.is_some());
    crate::condition::apply_weather_exposure(
        ctx,
        (character_id).into(),
        starting_minute,
        settled.elapsed,
        false,
        if at_settlement {
            ExposureShelter::Indoor
        } else {
            ExposureShelter::Field(FieldShelter::Bivouac)
        },
    )?;
    crate::organization::settle_membership_dues(ctx, (character_id).into())?;
    crate::social::settle_shared_party_time(ctx, (character_id).into());
    crate::disease::finish_disease_interval(ctx, (character_id).into(), terminal)?;
    settle_lifecycle_after_character_time_write(
        ctx,
        (character_id).into(),
        starting_minute.saturating_add_minutes(settled.elapsed),
    )?;
    if terminal.is_some() || !settled.alive {
        return Ok(false);
    }
    if at_settlement {
        crate::condition::apply_rest_condition(ctx, (character_id).into(), settled.elapsed)?;
    } else {
        crate::condition::apply_elapsed_needs(ctx, (character_id).into(), settled.elapsed)?;
        crate::condition::apply_camp_rest_recovery_condition(
            ctx,
            (character_id).into(),
            settled.elapsed,
        )?;
    }
    crate::capability::refresh_character_capability(ctx, (character_id).into())?;
    Ok(true)
}

pub fn advance_character_wait_time_in_plan(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    minutes: u64,
    plan: &crate::disease::PartyDiseaseIntervalPlan,
) -> Result<bool, String> {
    crate::character::require_living_character(ctx, (character_id).into())?;
    initialize_character_time(ctx, (character_id).into())?;
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character time record not found")?;
    let starting_minute = time.minutes;
    if let Some(first) =
        first_lifecycle_segment(ctx, (character_id).into(), starting_minute, minutes)
    {
        if !advance_character_wait_time_in_plan(ctx, (character_id).into(), first, plan)? {
            return Ok(false);
        }
        return advance_character_wait_time_in_plan(
            ctx,
            (character_id).into(),
            minutes.saturating_sub(first),
            plan,
        );
    }
    let injury_limit = crate::surgery::preview_injury_boundary(
        ctx,
        (character_id).into(),
        minutes,
        InjuryRecoveryMinutes::new(minutes),
    )?
    .elapsed;
    let (elapsed, terminal) = crate::disease::clip_elapsed_for_disease_in_plan(
        ctx,
        (character_id).into(),
        injury_limit,
        true,
        plan,
    )?;
    let settled = crate::surgery::settle_injuries(
        ctx,
        (character_id).into(),
        elapsed,
        InjuryRecoveryMinutes::new(elapsed),
    )?;
    time.minutes = time.minutes.saturating_add_minutes(settled.elapsed);
    ctx.db.character_time().character_id().update(time);
    advance_married_family_by(ctx, (character_id).into(), settled.elapsed)?;
    crate::organization::settle_membership_dues(ctx, (character_id).into())?;
    crate::social::settle_shared_party_time(ctx, (character_id).into());
    crate::disease::finish_disease_interval(ctx, (character_id).into(), terminal)?;
    settle_lifecycle_after_character_time_write(
        ctx,
        (character_id).into(),
        starting_minute.saturating_add_minutes(settled.elapsed),
    )?;
    if terminal.is_some() || !settled.alive {
        return Ok(false);
    }
    let at_settlement = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .is_some_and(|row| row.current_settlement_id.is_some());
    if at_settlement {
        crate::condition::apply_rest_condition(ctx, (character_id).into(), settled.elapsed)?;
    } else {
        crate::condition::apply_elapsed_needs(ctx, (character_id).into(), settled.elapsed)?;
        crate::condition::apply_camp_rest_recovery_condition(
            ctx,
            (character_id).into(),
            settled.elapsed,
        )?;
    }
    crate::capability::refresh_character_capability(ctx, (character_id).into())?;
    Ok(true)
}
