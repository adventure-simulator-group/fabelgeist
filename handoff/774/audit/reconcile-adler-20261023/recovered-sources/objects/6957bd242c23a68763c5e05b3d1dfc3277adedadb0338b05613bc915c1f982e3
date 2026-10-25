/// Present NPC policy candidates at one actor-relative strategic instant.
fn present_romance_candidates(
    ctx: &ReducerContext,
    settlement_id: &str,
    actor_id: u64,
    minute: adventuresim_world_schema::calendar::StrategicMinute,
) -> Vec<adventuresim_core::npc_policy::NpcCandidate> {
    let candidates = ctx
        .db
        .settlement_resident_presence()
        .settlement_id()
        .filter(settlement_id)
        .filter(|presence| {
            presence.character_id != actor_id
                && crate::settlement_population::npc_is_present(ctx, presence, minute)
        })
        .filter_map(|presence| {
            ctx.db
                .npc_policy()
                .character_id()
                .find(presence.character_id)
                .map(|policy| adventuresim_core::npc_policy::NpcCandidate {
                    character_id: presence.character_id,
                    policy_seed: policy.policy_seed,
                })
        });
    candidates.collect()
}
