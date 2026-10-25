use serde::{Deserialize, Serialize};

/// Game-scale contact and handling calibration authored in tactical combat YAML.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponContactParameters {
    pub reference_control_length_metres: f32,
    pub point_reference_area_mm2: f32,
    pub edge_reference_area_mm2: f32,
    pub broad_reference_area_mm2: f32,
    pub unarmed_precision: f32,
}

impl WeaponContactParameters {
    pub fn validate(self) -> Result<(), &'static str> {
        if ![
            self.reference_control_length_metres,
            self.point_reference_area_mm2,
            self.edge_reference_area_mm2,
            self.broad_reference_area_mm2,
        ]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
        {
            return Err("weapon contact calibration must be finite and positive");
        }
        if !self.unarmed_precision.is_finite() || self.unarmed_precision < 0.0 {
            return Err("unarmed precision must be finite and non-negative");
        }
        Ok(())
    }
}
