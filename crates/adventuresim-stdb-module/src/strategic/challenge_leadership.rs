fn require_errantry_party_leader(
    ctx: &ReducerContext,
    character_id: u64,
    leader_error: &'static str,
) -> Result<(crate::Character, String, Party), String> {
    let character = crate::character::require_living_character(ctx, character_id)?;
    let party_id = character.party_id.clone().ok_or("Must be in a party")?;
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(&party_id)
        .ok_or("Party not found")?;
    if party.leader_id != character_id {
        return Err(leader_error.into());
    }
    Ok((character, party_id, party))
}
