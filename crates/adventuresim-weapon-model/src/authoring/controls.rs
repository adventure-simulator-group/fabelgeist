//! Validation of numeric editor descriptors at the JSON presentation boundary.
use super::*;
fn at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |value, key| match value {
        Value::Array(values) => values.get(key.parse::<usize>().ok()?),
        Value::Object(values) => values.get(key),
        _ => None,
    })
}
pub fn validate_controls(recipe: &Recipe, controls: &[Value]) -> Result<(), String> {
    let value = serde_json::to_value(recipe).map_err(|e| e.to_string())?;
    for control in controls {
        let number = if control["target"] == "shaft" {
            value["shaft"].get(control["key"].as_str().ok_or("control needs key")?)
        } else if let Some(id) = control["componentId"].as_str() {
            value["components"]
                .as_array()
                .and_then(|parts| parts.iter().find(|p| p["id"] == id))
                .and_then(|p| p.get(control["key"].as_str()?))
        } else {
            at_path(
                &value,
                control["path"]
                    .as_str()
                    .or_else(|| control["paths"][0].as_str())
                    .ok_or("control needs a path")?,
            )
        }
        .and_then(Value::as_f64)
        .ok_or("control target is not numeric")?;
        let minimum = control["min"]
            .as_f64()
            .ok_or("control minimum is not numeric")?;
        let maximum = control["max"]
            .as_f64()
            .ok_or("control maximum is not numeric")?;
        let step = control["step"]
            .as_f64()
            .ok_or("control step is not numeric")?;
        if number < minimum - 1e-9 || number > maximum + 1e-9 {
            return Err(format!(
                "{}: value is outside the control range",
                control["label"].as_str().unwrap_or("control")
            ));
        }
        if step > 0.0 {
            let position = (number - minimum) / step;
            if (position - position.round()).abs() > 1e-6 {
                return Err("value does not align with the control step".into());
            }
        }
        if let Some(paths) = control["paths"].as_array() {
            for path in paths {
                let linked = at_path(&value, path.as_str().ok_or("linked path is not a string")?)
                    .and_then(Value::as_f64)
                    .ok_or("linked control target is not numeric")?;
                if (linked - number).abs() > 1e-8 {
                    return Err("linked control targets differ".into());
                }
            }
        }
    }
    Ok(())
}
