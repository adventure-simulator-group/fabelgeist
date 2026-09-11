use super::{CatalogDiagnostics, reject_unknown};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate_parents(
    parents: &[Value],
    placement_path: &str,
    errors: &mut CatalogDiagnostics<'_>,
) {
    for (index, parent) in parents.iter().enumerate() {
        if let Err(error) =
            serde_json::from_value::<crate::item_catalog_schema::ParentRequirement>(parent.clone())
        {
            errors.push(
                format!("{placement_path}.parents.{index}"),
                error.to_string(),
            );
        }
    }
}

pub(super) fn validate_occupancy(
    occupancy: &[Value],
    placement_path: &str,
    valid_locations: &BTreeSet<&str>,
    valid_channels: &BTreeSet<&str>,
    errors: &mut CatalogDiagnostics<'_>,
) {
    let mut occupied: Vec<crate::item_catalog_schema::OccupancyRequirement> = Vec::new();
    for (occupancy_index, requirement) in occupancy.iter().enumerate() {
        let Some(requirement) = requirement.as_object() else {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}"),
                "must be an object",
            );
            continue;
        };
        reject_unknown(
            requirement,
            &["location", "channel", "order", "fit_zone"],
            &format!("{placement_path}.occupancy.{occupancy_index}"),
            errors,
        );
        let location = requirement
            .get("location")
            .and_then(Value::as_str)
            .unwrap_or("");
        let channel = requirement
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !valid_locations.contains(location) {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}.location"),
                "invalid location",
            );
        }
        if !valid_channels.contains(channel) {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}.channel"),
                "invalid channel",
            );
        }
        let typed = serde_json::from_value::<crate::item_catalog_schema::OccupancyRequirement>(
            Value::Object(requirement.clone()),
        )
        .ok();
        if typed.is_some_and(|typed| {
            typed
                .fit_zone
                .is_some_and(|zone| !zone.fits_location(typed.location))
        }) {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}.fit_zone"),
                "fit zone does not belong to the occupied location",
            );
        }
        let order = requirement
            .get("order")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if matches!(
            channel,
            "held" | "base_clothing" | "padding" | "flexible_armor" | "rigid_armor" | "outerwear"
        ) && order != 0
        {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}.order"),
                "singleton channel requires order 0",
            );
        }
        if order > u16::MAX.into()
            || typed.is_some_and(|requirement| {
                let conflict = occupied
                    .iter()
                    .any(|other| requirement.conflicts_with(*other));
                occupied.push(requirement);
                conflict
            })
        {
            errors.push(
                format!("{placement_path}.occupancy.{occupancy_index}"),
                "duplicate or invalid ordered occupancy",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn diagnostics(requirements: Value) -> Vec<super::super::CatalogDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut errors = CatalogDiagnostics {
            source_index: 0,
            diagnostics: &mut diagnostics,
        };
        validate_occupancy(
            requirements.as_array().unwrap(),
            "test",
            &["left_arm"].into_iter().collect(),
            &["rigid_armor"].into_iter().collect(),
            &mut errors,
        );
        diagnostics
    }

    #[test]
    fn zones_must_belong_to_the_anatomical_location() {
        assert!(
            !diagnostics(
                json!([{"location":"left_arm", "channel":"rigid_armor", "fit_zone":"shin"}])
            )
            .is_empty()
        );
    }

    #[test]
    fn one_piece_can_reserve_multiple_disjoint_zones_but_not_duplicate_or_whole_limb_zones() {
        let forearm = json!({"location":"left_arm", "channel":"rigid_armor", "fit_zone":"forearm"});
        let elbow = json!({"location":"left_arm", "channel":"rigid_armor", "fit_zone":"elbow"});
        let whole_arm = json!({"location":"left_arm", "channel":"rigid_armor"});
        assert!(diagnostics(json!([forearm, elbow])).is_empty());
        assert!(!diagnostics(json!([forearm, forearm])).is_empty());
        assert!(!diagnostics(json!([forearm, whole_arm])).is_empty());
    }
}
