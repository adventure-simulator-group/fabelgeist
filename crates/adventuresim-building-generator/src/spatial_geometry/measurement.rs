//! Scalar set-out distances and overlap areas retain their units and admission.
use super::{CoordinateAxis, GeometryError, GeometryRole};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Finite signed metres, including zero offsets and negative insets.
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{SignedLength, PositiveLength};
/// fn wrong_role(inset: SignedLength) -> PositiveLength { inset }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct SignedLength(f32);
impl SignedLength {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres(value: f32) -> Result<Self, GeometryError> {
        if !value.is_finite() {
            return Err(GeometryError::NonFinite {
                role: GeometryRole::SignedLength,
                axis: CoordinateAxis::X,
            });
        }
        Ok(Self(value))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for SignedLength {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Finite nonnegative square metres. Empty overlap is a valid zero area.
/// ```compile_fail
/// use adventuresim_building_generator::spatial_geometry::{Area, SignedLength};
/// fn wrong_unit(area: Area) -> SignedLength { area }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct Area(f32);
impl Area {
    pub const ZERO: Self = Self(0.0);
    pub fn from_square_metres(value: f32) -> Result<Self, GeometryError> {
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
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_square_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
