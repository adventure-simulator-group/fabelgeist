//! Authored camera profiles and checked temporal shaping coefficients.

use super::collision::CloseCameraProfile;
use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, UnitInterval,
};
use bevy::prelude::*;

// Checked during constant evaluation: no runtime input can reach this failure.
const DEFAULT_COLLISION_HOLD: AnimationElapsed = match AnimationElapsed::from_seconds(0.12) {
    Ok(value) => value,
    Err(_) => panic!("authored camera collision hold must be valid"),
};

/// Finite signed seconds shaping a camera response, not elapsed physical time.
/// Zero and negative coefficients remain valid; damping floors only after blend.
#[derive(Clone, Copy, PartialEq, derive_more::Debug)]
#[debug("{_0:?}")]
pub struct CameraShapingTime(f32);

#[derive(Debug, Clone, Copy)]
pub struct CameraProfile {
    pub distance: f32,
    pub shoulder_offset: f32,
    /// Height above the controller transform, which is centered in its capsule.
    pub focus_height: f32,
    pub horizontal_follow_time: CameraShapingTime,
    pub vertical_follow_time: CameraShapingTime,
    pub maximum_follow_error: f32,
    /// Half-extents in normalized screen coordinates.
    pub sweet_spot: Vec2,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraRigConfig {
    pub lowered: CameraProfile,
    pub raised: CameraProfile,
    pub transition_time: CameraShapingTime,
    pub close: CloseCameraProfile,
    pub collision_margin: f32,
    pub collision_recovery_time: CameraShapingTime,
    pub collision_hysteresis: f32,
    /// Seconds to hold a retracted view before recovering through a doorway.
    pub collision_hold_time: AnimationElapsed,
    pub teleport_distance: f32,
    pub crouch_focus_adjustment: f32,
    pub prone_focus_adjustment: f32,
    pub supine_focus_adjustment: f32,
    pub aim_distance: f32,
    pub muzzle_offset: Vec3,
}

impl CameraShapingTime {
    pub fn from_seconds(seconds: f32) -> AnimationValueResult<Self> {
        if !seconds.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::CameraShapingTime,
            });
        }
        Ok(Self(seconds))
    }

    pub(super) fn checked_lerp(
        self,
        other: Self,
        blend: UnitInterval,
    ) -> AnimationValueResult<Self> {
        let seconds = self.0.lerp(other.0, blend.fraction());
        Self::from_seconds(seconds).map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::CameraShapingTime,
        })
    }

    /// Project only inside the complete scalar/vector damping equation.
    pub(super) fn seconds(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for CameraShapingTime {
    type Error = AnimationValueError;
    fn try_from(seconds: f32) -> AnimationValueResult<Self> {
        Self::from_seconds(seconds)
    }
}

impl Default for CameraRigConfig {
    fn default() -> Self {
        Self {
            lowered: CameraProfile {
                distance: 3.75,
                shoulder_offset: 0.0,
                focus_height: 0.48,
                horizontal_follow_time: CameraShapingTime(0.18),
                vertical_follow_time: CameraShapingTime(0.34),
                maximum_follow_error: 0.85,
                sweet_spot: Vec2::new(0.09, 0.065),
            },
            raised: CameraProfile {
                distance: 2.75,
                shoulder_offset: 0.5,
                focus_height: 0.62,
                horizontal_follow_time: CameraShapingTime(0.055),
                vertical_follow_time: CameraShapingTime(0.08),
                maximum_follow_error: 0.22,
                sweet_spot: Vec2::splat(0.01),
            },
            transition_time: CameraShapingTime(0.18),
            close: CloseCameraProfile::default(),
            collision_margin: 0.04,
            collision_recovery_time: CameraShapingTime(0.32),
            collision_hysteresis: 0.08,
            collision_hold_time: DEFAULT_COLLISION_HOLD,
            teleport_distance: 2.0,
            crouch_focus_adjustment: -0.45,
            prone_focus_adjustment: -0.82,
            supine_focus_adjustment: -0.76,
            aim_distance: 100.0,
            // Local right, up, and forward from the controller center.
            muzzle_offset: Vec3::new(0.28, 0.55, 0.42),
        }
    }
}
