//! Authored camera profiles and checked temporal shaping coefficients.

use super::collision::CloseCameraProfile;
use adventuresim_tactical_core::animation::{AnimationElapsed, ResponseShapingTime};
use bevy::prelude::*;

// Checked during constant evaluation: no runtime input can reach this failure.
const DEFAULT_COLLISION_HOLD: AnimationElapsed = match AnimationElapsed::from_seconds(0.12) {
    Ok(value) => value,
    Err(_) => panic!("authored camera collision hold must be valid"),
};

const DEFAULT_FOLLOW_TIME: ResponseShapingTime = match ResponseShapingTime::from_seconds(0.18) {
    Ok(value) => value,
    Err(_) => panic!("authored camera shaping seconds must be finite"),
};

const DEFAULT_VERTICAL_FOLLOW_TIME: ResponseShapingTime =
    match ResponseShapingTime::from_seconds(0.34) {
        Ok(value) => value,
        Err(_) => panic!("authored camera shaping seconds must be finite"),
    };

const DEFAULT_RAISED_HORIZONTAL_TIME: ResponseShapingTime =
    match ResponseShapingTime::from_seconds(0.055) {
        Ok(value) => value,
        Err(_) => panic!("authored camera shaping seconds must be finite"),
    };

const DEFAULT_RAISED_VERTICAL_TIME: ResponseShapingTime =
    match ResponseShapingTime::from_seconds(0.08) {
        Ok(value) => value,
        Err(_) => panic!("authored camera shaping seconds must be finite"),
    };

const DEFAULT_COLLISION_RECOVERY_TIME: ResponseShapingTime =
    match ResponseShapingTime::from_seconds(0.32) {
        Ok(value) => value,
        Err(_) => panic!("authored camera shaping seconds must be finite"),
    };

#[derive(Debug, Clone, Copy)]
pub struct CameraProfile {
    pub distance: f32,
    pub shoulder_offset: f32,
    /// Height above the controller transform, which is centered in its capsule.
    pub focus_height: f32,
    pub horizontal_follow_time: ResponseShapingTime,
    pub vertical_follow_time: ResponseShapingTime,
    pub maximum_follow_error: f32,
    /// Half-extents in normalized screen coordinates.
    pub sweet_spot: Vec2,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraRigConfig {
    pub lowered: CameraProfile,
    pub raised: CameraProfile,
    pub transition_time: ResponseShapingTime,
    pub close: CloseCameraProfile,
    pub collision_margin: f32,
    pub collision_recovery_time: ResponseShapingTime,
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

impl Default for CameraRigConfig {
    fn default() -> Self {
        Self {
            lowered: CameraProfile {
                distance: 3.75,
                shoulder_offset: 0.0,
                focus_height: 0.48,
                horizontal_follow_time: DEFAULT_FOLLOW_TIME,
                vertical_follow_time: DEFAULT_VERTICAL_FOLLOW_TIME,
                maximum_follow_error: 0.85,
                sweet_spot: Vec2::new(0.09, 0.065),
            },
            raised: CameraProfile {
                distance: 2.75,
                shoulder_offset: 0.5,
                focus_height: 0.62,
                horizontal_follow_time: DEFAULT_RAISED_HORIZONTAL_TIME,
                vertical_follow_time: DEFAULT_RAISED_VERTICAL_TIME,
                maximum_follow_error: 0.22,
                sweet_spot: Vec2::splat(0.01),
            },
            transition_time: DEFAULT_FOLLOW_TIME,
            close: CloseCameraProfile::default(),
            collision_margin: 0.04,
            collision_recovery_time: DEFAULT_COLLISION_RECOVERY_TIME,
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
