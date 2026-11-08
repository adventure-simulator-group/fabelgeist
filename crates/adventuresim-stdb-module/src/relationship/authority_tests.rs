// Guarded acceptance of canonical chronology at historical conception trials.

#[reducer]
pub fn authority_test_conception_chronology(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    use crate::character::{character, character_death, CharacterDeath, DeathCause, DeathSource};
    use crate::personality::character_personality;
    use crate::time::character_time;
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mother = 732011;
    let father = 732012;
    for (id, sex) in [(mother, Sex::Female), (father, Sex::Male)] {
        crate::character::create_named_character_with_id(ctx, id, "Chronology Fixture".into())?;
        let mut personality = ctx.db.character_personality().character_id().find(id).ok_or("Personality missing")?;
        personality.sex = sex;
        ctx.db.character_personality().character_id().update(personality);
        let mut birth = ctx.db.character_birth().character_id().find(id).ok_or("Birth missing")?;
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
    if conception_parents(ctx, mother, father, StrategicMinute::ZERO)?.is_some()
        || conception_parents(ctx, mother, father, StrategicMinute::new(10))?.is_some()
        || conception_parents(ctx, mother, father, adulthood.saturating_sub_minutes(1))?.is_some()
        || conception_parents(ctx, mother, father, adulthood)? != Some((mother, father))
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
        let mut clock = ctx.db.character_time().character_id().find(id).ok_or("Clock missing")?;
        clock.minutes = death.saturating_add_minutes(1);
        ctx.db.character_time().character_id().update(clock);
    }
    let mut row = ctx.db.character().id().find(mother).ok_or("Mother missing")?;
    row.alive = false;
    row.age_years = 0;
    ctx.db.character().id().update(row);
    if conception_parents(ctx, mother, father, adulthood)? != Some((mother, father))
        || conception_parents(ctx, mother, father, death)?.is_some()
        || conception_parents(ctx, mother, father, death.saturating_add_minutes(1))?.is_some()
    {
        return Err("Conception trusted current fields instead of historical death authority".into());
    }
    ctx.db.character_birth().character_id().delete(father);
    if effective_age_years(ctx, father, adulthood).is_some()
        || character_alive_at(ctx, father, adulthood)
        || conception_parents(ctx, mother, father, adulthood)?.is_some()
    {
        return Err("Missing birth authority fell back to a current projection".into());
    }
    Ok(())
}

