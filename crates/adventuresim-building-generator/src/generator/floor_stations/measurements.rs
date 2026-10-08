use crate::spatial_geometry::{CoordinateAxis, GeometryError, GeometryRole};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Positive architectural width in metres before the bearing insets.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub(in crate::generator) struct FloorWidth(f32);
impl FloorWidth {
    pub fn from_metres(value: f32) -> Result<Self, GeometryError> {
        scalar(value, GeometryRole::FloorWidth)?;
        positive(value, GeometryRole::FloorWidth)?;
        Ok(Self(value))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for FloorWidth {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
/// Positive distance between the inset architectural bearings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub(in crate::generator) struct FloorSpan(f32);
impl FloorSpan {
    pub fn from_metres(value: f32) -> Result<Self, GeometryError> {
        scalar(value, GeometryRole::FloorSpan)?;
        positive(value, GeometryRole::FloorSpan)?;
        Ok(Self(value))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for FloorSpan {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
/// Signed architectural X station in metres, independent of width or span.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub(in crate::generator) struct FloorStation(f32);
impl FloorStation {
    pub fn from_metres(value: f32) -> Result<Self, GeometryError> {
        scalar(value, GeometryRole::FloorStation)?;
        Ok(Self(value))
    }
    pub fn metres(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for FloorStation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
fn scalar(value: f32, role: GeometryRole) -> Result<(), GeometryError> {
    if !value.is_finite() {
        return Err(GeometryError::NonFinite {
            role,
            axis: CoordinateAxis::X,
        });
    }
    Ok(())
}
fn positive(value: f32, role: GeometryRole) -> Result<(), GeometryError> {
    if value < 0.0 {
        return Err(GeometryError::NegativeExtent {
            role,
            axis: CoordinateAxis::X,
        });
    }
    if value == 0.0 {
        return Err(GeometryError::ZeroExtent {
            role,
            axis: CoordinateAxis::X,
        });
    }
    Ok(())
}
