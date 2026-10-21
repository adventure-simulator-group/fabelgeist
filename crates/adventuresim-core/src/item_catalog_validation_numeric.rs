//! Numeric admission at the catalog's serialized representation boundary.

use serde_json::{Map, Value};

use super::CatalogDiagnostics;

pub(super) fn finite_in(
    object: &Map<String, Value>,
    field: &str,
    min: f64,
    max: f64,
    path: &str,
    errors: &mut CatalogDiagnostics<'_>,
) {
    if object
        .get(field)
        .and_then(Value::as_f64)
        .is_none_or(|value| !value.is_finite() || value < min || value > max)
    {
        errors.push(
            format!("{path}.{field}"),
            format!("expected finite value in {min}..={max}"),
        );
    }
}

pub(super) fn positive_f32_in(
    object: &Map<String, Value>,
    field: &str,
    max: f32,
    path: &str,
    errors: &mut CatalogDiagnostics<'_>,
) {
    if object
        .get(field)
        .and_then(Value::as_f64)
        .is_none_or(|value| {
            // Catalog JSON is parsed as f64 before deserializing these fields to
            // f32. Positivity must survive that conversion, including subnormals.
            let stored = value as f32;
            !stored.is_finite() || stored <= 0.0 || value > f64::from(max)
        })
    {
        errors.push(
            format!("{path}.{field}"),
            format!("expected finite positive f32 value at most {max}"),
        );
    }
}