/// Exercise a capacity-rejected wedding through the chronological queue.
#[reducer]
pub fn authority_test_wedding_capacity_atomicity(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const FIRST: u64 = 726_001;
    const SECOND: u64 = 726_002;
    const FATHER: u64 = 726_004;
    const COMMITMENT: &str = "authority-joint-capacity-wedding";
    const OLD_HOUSEHOLD: &str = "authority-joint-capacity-old-household";
    const SETTLEMENT: &str = "authority-joint-capacity";
    const DOWRY: u32 = 17;
    let ceremony = StrategicMinute::new(100);
    let now = crate::time::refresh_clock(ctx)?;
    for character_id in [FIRST, SECOND, FATHER] {
        crate::character::create_named_character_with_id(
            ctx, character_id, format!("Wedding acceptance {character_id}"),
        )?;
        set_seeded_character_birth_from_age(ctx, character_id, 25);
        let mut time = ctx.db.character_time().character_id().find(character_id)
            .ok_or("Wedding acceptance clock missing")?;
        time.minutes = now;
        ctx.db.character_time().character_id().update(time);
    }
    ctx.db.household().insert(Household {
        id: OLD_HOUSEHOLD.into(), home_settlement_id: SETTLEMENT.into(),
        created_minute: StrategicMinute::ZERO,
    });
    for character_id in [FIRST, SECOND] {
        join_household(ctx, OLD_HOUSEHOLD, character_id, StrategicMinute::ZERO, HouseholdRole::AdultChild);
    }
    crate::item::credit_personal_currency(ctx, FATHER, SETTLEMENT, DOWRY)?;
    crate::item::consume_personal_currency(ctx, FATHER, u64::from(DOWRY))?;
    let before_father = crate::item::personal_currency_total(ctx, FATHER);
    let before_recipient = crate::item::personal_currency_total(ctx, SECOND);
    ctx.db.courtship().insert(CourtshipRecord {
        id: format!("courtship:{FIRST}:{SECOND}"),
        first_character_id: FIRST,
        second_character_id: SECOND,
        kind: CourtshipKind::Formal,
        status: CourtshipStatus::Active,
        secrecy_reason: None,
        approved_father_id: Some(FATHER),
        planned_dowry_amount: DOWRY,
        weaker_deception_baseline: 0.0,
        started_minute: StrategicMinute::ZERO,
        next_discovery_day: 0,
        resolved_minute: None,
        terminal_reason: None,
    });
    ctx.db.exclusive_commitment().insert(ExclusiveCommitment {
        id: COMMITMENT.into(), first_character_id: FIRST, second_character_id: SECOND,
        kind: CommitmentKind::Engagement, status: CommitmentStatus::Reserved,
        ceremony_settlement_id: SETTLEMENT.into(), effective_minute: ceremony,
        created_minute: StrategicMinute::ZERO, resolved_minute: None, terminal_reason: None,
    });
    for character_id in [FIRST, SECOND] {
        ctx.db.exclusive_commitment_participant().insert(ExclusiveCommitmentParticipant {
            character_id, commitment_id: COMMITMENT.into(),
        });
    }
    ctx.db.dowry_escrow().insert(DowryEscrow {
        commitment_id: COMMITMENT.into(), father_id: FATHER, amount: DOWRY,
        reserved_minute: StrategicMinute::ZERO,
    });
    settle_due_lifecycle_events_global(ctx, now, usize::MAX)?;
    let commitment = ctx.db.exclusive_commitment().id().find(COMMITMENT.to_owned())
        .ok_or("Wedding acceptance commitment missing")?;
    if commitment.status != CommitmentStatus::Cancelled
        || commitment.terminal_reason != Some(CommitmentTerminalReason::ResidenceUnavailable)
        || ctx.db.dowry_escrow().commitment_id().find(COMMITMENT.to_owned()).is_some()
        || crate::item::personal_currency_total(ctx, FATHER) != before_father + u64::from(DOWRY)
        || crate::item::personal_currency_total(ctx, SECOND) != before_recipient
        || ctx.db.marriage().id().find(format!("marriage:{COMMITMENT}")).is_some()
        || [FIRST, SECOND].into_iter().any(|character_id| {
            ctx.db.household_member().character_id().find(character_id)
                .is_none_or(|member| member.household_id != OLD_HOUSEHOLD)
        })
    {
        return Err("Rejected wedding changed household or mishandled escrow".into());
    }
    Ok(())
}

/// Seed an inconsistent terminal marker so final settlement fails after writes.
#[reducer]
pub fn authority_test_prepare_wedding_rollback(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const FIRST: u64 = 726_001;
    const SECOND: u64 = 726_002;
    const FATHER: u64 = 726_004;
    const COMMITMENT: &str = "authority-joint-capacity-rollback";
    let ceremony = StrategicMinute::new(100);
    crate::item::consume_personal_currency(ctx, FATHER, 17)?;
    ctx.db.exclusive_commitment().insert(ExclusiveCommitment {
        id: COMMITMENT.into(), first_character_id: FIRST, second_character_id: SECOND,
        kind: CommitmentKind::Engagement, status: CommitmentStatus::Reserved,
        ceremony_settlement_id: "authority-joint-capacity".into(), effective_minute: ceremony,
        created_minute: StrategicMinute::ZERO, resolved_minute: Some(ceremony), terminal_reason: None,
    });
    for character_id in [FIRST, SECOND] {
        ctx.db.exclusive_commitment_participant().insert(ExclusiveCommitmentParticipant {
            character_id, commitment_id: COMMITMENT.into(),
        });
    }
    ctx.db.dowry_escrow().insert(DowryEscrow {
        commitment_id: COMMITMENT.into(), father_id: FATHER, amount: 17,
        reserved_minute: StrategicMinute::ZERO,
    });
    Ok(())
}

/// This must return an error after the wedding commit path has begun.
#[reducer]
pub fn authority_test_execute_wedding_rollback(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let now = crate::time::refresh_clock(ctx)?;
    settle_due_lifecycle_events_global(ctx, now, usize::MAX)?;
    Err("Expected malformed wedding settlement to fail".into())
}

