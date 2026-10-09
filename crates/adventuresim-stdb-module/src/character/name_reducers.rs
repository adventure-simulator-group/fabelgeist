// Authored renames at the reducer boundary of the semantic name authority.

#[reducer]
pub fn update_character(ctx: &ReducerContext, id: u64, name: String) -> Result<(), String> {
    require_living_character(ctx, id)?;
    assign_authored_character_name(ctx, CharacterId::new(id), name)
        .map_err(|error| error.to_string())
}
