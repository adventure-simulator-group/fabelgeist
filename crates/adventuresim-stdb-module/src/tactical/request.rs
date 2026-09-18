use spacetimedb::{ReducerContext, SpacetimeType};

use crate::{
    character::character,
    settlement_population::{settlement_business_operator, settlement_resident_profile},
    strategic::settlement,
};

#[derive(Clone, Debug, PartialEq, Eq, SpacetimeType)]
pub struct TacticalBusinessOperator {
    pub business_id: adventuresim_world_schema::settlement_buildings::BusinessId,
    pub operator_character_id: u64,
    pub operator_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, SpacetimeType)]
pub struct TacticalSettlementSnapshot {
    pub id: String,
    pub population_level: i32,
    pub population_estimate: u32,
    pub economy: adventuresim_world_schema::SettlementEconomyProfile,
    pub operators: Vec<TacticalBusinessOperator>,
}

pub(crate) fn tactical_settlement_snapshot(
    ctx: &ReducerContext,
    origin_settlement_id: &str,
    scene_key: &str,
) -> Result<Option<TacticalSettlementSnapshot>, String> {
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(origin_settlement_id.to_owned());
    let Some(settlement) = settlement else {
        return Ok(None);
    };
    if settlement.scene_key != scene_key {
        return Ok(None);
    }
    crate::settlement_population::ensure_settlement_population(ctx, origin_settlement_id)?;
    let mut operators = ctx
        .db
        .settlement_business_operator()
        .settlement_id()
        .filter(&origin_settlement_id.to_owned())
        .map(|assignment| {
            if assignment.business_id.settlement_id != origin_settlement_id {
                return Err("Business operator crosses the tactical settlement boundary".into());
            }
            ctx.db
                .settlement_resident_profile()
                .character_id()
                .find(assignment.operator_character_id)
                .filter(|resident| resident.home_settlement_id == origin_settlement_id)
                .ok_or("Business operator is not a resident of its settlement")?;
            let operator_name = ctx
                .db
                .character()
                .id()
                .find(assignment.operator_character_id)
                .ok_or("Business operator is missing its Character")?
                .name;
            Ok(TacticalBusinessOperator {
                business_id: assignment.business_id,
                operator_character_id: assignment.operator_character_id,
                operator_name,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    operators.sort_by(|left, right| left.business_id.cmp(&right.business_id));
    Ok(Some(TacticalSettlementSnapshot {
        id: settlement.id,
        population_level: settlement.population_level,
        population_estimate: settlement.population_estimate,
        economy: settlement.economy,
        operators,
    }))
}

pub(crate) fn tactical_party_roster(
    ctx: &ReducerContext,
    party_id: &str,
) -> Result<(Vec<u64>, u32), String> {
    let members = crate::strategic::living_party_member_ids(ctx, party_id);
    let count =
        u32::try_from(members.len()).map_err(|_| "Party is too large for tactical enrollment")?;
    if count == 0 {
        return Err("A tactical mission requires at least one living party member".into());
    }
    if count as usize > adventuresim_core::mission::MAX_TACTICAL_RECEIPT_PARTICIPANTS {
        return Err("Party exceeds the tactical receipt participant limit".into());
    }
    Ok((members, count))
}
