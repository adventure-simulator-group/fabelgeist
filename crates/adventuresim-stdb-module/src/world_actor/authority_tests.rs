// Guarded persistence acceptance for interval-owned context membership.

#[reducer]
pub fn authority_test_context_intervals(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let context_id = "authority-test-context";
    let id = "authority-test-membership";
    let entered = StrategicMinute::new(10);
    let left = StrategicMinute::new(20);
    let row = CharacterContextMembership {
        id: id.into(),
        context_id: context_id.into(),
        location_id: context_id.into(),
        character_id: 732010,
        context_kind: CharacterContextKind::StrategicEncounter,
        role: CharacterContextRole::Bystander,
        ordinal: 0,
        entered_at: entered,
        left_at: None,
        revision: 0,
        contact_decision: ContextualDecisionState::Allowed,
        treatment_decision: ContextualDecisionState::Unavailable,
    };
    ctx.db.character_context_membership().insert(row.clone());
    if context_members(ctx, context_id).len() != 1
        || context_membership_valid_at(&row, StrategicMinute::new(9))
        || !context_membership_valid_at(&row, entered)
    {
        return Err("Open membership projection disagrees with its interval".into());
    }
    deactivate_context_roster_at(ctx, context_id, left);
    let closed = ctx.db.character_context_membership().id().find(id.to_owned()).ok_or("Membership missing")?;
    if closed.is_open()
        || !context_members(ctx, context_id).is_empty()
        || !context_membership_valid_at(&closed, entered)
        || context_membership_valid_at(&closed, left)
    {
        return Err("Closed membership lost historical interval semantics".into());
    }
    close_context_membership_at(ctx, closed.clone(), StrategicMinute::new(30));
    let after = ctx.db.character_context_membership().id().find(id.to_owned()).ok_or("Membership missing")?;
    if after.left_at != Some(left) || after.revision != closed.revision {
        return Err("Repeated closure rewrote membership history".into());
    }
    Ok(())
}