/// Check the state visible in a later transaction after the failed call.
#[reducer]
pub fn authority_test_verify_wedding_rollback(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const COMMITMENT: &str = "authority-joint-capacity-rollback";
    if ctx.db.dowry_escrow().commitment_id().find(COMMITMENT.to_owned()).is_none()
        || ctx.db.marriage().id().find(format!("marriage:{COMMITMENT}")).is_some()
        || ctx.db.household().id().find(format!("household:{COMMITMENT}")).is_some()
        || [726_001, 726_002].into_iter().any(|character_id| {
            ctx.db.household_member().character_id().find(character_id)
                .is_none_or(|member| member.household_id != "authority-joint-capacity-old-household")
        })
    {
        return Err("Unexpected wedding error committed partial settlement".into());
    }
    Ok(())
}

/// Remove the deliberately malformed fixture after its rollback is verified.
#[reducer]
pub fn authority_test_cleanup_wedding_rollback(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const COMMITMENT: &str = "authority-joint-capacity-rollback";
    let escrow = ctx.db.dowry_escrow().commitment_id().find(COMMITMENT.to_owned())
        .ok_or("Rollback escrow missing")?;
    crate::item::credit_personal_currency(ctx, escrow.father_id, "authority-joint-capacity", escrow.amount)?;
    ctx.db.dowry_escrow().commitment_id().delete(COMMITMENT.to_owned());
    ctx.db.exclusive_commitment().id().delete(COMMITMENT.to_owned());
    for character_id in [726_001, 726_002] {
        ctx.db.exclusive_commitment_participant().character_id().delete(character_id);
    }
    Ok(())
}

/// Confirm an admissible formal wedding completes and pays its escrow once.
#[reducer]
pub fn authority_test_wedding_capacity_success(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const FIRST: u64 = 726_001;
    const SECOND: u64 = 726_002;
    const FATHER: u64 = 726_004;
    const COMMITMENT: &str = "authority-joint-capacity-success";
    const DOWRY: u32 = 17;
    let ceremony = StrategicMinute::new(100);
    let now = crate::time::refresh_clock(ctx)?;
    let before_first = crate::item::personal_currency_total(ctx, FIRST);
    let before_second = crate::item::personal_currency_total(ctx, SECOND);
    let before_father = crate::item::personal_currency_total(ctx, FATHER);
    crate::item::consume_personal_currency(ctx, FATHER, u64::from(DOWRY))?;
    ctx.db.exclusive_commitment().insert(ExclusiveCommitment {
        id: COMMITMENT.into(), first_character_id: FIRST, second_character_id: SECOND,
        kind: CommitmentKind::Engagement, status: CommitmentStatus::Reserved,
        ceremony_settlement_id: "authority-joint-capacity".into(), effective_minute: ceremony,
        created_minute: StrategicMinute::ZERO, resolved_minute: None, terminal_reason: None,
    });
    for character_id in [FIRST, SECOND] {
        ctx.db.exclusive_commitment_participant().insert(ExclusiveCommitmentParticipant {
            character_id, commitment_id: COMMITMENT.into(),
        });
    }
    ctx.db.dowry_escrow().insert(DowryEscrow {
        commitment_id: COMMITMENT.into(), father_id: FATHER, amount: DOWRY,
        reserved_minute: StrategicMinute::ZERO,
    });
    settle_due_lifecycle_events_global(ctx, now, usize::MAX)?;
    let commitment = ctx.db.exclusive_commitment().id().find(COMMITMENT.to_owned())
        .ok_or("Successful wedding commitment missing")?;
    let household_id = format!("household:{COMMITMENT}");
    if commitment.status != CommitmentStatus::Fulfilled
        || commitment.terminal_reason != Some(CommitmentTerminalReason::WeddingCompleted)
        || ctx.db.dowry_escrow().commitment_id().find(COMMITMENT.to_owned()).is_some()
        || ctx.db.marriage().id().find(format!("marriage:{COMMITMENT}")).is_none()
        || crate::item::personal_currency_total(ctx, FATHER) + u64::from(DOWRY) != before_father
        || crate::item::personal_currency_total(ctx, FIRST)
            + crate::item::personal_currency_total(ctx, SECOND)
            != before_first + before_second + u64::from(DOWRY)
        || [FIRST, SECOND].into_iter().any(|character_id| {
            ctx.db.household_member().character_id().find(character_id)
                .is_none_or(|member| member.household_id != household_id)
        })
    {
        return Err("Admissible wedding failed to settle household and escrow".into());
    }
    Ok(())
}
