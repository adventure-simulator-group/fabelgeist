//! Check authored condition values and participant role references.

use super::{Condition, DialogueError, Role};
use std::collections::BTreeMap;

pub(super) fn validate_condition_roles(
    condition: &Condition,
    roles: &BTreeMap<String, Role>,
    errors: &mut Vec<DialogueError>,
) {
    match condition {
        Condition::All { conditions } | Condition::Any { conditions } => {
            for child in conditions {
                validate_condition_roles(child, roles, errors);
            }
        }
        Condition::Not { condition } => validate_condition_roles(condition, roles, errors),
        Condition::Fact { key, equals } => {
            if !key.authoring_value_is_valid(equals) {
                errors.push(DialogueError::InvalidFactValue(key.clone()));
            }
            for role in key.participant_roles() {
                if !roles.contains_key(role) {
                    errors.push(DialogueError::UnknownRole(role.into()));
                }
            }
        }
        Condition::Always => {}
    }
}
