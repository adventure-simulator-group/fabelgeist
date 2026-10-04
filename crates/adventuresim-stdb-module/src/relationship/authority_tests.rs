// Guarded acceptance of canonical chronology at historical conception trials.

#[reducer]
pub fn authority_test_conception_chronology(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    use crate::character::{CharacterDeath, DeathCause, DeathSource, character, character_death};
    use crate::personality::character_personality;
    use crate::time::character_time;
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mother = 732011;
    let father = 732012;
    for (id, sex) in [(mother, Sex::Female), (father, Sex::Male)] {
        crate::character::create_named_character_with_id(ctx, id, "Chronology Fixture".into())?;
        let mut personality = ctx
            .db
            .character_personality()
            .character_id()
            .find(id)
            .ok_or("Personality missing")?;
        personality.sex = sex;
        ctx.db
            .character_personality()
            .character_id()
            .update(personality);
        let mut birth = ctx
            .db
            .character_birth()
            .character_id()
            .find(id)
            .ok_or("Birth missing")?;
        birth.birth_minute = 10;
        ctx.db.character_birth().character_id().update(birth);
    }
    ctx.db.marriage().insert(Marriage {
        id: "authority-test-marriage".into(),
        first_character_id: mother,
        second_character_id: father,
        commitment_id: "authority-test-commitment".into(),
        household_id: "authority-test-household".into(),
        ceremony_settlement_id: "authority-test-settlement".into(),
        married_minute: StrategicMinute::ZERO,
        status: MarriageStatus::Active,
        resolved_minute: None,
    });
    let adulthood = StrategicMinute::new(10).saturating_add_years(ADULT_AGE_YEARS);
    if conception_parents(ctx, mother.into(), father.into(), StrategicMinute::ZERO)?.is_some()
        || conception_parents(ctx, mother.into(), father.into(), StrategicMinute::new(10))?
            .is_some()
        || conception_parents(
            ctx,
            mother.into(),
            father.into(),
            adulthood.saturating_sub_minutes(1),
        )?
        .is_some()
        || conception_parents(ctx, mother.into(), father.into(), adulthood)?
            != Some((mother.into(), father.into()))
    {
        return Err("Conception eligibility disagrees with canonical birth/adulthood".into());
    }
    // Advance materialized state beyond the trial and deliberately leave stale
    // display projections. Neither cache may redefine the trial's history.
    let death = adulthood.saturating_add_minutes(10);
    ctx.db.character_death().insert(CharacterDeath {
        character_id: mother,
        cause: DeathCause::DevTest,
        source: DeathSource::DevTest,
        source_id: None,
        strategic_minute: death,
    });
    for id in [mother, father] {
        let mut clock = ctx
            .db
            .character_time()
            .character_id()
            .find(id)
            .ok_or("Clock missing")?;
        clock.minutes = death.saturating_add_minutes(1);
        ctx.db.character_time().character_id().update(clock);
    }
    let mut row = ctx
        .db
        .character()
        .id()
        .find(mother)
        .ok_or("Mother missing")?;
    row.alive = false;
    row.age_years = 0;
    ctx.db.character().id().update(row);
    if conception_parents(ctx, mother.into(), father.into(), adulthood)?
        != Some((mother.into(), father.into()))
        || conception_parents(ctx, mother.into(), father.into(), death)?.is_some()
        || conception_parents(
            ctx,
            mother.into(),
            father.into(),
            death.saturating_add_minutes(1),
        )?
        .is_some()
    {
        return Err(
            "Conception trusted current fields instead of historical death authority".into(),
        );
    }
    ctx.db.character_birth().character_id().delete(father);
    if effective_age_years(ctx, (father).into(), adulthood).is_some()
        || character_alive_at(ctx, (father).into(), adulthood)
        || conception_parents(ctx, mother.into(), father.into(), adulthood)?.is_some()
    {
        return Err("Missing birth authority fell back to a current projection".into());
    }
    Ok(())
}
