//! Catalog bounds and anatomical chain validation.

use super::{CatalogDiagnostics, finite_in, reject_unknown};
use crate::item_catalog_schema::{EquipmentAnatomicalRegion as Region, SurfaceAnchor};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate_equipment_surface(
    value: Option<&Value>,
    placement_path: &str,
    item_kind: &str,
    has_material: bool,
    protected: &BTreeSet<&str>,
    errors: &mut CatalogDiagnostics<'_>,
) {
    let path = format!("{placement_path}.surface");
    let required = matches!(item_kind, "armor" | "clothing");
    let Some(spans) = value.and_then(Value::as_array) else {
        if required {
            errors.push(
                &path,
                "armor and clothing require non-empty anatomical surface spans",
            );
        }
        return;
    };
    if spans.is_empty() {
        if required {
            errors.push(
                &path,
                "armor and clothing require non-empty anatomical surface spans",
            );
        }
        return;
    }
    if !has_material {
        errors.push(
            &path,
            "anatomical surface spans require a procedural PBR material",
        );
    }
    let segment_count = spans
        .iter()
        .filter_map(|span| span.get("regions")?.as_array())
        .map(Vec::len)
        .sum::<usize>();
    if segment_count > crate::item_catalog_schema::MAX_EQUIPMENT_SURFACE_SEGMENTS {
        errors.push(&path, "too many anatomical surface segments");
    }
    for (index, span) in spans.iter().enumerate() {
        validate_equipment_surface_span(span, &format!("{path}.{index}"), protected, errors);
    }
}

fn validate_equipment_surface_span(
    span: &Value,
    span_path: &str,
    protected: &BTreeSet<&str>,
    errors: &mut CatalogDiagnostics<'_>,
) {
    let valid_regions = [
        "head",
        "neck",
        "chest",
        "stomach",
        "groin",
        "left_axilla",
        "right_axilla",
        "left_upper_arm",
        "left_forearm",
        "right_upper_arm",
        "right_forearm",
        "left_thigh",
        "left_lower_leg",
        "right_thigh",
        "right_lower_leg",
    ];
    let Some(span) = span.as_object() else {
        errors.push(span_path, "must be an object");
        return;
    };
    reject_unknown(span, &["regions", "anchor", "coverage"], span_path, errors);
    let regions = span
        .get("regions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if regions.is_empty()
        || regions.iter().any(|region| {
            !region
                .as_str()
                .is_some_and(|name| valid_regions.contains(&name))
        })
    {
        errors.push(
            format!("{span_path}.regions"),
            "expected a non-empty anatomical region chain",
        );
        return;
    }
    let region_names = regions.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    if region_names
        .windows(2)
        .any(|pair| !contiguous_regions(pair[0], pair[1]))
    {
        errors.push(
            format!("{span_path}.regions"),
            "regions must form a proximal-to-distal contiguous chain",
        );
    }
    for region in &region_names {
        let body_part = anatomical_body_part(region);
        if !protected.is_empty() && !protected.contains(body_part) {
            errors.push(
                format!("{span_path}.regions"),
                format!("{region:?} is outside placement protection {protected:?}"),
            );
        }
    }
    let anchor = span
        .get("anchor")
        .and_then(|value| serde_json::from_value::<SurfaceAnchor>(value.clone()).ok());
    if anchor.is_none() {
        errors.push(
            format!("{span_path}.anchor"),
            "expected proximal, distal, or center",
        );
    }
    finite_in(span, "coverage", f64::EPSILON, 1.0, span_path, errors);
}

fn anatomical_body_part(region: &str) -> &str {
    let Ok(region) = serde_json::from_value::<Region>(Value::String(region.into())) else {
        return "";
    };
    match region {
        Region::Head | Region::Neck => "head",
        Region::Chest | Region::LeftAxilla | Region::RightAxilla => "chest",
        Region::Stomach | Region::Groin => "stomach",
        Region::LeftUpperArm | Region::LeftForearm => "left_arm",
        Region::RightUpperArm | Region::RightForearm => "right_arm",
        Region::LeftThigh | Region::LeftLowerLeg => "left_leg",
        Region::RightThigh | Region::RightLowerLeg => "right_leg",
    }
}

fn contiguous_regions(proximal: &str, distal: &str) -> bool {
    matches!(
        (proximal, distal),
        ("stomach", "chest")
            | ("chest", "neck")
            | ("neck", "head")
            | ("left_upper_arm", "left_forearm")
            | ("right_upper_arm", "right_forearm")
            | ("left_thigh", "left_lower_leg")
            | ("right_thigh", "right_lower_leg")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn catalog_rejects_segment_overflow_before_combat_projection() {
        let span = json!({"regions":["chest"], "anchor":"center", "coverage":1.0});
        let maximum = crate::item_catalog_schema::MAX_EQUIPMENT_SURFACE_SEGMENTS;
        for (count, valid) in [(maximum, true), (maximum + 1, false)] {
            let mut diagnostics = Vec::new();
            let mut errors = CatalogDiagnostics {
                source_index: 0,
                diagnostics: &mut diagnostics,
            };
            validate_equipment_surface(
                Some(&json!(vec![span.clone(); count])),
                "test",
                "armor",
                true,
                &["chest"].into_iter().collect(),
                &mut errors,
            );
            assert_eq!(diagnostics.is_empty(), valid);
        }
    }
}
