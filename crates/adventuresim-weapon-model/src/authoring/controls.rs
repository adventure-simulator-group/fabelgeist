//! Validation of numeric editor descriptors at the JSON presentation boundary.
use super::*;
fn at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |value, key| match value {
        Value::Array(values) => values.get(key.parse::<usize>().ok()?),
        Value::Object(values) => values.get(key),
        _ => None,
    })
}
pub fn validate_controls(recipe: &Recipe, controls: &[Value]) -> Result<(), ControlError> {
    let value = serde_json::to_value(recipe).map_err(ControlError::Json)?;
    for control in controls {
        let number = if control["target"] == "shaft" {
            value["shaft"].get(control["key"].as_str().ok_or(ControlError::MissingKey)?)
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
                    .ok_or(ControlError::MissingPath)?,
            )
        }
        .and_then(Value::as_f64)
        .ok_or(ControlError::TargetNotNumeric)?;
        let minimum = control["min"]
            .as_f64()
            .ok_or(ControlError::MinimumNotNumeric)?;
        let maximum = control["max"]
            .as_f64()
            .ok_or(ControlError::MaximumNotNumeric)?;
        let step = control["step"]
            .as_f64()
            .ok_or(ControlError::StepNotNumeric)?;
        if number < minimum - 1e-9 || number > maximum + 1e-9 {
            return Err(ControlError::OutsideRange {
                label: control["label"].as_str().unwrap_or("control").into(),
                value: number,
                minimum,
                maximum,
            });
        }
        if step > 0.0 {
            let position = (number - minimum) / step;
            if (position - position.round()).abs() > 1e-6 {
                return Err(ControlError::StepMismatch);
            }
        }
        if let Some(paths) = control["paths"].as_array() {
            for path in paths {
                let linked = at_path(&value, path.as_str().ok_or(ControlError::LinkedPath)?)
                    .and_then(Value::as_f64)
                    .ok_or(ControlError::LinkedTargetNotNumeric)?;
                if (linked - number).abs() > 1e-8 {
                    return Err(ControlError::LinkedTargetsDiffer);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    #[error("control JSON: {0}")]
    Json(#[source] serde_json::Error),
    #[error("{label}: value {value} is outside control range {minimum}..={maximum}")]
    OutsideRange {
        label: String,
        value: f64,
        minimum: f64,
        maximum: f64,
    },
    #[error("control needs key")]
    MissingKey,
    #[error("control needs a path")]
    MissingPath,
    #[error("control target is not numeric")]
    TargetNotNumeric,
    #[error("control minimum is not numeric")]
    MinimumNotNumeric,
    #[error("control maximum is not numeric")]
    MaximumNotNumeric,
    #[error("control step is not numeric")]
    StepNotNumeric,
    #[error("value does not align with the control step")]
    StepMismatch,
    #[error("linked path is not a string")]
    LinkedPath,
    #[error("linked control target is not numeric")]
    LinkedTargetNotNumeric,
    #[error("linked control targets differ")]
    LinkedTargetsDiffer,
}
