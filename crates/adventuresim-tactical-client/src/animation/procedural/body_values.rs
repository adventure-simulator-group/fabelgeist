//! Finite signed quantities in the presentation body's local frame.

use crate::animation::sample_values::AnimationValueResult;
use adventuresim_building_generator::spatial_geometry::Radians;
use adventuresim_tactical_core::animation::{AnimationQuantity, AnimationValueError};
use bevy::prelude::*;

#[derive(Clone, Copy, derive_more::Debug)]
#[debug("BodyAngle({:?})", self.0.radians())]
pub(crate) struct BodyAngle(Radians);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyAngularVelocity(Vec2);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyLocalVelocity(Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyLocalAcceleration(Vec3);

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BodyResponseAngles {
    pub(super) pitch: BodyAngle,
    pub(super) roll: BodyAngle,
}

pub(super) struct BodyResponseStep {
    pub(super) angles: BodyResponseAngles,
    pub(super) angular_velocity: BodyAngularVelocity,
}

impl Default for BodyAngle {
    fn default() -> Self {
        Self::ZERO
    }
}
impl BodyAngle {
    pub(super) const ZERO: Self = Self(Radians::ZERO);
    pub(super) fn from_radians(value: f32) -> AnimationValueResult<Self> {
        Radians::new(value)
            .map(Self)
            .map_err(|_| AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyAngle,
            })
    }
    pub(crate) fn radians(self) -> f32 {
        self.0.radians()
    }
}
impl BodyAngularVelocity {
    pub(super) const ZERO: Self = Self(Vec2::ZERO);
    pub(super) fn from_radians_per_second(value: Vec2) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyAngularVelocity,
            });
        }
        Ok(Self(value))
    }
    pub(super) fn radians_per_second(self) -> Vec2 {
        self.0
    }
}
impl BodyLocalVelocity {
    pub(super) fn from_metres_per_second(value: Vec3) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyLocalVelocity,
            });
        }
        Ok(Self(value))
    }
    pub(super) fn metres_per_second(self) -> Vec3 {
        self.0
    }
}
impl BodyLocalAcceleration {
    pub(super) const ZERO: Self = Self(Vec3::ZERO);
    pub(super) fn from_metres_per_second_squared(value: Vec3) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyLocalAcceleration,
            });
        }
        Ok(Self(value))
    }
    pub(super) fn metres_per_second_squared(self) -> Vec3 {
        self.0
    }
}
impl BodyResponseAngles {
    /// Pitch/roll radians at the native two-axis response kernel boundary.
    pub(super) fn from_radians(value: Vec2) -> AnimationValueResult<Self> {
        Ok(Self {
            pitch: BodyAngle::from_radians(value.x)?,
            roll: BodyAngle::from_radians(value.y)?,
        })
    }
    pub(super) fn radians(self) -> Vec2 {
        Vec2::new(self.pitch.radians(), self.roll.radians())
    }
}
