//! Checked constants and arithmetic for the built-in review-frame catalogue.

use adventuresim_building_generator::spatial_geometry::Radians;
use adventuresim_tactical_core::animation::AnimationElapsed;

/// These inputs are the authored catalogue's constants, not gameplay input.
pub(super) fn authored_frame_time(seconds: f32) -> AnimationElapsed {
    match AnimationElapsed::from_seconds(seconds) {
        Ok(elapsed) => elapsed,
        Err(error) => panic!("invalid authored review-frame time: {error}"),
    }
}

pub(super) fn authored_camera_angle(radians: f32) -> Radians {
    match Radians::new(radians) {
        Ok(angle) => angle,
        Err(error) => panic!("invalid authored review-camera angle: {error}"),
    }
}
