//! Positive scalar measurements are independent of coordinate frame and datum.
use super::{CoordinateAxis, GeometryError, GeometryRole};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// A finite positive length in metres, for dimensions and working clearances.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct PositiveLength(f32);

impl PositiveLength {
    pub const fn from_metres(value: f32) -> Result<Self, GeometryError> {
        let role = GeometryRole::PositiveLength;
        let axis = CoordinateAxis::X;
        if !value.is_finite() {
            return Err(GeometryError::NonFinite { role, axis });
        }
        if value < 0.0 {
            return Err(GeometryError::NegativeExtent { role, axis });
        }
        if value == 0.0 {
            return Err(GeometryError::ZeroExtent { role, axis });
        }
        Ok(Self(value))
    }
    pub const fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for PositiveLength {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
