//! Validate a candidate container stack's physical-object backing.

use super::{
    CarriedInventoryRowId, OperationalCustody, ReducerContext, empty_container_can_stack,
    object_for_row, object_is_nested, object_is_nonempty, require_exact_carried_backing,
};

pub(super) fn stackable_empty_root(ctx: &ReducerContext, object_id: u64) -> bool {
    empty_container_can_stack(
        object_is_nonempty(ctx, object_id),
        object_is_nested(ctx, object_id),
        false,
    )
}

pub(super) fn mergeable_object(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
    expected: &OperationalCustody,
) -> Result<bool, String> {
    match object_for_row(ctx, row_id)? {
        Some(object) => {
            require_exact_carried_backing(ctx, &object, expected)?;
            Ok(!object_is_nonempty(ctx, object.id) && !object_is_nested(ctx, object.id))
        }
        None => Ok(true),
    }
}
