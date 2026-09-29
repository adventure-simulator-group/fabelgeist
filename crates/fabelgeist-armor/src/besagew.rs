//! Circular armpit defenses suspended independently in front of the shoulder.
use crate::{DesignError, Millimeters, Milliradians, RadialFluting};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BesagewDesign {
    pub radius: Millimeters,
    pub boss_height: Millimeters,
    pub fluting: Option<RadialFluting>,
    pub shoulder_drop: Millimeters,
    pub medial_offset: Millimeters,
    pub plate_clearance: Millimeters,
    pub outward_tilt: Milliradians,
}

impl Default for BesagewDesign {
    fn default() -> Self {
        Self {
            radius: Millimeters(58),
            boss_height: Millimeters(14),
            fluting: None,
            shoulder_drop: Millimeters(65),
            medial_offset: Millimeters(30),
            plate_clearance: Millimeters(5),
            outward_tilt: Milliradians(100),
        }
    }
}

impl BesagewDesign {
    pub fn validate(&self) -> Result<(), DesignError> {
        if !(35..=85).contains(&self.radius.0)
            || self.boss_height.0 > 30
            || !(35..=120).contains(&self.shoulder_drop.0)
            || self.medial_offset.0 > 70
            || !(3..=15).contains(&self.plate_clearance.0)
            || self.outward_tilt.0 > 350
        {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(fluting) = &self.fluting {
            fluting.validate()?;
        }
        Ok(())
    }
}
