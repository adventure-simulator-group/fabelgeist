//! Scalar set-out distances and overlap areas retain their units and admission.

#[cfg(feature = "reflect")]
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::{CoordinateAxis, GeometryError, GeometryResult as Result, GeometryRole, PositiveLength};

/// Finite signed metres, including zero offsets and negative insets.
/// ```compile_fail
/// use fabelgeist_geometry::{SignedLength, PositiveLength};
/// fn wrong_role(inset: SignedLength) -> PositiveLength { inset }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "reflect", reflect(opaque))]
pub struct SignedLength(f32);

/// Finite nonnegative square metres. Empty overlap is a valid zero area.
/// ```compile_fail
/// use fabelgeist_geometry::{Area, SignedLength};
/// fn wrong_unit(area: Area) -> SignedLength { area }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(feature = "reflect", reflect(opaque))]
pub struct Area(f32);

impl SignedLength {
    pub const ZERO: Self = Self(0.0);
    pub const fn from_metres(value: f32) -> Result<Self> {
        if !value.is_finite() {
            return Err(GeometryError::NonFinite {
                role: GeometryRole::SignedLength,
                axis: CoordinateAxis::X,
            });
        }
        Ok(Self(value))
    }
    /// Native minimum of two finite admitted lengths, including signed zero.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Native maximum of two finite admitted lengths, including signed zero.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// A bounded millimetre observation is always finite. Preserve the native
    /// armor conversion's f32 promotion followed by division, without rounding
    /// through an intermediate representation or changing it to multiplication.
    pub fn from_millimetres(value: u16) -> Self {
        Self(f32::from(value) / 1_000.0)
    }

    /// Scale a signed offset at its numerical kernel and retain finite admission.
    pub fn scaled_by(self, factor: f32) -> Result<Self> {
        Self::from_metres(self.0 * factor)
    }

    pub fn metres(self) -> f32 {
        self.0
    }
}
impl From<PositiveLength> for SignedLength {
    fn from(value: PositiveLength) -> Self {
        Self(value.metres())
    }
}
impl<'de> Deserialize<'de> for SignedLength {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

impl Area {
    pub const ZERO: Self = Self(0.0);
    pub fn from_square_metres(value: f32) -> Result<Self> {
        let role = GeometryRole::Area;
        let axis = CoordinateAxis::X;
        if !value.is_finite() {
            return Err(GeometryError::NonFinite { role, axis });
        }
        if value < 0.0 {
            return Err(GeometryError::NegativeExtent { role, axis });
        }
        Ok(Self(value))
    }
    pub fn square_metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for Area {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        Self::from_square_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
