//! Engineering policy carries checked units independently of terrain samples.
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Finite positive rise/run bound, independent of metre elevations.
/// ```compile_fail
/// use adventuresim_tactical_core::city_layout::grounding::SupportGrade;
/// use adventuresim_building_generator::spatial_geometry::PositiveLength;
/// fn wrong_unit(length: PositiveLength) -> SupportGrade { length }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct SupportGrade(f32);
impl SupportGrade {
    pub const fn from_ratio(ratio: f32) -> Option<Self> {
        if ratio.is_finite() && ratio > 0.0 {
            Some(Self(ratio))
        } else {
            None
        }
    }
    pub fn ratio(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for SupportGrade {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_ratio(f32::deserialize(d)?)
            .ok_or_else(|| serde::de::Error::custom("support grade must be finite and positive"))
    }
}

/// Construction bounds supplied by generation/capture, not actor movement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(opaque)]
pub struct SupportLimits {
    pub(super) maximum_grade: SupportGrade,
    pub(super) maximum_displacement_metres: PositiveLength,
    pub(super) contact_tolerance_metres: PositiveLength,
}
impl SupportLimits {
    pub const fn new(
        maximum_grade: SupportGrade,
        maximum_displacement: PositiveLength,
        contact_tolerance: PositiveLength,
    ) -> Self {
        Self {
            maximum_grade,
            maximum_displacement_metres: maximum_displacement,
            contact_tolerance_metres: contact_tolerance,
        }
    }
    pub fn contact_tolerance_metres(self) -> f32 {
        self.contact_tolerance_metres.metres()
    }
}

impl core::fmt::Display for SupportGrade {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.ratio().fmt(f)
    }
}
