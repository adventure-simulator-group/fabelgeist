//! Quest coverage fixture installation for an owned disposable simulation.

use super::*;
use crate::strategic::StrategicCombatResult;

/// Install deterministic quest coverage through private authority only. The
/// simulator must still discover, accept, travel, fight, and report through
/// the same public surfaces and ordinary reducers as a player.
#[reducer]
pub fn seed_simulation_quest_fixture(
    ctx: &ReducerContext,
    nonce: String,
    direct_leader_id: u64,
    generated_leader_id: u64,
) -> StrategicCombatResult<()> {
    let run = owned_run(ctx, &nonce)?;
    if direct_leader_id == generated_leader_id {
        return Err("Quest coverage requires two distinct party leaders".into());
    }
    for character_id in [direct_leader_id, generated_leader_id] {
        let simulation_character = ctx
            .db
            .simulation_character()
            .character_id()
            .find(character_id)
            .ok_or("Quest coverage leader is not owned by this simulation run")?;
        if simulation_character.run_id != run.id {
            return Err("Quest coverage leader belongs to another simulation run".into());
        }
        let character = ctx
            .db
            .character()
            .id()
            .find(character_id)
            .ok_or("Quest coverage leader does not exist")?;
        let party = character
            .party_id
            .as_deref()
            .and_then(|party_id| ctx.db.party_authority().id().find(party_id.to_owned()))
            .ok_or("Quest coverage leader is not in a party")?;
        if party.leader_id != character_id {
            return Err("Quest coverage character is not its party leader".into());
        }
    }
    if let Some(existing) = ctx.db.simulation_quest_fixture_authority().id().find(0) {
        return if existing.run_id == run.id
            && existing.direct_leader_id == direct_leader_id
            && existing.generated_leader_id == generated_leader_id
        {
            Ok(())
        } else {
            Err("Simulation quest fixture is already bound to different leaders".into())
        };
    }
    let party_power = |leader_id| -> Result<u64, String> {
        let party_id = ctx
            .db
            .character()
            .id()
            .find(leader_id)
            .and_then(|character| character.party_id)
            .ok_or("Quest coverage leader has no party")?;
        let party = ctx
            .db
            .party_authority()
            .id()
            .find(&party_id)
            .ok_or("Quest coverage party does not exist")?;
        crate::condition::refresh_character_strategic_condition(ctx, leader_id)?;
        for member_id in crate::strategic::living_party_member_ids(ctx, &party_id) {
            crate::capability::refresh_character_capability(ctx, member_id)?;
        }
        crate::strategic::publicly_ready_party_combat_power(ctx, &party).map(|(_, power)| power)
    };
    let direct_power = party_power(direct_leader_id)?;
    let fixture_enemy_power = crate::strategic::simulation_quest_fixture_enemy_power()?;
    if direct_power == 0
        || !adventuresim_core::autoresolve::combat_power_meets_safety_margin(
            direct_power,
            fixture_enemy_power,
        )
        .unwrap_or(false)
    {
        return Err("Quest coverage has no party safe for its ordinary fixture opponent".into());
    }
    let seeded = crate::strategic::seed_simulation_quest_fixture_inner(
        ctx,
        run.policy_seed,
        direct_leader_id,
        generated_leader_id,
    )?;
    ctx.db
        .simulation_quest_fixture_authority()
        .insert(SimulationQuestFixtureAuthority {
            id: 0,
            run_id: run.id,
            direct_contract_id: seeded.direct_contract_id,
            generated_canonical_case_id: seeded.generated_canonical_case_id,
            direct_leader_id,
            generated_leader_id,
            direct_party_id: seeded.direct_party_id,
            generated_party_id: seeded.generated_party_id,
        });
    Ok(())
}

