use super::{CoordinateAxis, GeometryError, GeometryRole, error::finite};
use bevy::{
    math::{Vec2, Vec3},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

fn dimensions(value: Vec3, role: GeometryRole, positive: bool) -> Result<(), GeometryError> {
    finite(value, role)?;
    for (axis, component) in [CoordinateAxis::X, CoordinateAxis::Y, CoordinateAxis::Z]
        .into_iter()
        .zip(value.to_array())
    {
        if component < 0.0 {
            return Err(GeometryError::NegativeExtent { role, axis });
        }
        if positive && component == 0.0 {
            return Err(GeometryError::ZeroExtent { role, axis });
        }
    }
    Ok(())
}

/// Cuboid-axis extents in metres. Degenerate solids can contribute point or line contact.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct CuboidDimensions(Vec3);
impl CuboidDimensions {
    pub const ZERO: Self = Self(Vec3::ZERO);
    pub fn from_metres(value: Vec3) -> Result<Self, GeometryError> {
        dimensions(value, GeometryRole::CuboidDimensions, false)?;
        Ok(Self(value))
    }
    pub fn metres(self) -> Vec3 {
        self.0
    }
}
impl<'de> Deserialize<'de> for CuboidDimensions {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec3::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Positive operable-leaf width, height and thickness in leaf-local axes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct LeafDimensions(Vec3);
impl LeafDimensions {
    pub fn from_metres(value: Vec3) -> Result<Self, GeometryError> {
        dimensions(value, GeometryRole::LeafDimensions, true)?;
        Ok(Self(value))
    }
    pub fn metres(self) -> Vec3 {
        self.0
    }
    pub fn cuboid(self) -> CuboidDimensions {
        CuboidDimensions(self.0)
    }
}
impl<'de> Deserialize<'de> for LeafDimensions {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec3::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Positive architectural X/Z width and depth, in metres.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct PlanDimensions(Vec2);
impl PlanDimensions {
    pub fn from_metres(value: Vec2) -> Result<Self, GeometryError> {
        dimensions(
            Vec3::new(value.x, 1.0, value.y),
            GeometryRole::PlanDimensions,
            true,
        )?;
        Ok(Self(value))
    }
    pub fn metres(self) -> Vec2 {
        self.0
    }
}
impl<'de> Deserialize<'de> for PlanDimensions {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec2::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Nonnegative finite X/Z extents, including point and line bounds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct PlanExtents(Vec2);
impl PlanExtents {
    pub fn from_metres(value: Vec2) -> Result<Self, GeometryError> {
        dimensions(
            Vec3::new(value.x, 0.0, value.y),
            GeometryRole::PlanExtents,
            false,
        )?;
        Ok(Self(value))
    }
    pub fn metres(self) -> Vec2 {
        self.0
    }
}
impl<'de> Deserialize<'de> for PlanExtents {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec2::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
