//! Persist the one-time evaluation of an evening's alcohol consumption.

use super::{AlcoholConsumption, alcohol_consumption};
use spacetimedb::{ReducerContext, Table};

pub(super) fn mark_evening_evaluated(
    ctx: &ReducerContext,
    character_id: u64,
    evening: u64,
    id: &String,
    consumed: u32,
) {
    let mut row = ctx
        .db
        .alcohol_consumption()
        .id()
        .find(id)
        .unwrap_or(AlcoholConsumption {
            id: id.clone(),
            character_id,
            evening_id: evening,
            ethanol_ml: 0,
            morale_evaluated: false,
        });
    row.ethanol_ml = consumed;
    row.morale_evaluated = true;
    if ctx.db.alcohol_consumption().id().find(id).is_some() {
        ctx.db.alcohol_consumption().id().update(row);
    } else {
        ctx.db.alcohol_consumption().insert(row);
    }
}
