//! Validate a candidate container stack's physical-object backing.

use super::{
    CarriedInventoryRowId, OperationalCustody, ReducerContext, object_for_row, object_is_nested,
    object_is_nonempty, require_exact_carried_backing,
};

fn empty_container_can_stack(has_contents: bool, is_nested: bool, has_condition: bool) -> bool {
    !has_contents && !is_nested && !has_condition
}

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

#[cfg(test)]
mod tests {
    use super::empty_container_can_stack;

    #[test]
    fn only_empty_root_containers_without_instance_condition_stack() {
        assert!(empty_container_can_stack(false, false, false));
        assert!(!empty_container_can_stack(true, false, false));
        assert!(!empty_container_can_stack(false, true, false));
        assert!(!empty_container_can_stack(false, false, true));
    }
}
