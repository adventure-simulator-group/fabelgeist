//! Admit authored physical dimensions independently of equipment destinations.

use super::{CatalogDiagnostics, Map, Value, reject_unknown};

pub(super) fn validate(
    equipment: &Map<String, Value>,
    path: &str,
    errors: &mut CatalogDiagnostics<'_>,
) {
    match equipment.get("physical").and_then(Value::as_object) {
        Some(physical) => {
            reject_unknown(
                physical,
                &["dimensions_m", "grip_to_tip_m", "anchor_offset_m"],
                &format!("{path}.physical"),
                errors,
            );
            match physical.get("dimensions_m").and_then(Value::as_array) {
                Some(values)
                    if values.len() == 3
                        && values.iter().all(|value| {
                            value
                                .as_f64()
                                .is_some_and(|value| value.is_finite() && value > 0.0)
                        }) => {}
                _ => errors.push(
                    format!("{path}.physical.dimensions_m"),
                    "expected three finite positive metres",
                ),
            }
            let grip_to_tip = physical
                .get("grip_to_tip_m")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            if !grip_to_tip.is_finite() || grip_to_tip < 0.0 {
                errors.push(
                    format!("{path}.physical.grip_to_tip_m"),
                    "expected a finite non-negative distance",
                );
            }
            if physical.get("anchor_offset_m").is_some_and(|offset| {
                offset.as_array().is_none_or(|values| {
                    values.len() != 3
                        || values
                            .iter()
                            .any(|value| value.as_f64().is_none_or(|value| !value.is_finite()))
                })
            }) {
                errors.push(
                    format!("{path}.physical.anchor_offset_m"),
                    "expected three finite metres",
                );
            }
        }
        None => errors.push(format!("{path}.physical"), "required object"),
    }
}
