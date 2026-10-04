// Owns official and personal clock initialization, refresh, and family-clock propagation.
pub fn initialize_time(ctx: &ReducerContext) {
    if ctx.db.world_clock().id().find(0).is_none() {
        ctx.db.world_clock().insert(WorldClock {
            id: 0,
            official_minutes: WORLD_START_MINUTE,
            epoch_micros: ctx.timestamp.to_micros_since_unix_epoch(),
        });
    }
}

pub(crate) fn refresh_clock(ctx: &ReducerContext) -> Result<StrategicMinute, WorldClockError> {
    if ctx.db.world_clock().id().find(0).is_none() {
        initialize_time(ctx);
    }
    let mut clock = ctx
        .db
        .world_clock()
        .id()
        .find(0)
        .ok_or(WorldClockError::NotInitialized)?;
    let official_minutes = OfficialClockEpoch::from(clock.epoch_micros).at(
        UnixMicrosecondInstant::from(ctx.timestamp.to_micros_since_unix_epoch()),
    );
    if official_minutes != clock.official_minutes {
        clock.official_minutes = official_minutes;
        ctx.db.world_clock().id().update(clock);
    }
    Ok(official_minutes)
}

fn married_family_npc_ids(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Vec<adventuresim_core::identity::CharacterId> {
    let character_party_id = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .and_then(|character| character.party_id);
    let active_marriage = ctx.db.marriage().iter().find(|row| {
        row.status == MarriageStatus::Active
            && (adventuresim_core::identity::CharacterId::from(row.first_character_id)
                == character_id
                || adventuresim_core::identity::CharacterId::from(row.second_character_id)
                    == character_id)
    });
    let Some(marriage) = active_marriage else {
        return Vec::new();
    };
    let mut related = std::collections::BTreeSet::from([
        adventuresim_core::identity::CharacterId::from(marriage.first_character_id),
        adventuresim_core::identity::CharacterId::from(marriage.second_character_id),
    ]);
    related.extend(
        ctx.db
            .household_member()
            .household_id()
            .filter(&marriage.household_id)
            .map(|row| adventuresim_core::identity::CharacterId::from(row.character_id)),
    );
    related.extend(
        ctx.db
            .character_kinship()
            .subject_id()
            .filter(u64::from(character_id))
            .map(|row| adventuresim_core::identity::CharacterId::from(row.related_id)),
    );
    related.remove(&character_id);
    related
        .into_iter()
        .filter(|related_id| {
            ctx.db
                .npc_policy()
                .character_id()
                .find(u64::from(*related_id))
                .is_some()
        })
        .filter(|related_id| {
            ctx.db
                .character()
                .id()
                .find(u64::from(*related_id))
                .is_none_or(|related| related.party_id != character_party_id)
        })
        .take(32)
        .collect()
}

fn advance_married_family_by(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
    elapsed: u64,
) -> Result<(), String> {
    if elapsed == 0
        || ctx
            .db
            .npc_policy()
            .character_id()
            .find(u64::from(character_id))
            .is_some()
    {
        return Ok(());
    }
    for related_id in married_family_npc_ids(ctx, (character_id).into()) {
        if !ctx
            .db
            .character()
            .id()
            .find(u64::from(related_id))
            .is_some_and(|character| character.alive)
        {
            continue;
        }
        initialize_character_time(ctx, (related_id).into())?;
        let target = ctx
            .db
            .character_time()
            .character_id()
            .find(u64::from(related_id))
            .ok_or("Married family member has no subjective clock")?
            .minutes;
        let target = target.saturating_add_minutes(elapsed);
        advance_stationary_character_to(ctx, (related_id).into(), target)?;
    }
    Ok(())
}
