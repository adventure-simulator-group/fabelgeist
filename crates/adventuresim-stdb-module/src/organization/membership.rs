/// Membership chronology and role privileges.
fn current_minute(
    ctx: &ReducerContext,
    character_id: adventuresim_core::identity::CharacterId,
) -> Result<StrategicMinute, String> {
    ctx.db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map(|row| row.minutes)
        .ok_or_else(|| "Character time record not found".to_string())
}

pub fn membership_is_current(row: &OrganizationMembership, minute: StrategicMinute) -> bool {
    row.status == OrganizationMembershipStatus::Active && minute <= row.dues_paid_through_minute
}

fn current_membership_grants(
    definition: &OrganizationDefinition,
    membership: &OrganizationMembership,
    role: &adventuresim_core::organization::OrganizationRoleDefinition,
    minute: StrategicMinute,
    privilege: Privilege,
) -> bool {
    membership_is_current(membership, minute)
        && definition.has_privilege_at_role(&role.id, privilege)
}
