use super::{CoordinateAxis, GeometryError, GeometryFrame, GeometryRole};
use bevy::{
    math::{Dir2, Dir3, Vec2, Vec3},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Finite radians. Admission preserves the represented angle without wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct Radians(f32);
impl Radians {
    pub const ZERO: Self = Self(0.0);
    pub const QUARTER_TURN: Self = Self(std::f32::consts::FRAC_PI_2);
    pub const NEGATIVE_QUARTER_TURN: Self = Self(-std::f32::consts::FRAC_PI_2);
    pub const HALF_TURN: Self = Self(std::f32::consts::PI);
    pub const fn new(value: f32) -> Result<Self, GeometryError> {
        if !value.is_finite() {
            return Err(GeometryError::NonFinite {
                role: GeometryRole::Angle,
                axis: CoordinateAxis::Y,
            });
        }
        Ok(Self(value))
    }
    pub fn radians(self) -> f32 {
        self.0
    }
}
impl<'de> Deserialize<'de> for Radians {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(f32::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// A normalized plan direction in a declared frame, backed by Bevy's Dir2 owner.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent, bound = "")]
#[reflect(opaque)]
pub struct PlanDirection<F: GeometryFrame> {
    direction: Dir2,
    #[serde(skip)]
    frame: PhantomData<F>,
}
impl<F: GeometryFrame> PlanDirection<F> {
    pub fn from_vector(value: Vec2) -> Result<Self, GeometryError> {
        Dir2::new(value).map_err(|source| GeometryError::Direction { source })?;
        // Keep the generator's original reciprocal-multiply normalization.
        Self::from_normalized(value.normalize_or_zero())
    }
    pub fn from_normalized(value: Vec2) -> Result<Self, GeometryError> {
        Dir2::new(value).map_err(|source| GeometryError::Direction { source })?;
        if !value.is_normalized() {
            return Err(GeometryError::UnnormalizedDirection);
        }
        Ok(Self {
            direction: Dir2::new_unchecked(value),
            frame: PhantomData,
        })
    }
    pub fn vector(self) -> Vec2 {
        *self.direction
    }
    /// Lift X/Z into the same frame without changing the represented direction.
    pub fn spatial(self) -> SpatialDirection<F> {
        let direction = self.vector();
        SpatialDirection {
            direction: Dir3::new_unchecked(Vec3::new(direction.x, 0.0, direction.y)),
            frame: PhantomData,
        }
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for PlanDirection<F> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_normalized(Vec2::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Finite unit quaternion. Admission retains the represented native rotation;
/// it never repairs malformed or scaled quaternions by normalization.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent)]
#[reflect(opaque)]
pub struct RigidRotation(bevy::math::Quat);
impl RigidRotation {
    pub const IDENTITY: Self = Self(bevy::math::Quat::IDENTITY);
    pub fn from_quaternion(value: bevy::math::Quat) -> Result<Self, GeometryError> {
        if !value.is_finite() || !value.is_normalized() {
            return Err(GeometryError::InvalidRotation);
        }
        Ok(Self(value))
    }
    pub fn quaternion(self) -> bevy::math::Quat {
        self.0
    }
}
impl<'de> Deserialize<'de> for RigidRotation {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_quaternion(bevy::math::Quat::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// A normalized spatial direction, distinct from a signed metre displacement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Reflect)]
#[repr(transparent)]
#[serde(transparent, bound = "")]
#[reflect(opaque)]
pub struct SpatialDirection<F: GeometryFrame> {
    direction: Dir3,
    #[serde(skip)]
    frame: PhantomData<F>,
}
impl<F: GeometryFrame> SpatialDirection<F> {
    pub fn from_vector(value: Vec3) -> Result<Self, GeometryError> {
        Dir3::new(value).map_err(|source| GeometryError::Direction { source })?;
        Self::from_normalized(value.normalize_or_zero())
    }
    pub fn from_normalized(value: Vec3) -> Result<Self, GeometryError> {
        Dir3::new(value).map_err(|source| GeometryError::Direction { source })?;
        if !value.is_normalized() {
            return Err(GeometryError::UnnormalizedDirection);
        }
        Ok(Self {
            direction: Dir3::new_unchecked(value),
            frame: PhantomData,
        })
    }
    pub fn vector(self) -> Vec3 {
        *self.direction
    }
}
impl<'de, F: GeometryFrame> Deserialize<'de> for SpatialDirection<F> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_normalized(Vec3::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
