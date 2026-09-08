//! Bounds at the preset boundary keep editable coefficients finite and sampling work bounded.
use super::TextureParameters;
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub struct ParameterError(pub String);
impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for ParameterError {}

/// A numerical control's legal interval. Both the inspector and preset validator use it.
#[derive(Clone, Copy)]
pub struct ControlBounds {
    pub min: f64,
    pub max: f64,
    pub integer: bool,
}
impl ControlBounds {
    pub fn for_default(path: &str, default: &serde_json::Number) -> Self {
        let value = default.as_f64().unwrap_or_default();
        let name = path
            .split('/')
            .rev()
            .find(|part| part.parse::<usize>().is_err())
            .unwrap_or(path);
        let component = path
            .rsplit('/')
            .next()
            .and_then(|part| part.parse::<usize>().ok());
        let integer =
            default.is_u64() || default.is_i64() || matches!(name, "ring_count" | "fiber_count");
        let color = path.contains("/colors/")
            || path.contains("/leaf_colors/")
            || path.contains("srgb")
            || path.ends_with("transmitted_color")
            || path.contains("/transmitted_color/")
            || path.contains("palette")
            || name.ends_with("roughness");
        let (min, max) = match name {
            "bubbles" | "patches" if component.is_some_and(|index| index < 2) => (0.0, 1.0),
            "bubbles" | "patches" if component.is_some_and(|index| index < 4) => (0.001, 0.45),
            "patches" if component == Some(4) => (0.1, 12.0),
            "patches" if component == Some(5) => (0.0, 0.1),
            _ if color && integer => (0.0, 255.0),
            _ if color && !integer && value <= 1.0 => (0.0, 1.0),
            "seed" | "salt" => (0.0, u64::MAX as f64),
            "index_of_refraction" => (1.0, 3.0),
            "courses" | "rows"
                if path.starts_with("/dressed_stone/") || path.starts_with("/rubble_masonry/") =>
            {
                (1.0, 22.0)
            }
            "min_blocks_per_course" | "max_blocks_per_course" => (1.0, 16.0),
            "min_stones_per_row" | "max_stones_per_row" => (1.0, 22.0),
            "knot_taper" => (0.0, 0.8),
            "knot_flow_strength" => (0.0, 2.0),
            "knot_influence_radii" => (1.0, 5.0),
            "center" if path.contains("/knots/") => (0.0, 1.0),
            "radii" if path.contains("/knots/") => (0.001, 0.08),
            "lean" if path.contains("/knots/") => (-0.5, 0.5),
            _ if name.contains("fraction")
                || name.contains("probability")
                || name.contains("density") =>
            {
                (0.0, 1.0_f64.max(value))
            }
            _ if name.contains("roughness") && value <= 1.0 => (0.0, 1.0),
            _ if integer && value >= 0.0 => (
                if value == 0.0 { 0.0 } else { 1.0 },
                (value * 4.0).clamp(16.0, 2048.0),
            ),
            _ if value < 0.0 => (value * 4.0, -value * 4.0),
            _ if value == 0.0 => (-1.0, 1.0),
            _ => ((value * 0.05).min(0.000_001), value * 4.0),
        };
        Self { min, max, integer }
    }
}

