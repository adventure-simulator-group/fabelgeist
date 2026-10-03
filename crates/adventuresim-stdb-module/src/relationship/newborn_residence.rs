//! Resolve optional newborn housing after the birth itself has committed.

use super::*;

pub(super) fn attach_newborn_residence(
    ctx: &ReducerContext,
    child_id: u64,
    parent_ids: [u64; 2],
    due_minute: StrategicMinute,
    settlement_id: &str,
) {
    // Housing is ancillary to an uncomplicated birth. Membership remains
    // authoritative even when no parental property has a remaining bed.
    let _ = crate::residence::inherit_parent_home_at(
        ctx,
        child_id,
        parent_ids,
        due_minute,
        settlement_id,
    );
}
