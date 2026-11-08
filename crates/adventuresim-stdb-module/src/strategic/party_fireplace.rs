fn require_party_fireplace_clear(ctx: &ReducerContext, party_id: &String) -> Result<(), String> {
    let fireplace_member_ids = ctx
        .db
        .party_member()
        .party_id()
        .filter(party_id)
        .map(|member| member.character_id)
        .collect::<Vec<_>>();
    crate::food::require_members_clear_current_camp_fireplace(ctx, party_id, &fireplace_member_ids)
}
