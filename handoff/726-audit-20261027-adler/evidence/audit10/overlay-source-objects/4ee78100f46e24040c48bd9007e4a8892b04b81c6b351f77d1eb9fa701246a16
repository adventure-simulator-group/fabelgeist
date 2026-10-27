/// Descendant maturity and observer-frontier visibility.
fn validate_descendant_grant(
    ctx: &ReducerContext,
    owner_key: &String,
    character_id: u64,
    minute: StrategicMinute,
) -> Result<(), String> {
    let birth = ctx
        .db
        .character_birth()
        .character_id()
        .find(character_id)
        .ok_or("Adult descendant birth coordinate not found")?;
    let adulthood_minute = StrategicMinute::at_signed_birth_age(
        birth.birth_minute,
        adventuresim_core::courtship::ADULT_AGE_YEARS,
    )
    .ok_or("Adult descendant adulthood coordinate is invalid")?;
    let selected_living_observer_minute = ctx
        .db
        .browser_character_selection()
        .owner_key()
        .find(owner_key)
        .and_then(|selection| {
            ctx.db
                .character()
                .id()
                .find(selection.character_id)
                .filter(|selected| selected.alive)?;
            ctx.db
                .character_time()
                .character_id()
                .find(selection.character_id)
                .map(|time| time.minutes)
        });
    if !descendant_grant_visible_at(adulthood_minute, selected_living_observer_minute, minute) {
        return Err(
            "Adult descendant is not yet visible at the selected character's date".into(),
        );
    }
    Ok(())
}

fn descendant_grant_visible_at(
    adulthood_minute: StrategicMinute,
    selected_living_observer_minute: Option<StrategicMinute>,
    descendant_frontier: StrategicMinute,
) -> bool {
    adulthood_minute <= selected_living_observer_minute.unwrap_or(descendant_frontier)
}

fn selected_living_observer_minute_for_grant(
    ctx: &ViewContext,
    grant: &BrowserCharacterGrant,
) -> Option<StrategicMinute> {
    let selection = ctx
        .db
        .browser_character_selection()
        .owner_key()
        .find(&grant.owner_key)?;
    let selected_grant = ctx
        .db
        .browser_character_grant()
        .character_id()
        .find(selection.character_id)?;
    let selected_character = ctx.db.character().id().find(selection.character_id)?;
    if selected_grant.owner_key != grant.owner_key || !selected_character.alive {
        return None;
    }
    ctx.db
        .character_time()
        .character_id()
        .find(selection.character_id)
        .map(|time| time.minutes)
}

fn adulthood_minute_for_view(ctx: &ViewContext, character_id: u64) -> Option<StrategicMinute> {
    let birth = ctx.db.character_birth().character_id().find(character_id)?;
    StrategicMinute::at_signed_birth_age(
        birth.birth_minute,
        adventuresim_core::courtship::ADULT_AGE_YEARS,
    )
}

fn effective_age_years_for_view(
    ctx: &ViewContext,
    character_id: u64,
    minute: StrategicMinute,
) -> Option<u16> {
    let character = ctx.db.character().id().find(character_id)?;
    let Some(birth) = ctx.db.character_birth().character_id().find(character_id) else {
        return Some(character.age_years);
    };
    Some(minute.age_years_since_signed_birth(birth.birth_minute))
}
