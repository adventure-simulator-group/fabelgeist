fn wedding_participant_death(
    ctx: &ReducerContext,
    commitment: &ExclusiveCommitment,
) -> Option<StrategicMinute> {
    [
        commitment.first_character_id,
        commitment.second_character_id,
    ]
    .into_iter()
    .filter_map(|character_id| {
        ctx.db
            .character_death()
            .character_id()
            .find(character_id)
            .map(|death| death.strategic_minute)
    })
    .filter(|minute| *minute <= commitment.effective_minute)
    .min()
}

fn prepare_wedding_residence(
    ctx: &ReducerContext,
    commitment: &ExclusiveCommitment,
    holding_id: &str,
    now: StrategicMinute,
) -> crate::StrategicOperationResult<bool> {
    let minute = commitment.effective_minute;
    if let Err(error) = crate::residence::preflight_residence_occupants_at(
        ctx,
        holding_id,
        [
            commitment.first_character_id,
            commitment.second_character_id,
        ],
        minute,
    ) {
        record_lifecycle_failure(
            ctx,
            LifecycleEventKind::Wedding,
            &commitment.id,
            minute,
            now,
            error.to_string(),
        );
        transition_commitment_terminal(
            ctx,
            commitment.clone(),
            CommitmentStatus::Cancelled,
            CommitmentTerminalReason::ResidenceUnavailable,
            minute,
        )?;
        return Ok(false);
    }
    Ok(true)
}