impl TextureParameters {
    /// Validate the external shape before deserialization can discard an unknown nested field.
    pub fn from_value(value: Value) -> Result<Self, ParameterError> {
        let defaults =
            serde_json::to_value(Self::default()).expect("canonical parameters serialize");
        validate_node("", &value, &defaults)?;
        let parameters: Self =
            serde_json::from_value(value).map_err(|e| ParameterError(e.to_string()))?;
        parameters.validate()?;
        Ok(parameters)
    }
    pub fn validate(&self) -> Result<(), ParameterError> {
        let value = serde_json::to_value(self).map_err(|e| ParameterError(e.to_string()))?;
        let defaults =
            serde_json::to_value(Self::default()).expect("canonical parameters serialize");
        validate_node("", &value, &defaults)?;
        let stone = &self.dressed_stone;
        if stone.min_blocks_per_course > stone.max_blocks_per_course {
            return Err(ParameterError("Minimum block count exceeds maximum".into()));
        }
        let rubble = &self.rubble_masonry;
        if rubble.min_stones_per_row > rubble.max_stones_per_row {
            return Err(ParameterError("Minimum stone count exceeds maximum".into()));
        }
        let grain = &self.hewn_oak_grain;
        if grain.latewood_width[0] > grain.latewood_width[1]
            || grain.latewood_width[1] * (1.0 + grain.latewood_shoulder_ratio) >= 1.0
        {
            return Err(ParameterError(
                "Latewood widths must be ordered and fit inside each growth band".into(),
            ));
        }
        for knot in &grain.knots {
            let along = knot.radii[1] * grain.knot_influence_radii;
            let across = knot.radii[0] * (1.0 + grain.knot_taper) * grain.knot_influence_radii
                + 2.0 * knot.lean.abs() * along;
            if along >= 0.5 || across >= 0.5 {
                return Err(ParameterError(
                    "Knot influence must fit within half a tile to preserve seamless flow".into(),
                ));
            }
        }
        if self.window_glass.minimum_roughness > self.window_glass.maximum_roughness {
            return Err(ParameterError(
                "Minimum glass roughness exceeds maximum".into(),
            ));
        }
        Ok(())
    }
}

fn validate_node(path: &str, value: &Value, default: &Value) -> Result<(), ParameterError> {
    match (value, default) {
        (Value::Object(values), Value::Object(defaults)) => {
            if values.len() != defaults.len() {
                return Err(ParameterError(format!("Unexpected fields at {path}")));
            }
            for (key, default) in defaults {
                validate_node(
                    &format!("{path}/{key}"),
                    values
                        .get(key)
                        .ok_or_else(|| ParameterError(format!("Missing {path}/{key}")))?,
                    default,
                )?;
            }
        }
        (Value::Array(values), Value::Array(defaults)) => {
            let knots = matches!(
                path,
                "/hewn_oak_grain/knots" | "/window_glass/bubbles" | "/window_glass/patches"
            );
            if (knots && values.len() > 16) || (!knots && values.len() != defaults.len()) {
                return Err(ParameterError(format!("Unsupported list length at {path}")));
            }
            for (index, value) in values.iter().enumerate() {
                let default = if knots {
                    &defaults[0]
                } else {
                    &defaults[index]
                };
                validate_node(&format!("{path}/{index}"), value, default)?;
            }
        }
        (Value::Number(value), Value::Number(default)) => {
            if path == "/seed" || path.ends_with("/salt") {
                return Ok(());
            }
            let bounds = ControlBounds::for_default(path, default);
            let number = value.as_f64().unwrap_or(f64::NAN);
            // JSON's short float spelling must validate in the recipe's actual f32 precision.
            let number = if default.is_f64() {
                f64::from(number as f32)
            } else {
                number
            };
            let min = if default.is_f64() {
                f64::from(bounds.min as f32)
            } else {
                bounds.min
            };
            let max = if default.is_f64() {
                f64::from(bounds.max as f32)
            } else {
                bounds.max
            };
            if !number.is_finite()
                || !(min..=max).contains(&number)
                || (bounds.integer && number.fract() != 0.0)
            {
                return Err(ParameterError(format!(
                    "{path} must be {}..{}{}",
                    bounds.min,
                    bounds.max,
                    if bounds.integer {
                        " (whole numbers)"
                    } else {
                        ""
                    }
                )));
            }
        }
        (Value::Bool(_), Value::Bool(_)) | (Value::String(_), Value::String(_)) => {}
        _ => return Err(ParameterError(format!("Invalid value at {path}"))),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_defaults_and_rejects_unsafe_shape_ranges() {
        let mut params = TextureParameters::default();
        params.validate().unwrap();
        params.hewn_oak_grain.knots.clear();
        params.validate().unwrap();
        params.dressed_stone.min_blocks_per_course = 17;
        assert!(params.validate().is_err());
        params.dressed_stone.min_blocks_per_course = 1;
        params.hewn_oak_grain.ring_count = 2.5;
        assert!(params.validate().is_err());
    }
}
