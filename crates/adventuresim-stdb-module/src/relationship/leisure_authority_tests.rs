//! Guarded checkpoint persistence and overlap idempotence.

use super::*;

#[reducer]
pub fn authority_test_leisure_checkpoints(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let first = 732021;
    let second = 732022;
    for id in [first, second] {
        crate::character::create_named_character_with_id(ctx, id, "Leisure Fixture".into())?;
    }
    let pair_id = format!("spouse-leisure:{first}:{second}");
    let mut start = 0;
    for duration in [17, 43, 71, 6] {
        let end = start + duration;
        for character_id in [first, second] {
            ctx.db.spouse_leisure_slice().insert(SpouseLeisureSlice {
                id: format!("checkpoint:{character_id}:{start}"),
                character_id,
                start_minute: StrategicMinute::new(start),
                end_minute: StrategicMinute::new(end),
                location_id: "riverdale".into(),
            });
        }
        settle_spouse_leisure_pair(ctx, first, second)?;
        let state = ctx
            .db
            .spouse_leisure_accrual()
            .pair_id()
            .find(&pair_id)
            .ok_or("Checkpoint missing")?;
        let whole = conception_quantum_plan(ConceptionQuantumState::default(), end);
        if state.conserved_joint_minutes != whole.state.conserved_joint_minutes
            || state.next_trial_ordinal != whole.state.next_trial_ordinal
        {
            return Err("Persisted checkpoint differs from whole-interval planning".into());
        }
        start = end;
    }
    let before = ctx
        .db
        .spouse_leisure_accrual()
        .pair_id()
        .find(&pair_id)
        .ok_or("Checkpoint missing")?;
    let receipts = ctx
        .db
        .conception_trial_receipt()
        .iter()
        .filter(|row| row.pair_id == pair_id)
        .collect::<Vec<_>>();
    settle_spouse_leisure_pair(ctx, second, first)?;
    let after = ctx
        .db
        .spouse_leisure_accrual()
        .pair_id()
        .find(&pair_id)
        .ok_or("Checkpoint missing")?;
    if before.conserved_joint_minutes != after.conserved_joint_minutes
        || before.next_trial_ordinal != after.next_trial_ordinal
        || ctx
            .db
            .conception_trial_receipt()
            .iter()
            .filter(|row| row.pair_id == pair_id)
            .count()
            != receipts.len()
        || receipts
            .iter()
            .map(|row| row.minute.get())
            .collect::<std::collections::BTreeSet<_>>()
            != [60, 120].into_iter().collect()
    {
        return Err("Repeated overlap settlement changed checkpoint or trial crossings".into());
    }
    Ok(())
}
