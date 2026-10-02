//! Parameters for formed joint protection and its optional lower courses.
use crate::{DesignError, Millimeters, Permille, PlateFluting, PlateGauge};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

const JOINT_EDGE_MARGIN_M: f32 = 0.004;

/// A raised knee or elbow cop flowing into an integral lateral wing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JointCupConstruction {
    Wrapped,
    RaisedCop,
}

impl JointCupConstruction {
    /// The raised fan spreads independently of the enclosing cup. Wrapped
    /// plates couple this control to their angular coverage and remain narrower.
    pub const fn wing_range(self) -> RangeInclusive<u16> {
        match self {
            Self::Wrapped => 0..=650,
            Self::RaisedCop => 0..=2000,
        }
    }
}

/// Direction of existing plate fluting across the joint's carrier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JointFluteOrientation {
    Longitudinal,
    Transverse,
}

/// Parameters shared by wrapped joint plates and closed-outline raised cops.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointCupDesign {
    pub construction: JointCupConstruction,
    pub distal_extension: Option<crate::JointExtension>,
    /// Flare of the proximal rim over the adjacent upper-limb plate.
    pub proximal_flare: Millimeters,
    /// Rounds the free end of the fan while retaining its attachment to the cup.
    pub wing_roundness: Permille,
    pub wing_notch: Permille,
    pub fluting: Option<PlateFluting>,
    pub flute_orientation: JointFluteOrientation,
    /// Raised-cop return angles, each expressed as a fraction of a half-turn.
    pub medial_wrap: Permille,
    pub lateral_wrap: Permille,
    pub wing_height: Permille,
    /// Distal fan-lobe height relative to the proximal lobe.
    pub distal_wing_scale: Permille,
    pub center_ridge: Millimeters,

    pub gauge: PlateGauge,
    pub dome: Permille,
    pub wing: Permille,
    pub length: Permille,
}

impl Default for JointCupDesign {
    fn default() -> Self {
        Self::poleyn()
    }
}

impl JointCupDesign {
    pub const MEDIAL_WRAP_RANGE: RangeInclusive<u16> = 350..=750;
    pub const LATERAL_WRAP_RANGE: RangeInclusive<u16> = 500..=950;
    pub(crate) const DEFAULT_MEDIAL_WRAP: Permille = Permille(520);
    pub(crate) const DEFAULT_LATERAL_WRAP: Permille = Permille(850);

    pub fn poleyn() -> Self {
        Self {
            construction: JointCupConstruction::Wrapped,
            distal_extension: None,
            proximal_flare: Millimeters(6),
            wing_roundness: Permille(1000),
            fluting: None,
            flute_orientation: JointFluteOrientation::Longitudinal,
            medial_wrap: Self::DEFAULT_MEDIAL_WRAP,
            lateral_wrap: Self::DEFAULT_LATERAL_WRAP,
            wing_height: Permille(1000),
            distal_wing_scale: Permille(1000),
            wing_notch: Permille(0),
            center_ridge: Millimeters(0),
            gauge: PlateGauge::default(),
            dome: Permille(200),
            wing: Permille(350),
            length: Permille(1000),
        }
    }

    pub fn couter() -> Self {
        Self {
            construction: JointCupConstruction::Wrapped,
            distal_extension: None,
            proximal_flare: Millimeters(6),
            wing_roundness: Permille(500),
            fluting: None,
            flute_orientation: JointFluteOrientation::Longitudinal,
            medial_wrap: Self::DEFAULT_MEDIAL_WRAP,
            lateral_wrap: Self::DEFAULT_LATERAL_WRAP,
            wing_height: Permille(1000),
            distal_wing_scale: Permille(1000),
            wing_notch: Permille(0),
            center_ridge: Millimeters(0),
            gauge: PlateGauge::default(),
            dome: Permille(850),
            wing: Permille(450),
            length: Permille(900),
        }
    }
}

impl JointCupDesign {
    pub(crate) fn flute_coordinates(&self, u: f32, v: f32) -> [f32; 2] {
        if self.fluting.is_some() && self.flute_orientation == JointFluteOrientation::Transverse {
            [v, 1.0 - u]
        } else {
            [u, v]
        }
    }

    pub(crate) fn surface_clearance(&self) -> f32 {
        self.gauge.clearance.metres() + self.gauge.thickness.metres() + JOINT_EDGE_MARGIN_M
    }

    pub(crate) fn validate_construction(&self) -> Result<(), DesignError> {
        if !Self::MEDIAL_WRAP_RANGE.contains(&self.medial_wrap.0)
            || !Self::LATERAL_WRAP_RANGE.contains(&self.lateral_wrap.0)
        {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(extension) = &self.distal_extension {
            extension.validate()?;
            if self.construction == JointCupConstruction::RaisedCop {
                return Err(DesignError::ParametricParameters);
            }
        }
        Ok(())
    }
}
