//! Capture dimensions are physical pixels, independent of desktop scaling.
use bevy::prelude::UVec2;

const STANDARD_CAPTURE_PIXELS: UVec2 = UVec2::new(1280, 720);
pub(super) const CAPTURE_ASPECT_RATIO: f32 =
    STANDARD_CAPTURE_PIXELS.x as f32 / STANDARD_CAPTURE_PIXELS.y as f32;
// Match the frozen terrain comparison images. This is a capture setting, not
// a geographic measurement or a production gameplay resolution constraint.
const TERRAIN_COMPARISON_PIXELS: UVec2 = UVec2::new(2240, 1260);

pub(super) fn physical_pixels(profile: &str) -> UVec2 {
    if profile == super::fixed_city_cameras::PROFILE {
        TERRAIN_COMPARISON_PIXELS
    } else {
        STANDARD_CAPTURE_PIXELS
    }
}
